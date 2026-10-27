use std::cell::RefCell;

use crate::color::Rgb;
use crate::math::{Ray, Vec3};
use crate::parameters::{Acceleration, Parameters};
use crate::render::{RayKind, RaySegment, RaySegmentHit, RayTracingResult, RenderResult, Renderer};
use crate::scene::{Camera, CameraInstance, GeometryInstance, Light, LightInstance, Scene};
use crate::shading::{self, Surface};

use crate::bvh::AccelerationStructures;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RayTracer;

impl Renderer for RayTracer {
    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult {
        let (width, height) = (params.output.width, params.output.height);

        let (environment, camera_instance, light_instances, geometry_instances) =
            scene.flatten(params.transform.method);

        let mut result = RenderResult::new(width, height, environment);

        let Some(camera_instance) = camera_instance else {
            return result;
        };

        let ray_tracing = RefCell::new(RayTracingResult::new(width, height));

        let acceleration_structures = (params.ray_tracing.acceleration != Acceleration::BruteForce)
            .then(|| {
                AccelerationStructures::build(
                    &geometry_instances,
                    params.ray_tracing.acceleration,
                    params.transform.method,
                    &ray_tracing,
                )
            });

        let state = State {
            lights: light_instances,
            environment,
            geometry_instances,
            params,
            acceleration_structures,
            ray_tracing,
        };

        let (near, far) = camera_instance.local.clip_planes();

        for y in 0..height {
            for x in 0..width {
                if let Some(on_ray) = params.ray_tracing.on_ray {
                    on_ray(x, y);
                }

                let stats_snapshot = {
                    let stats = state.ray_tracing.borrow();
                    (stats.rays, stats.bvh_nodes_visited)
                };

                let ray = Self::primary_ray(&camera_instance, x, y, width, height);
                state.ray_tracing.borrow_mut().rays += 1;

                result.color.set(
                    x,
                    y,
                    Self::trace(&state, ray, near, far, 0, RayKind::Primary),
                );

                let mut stats = state.ray_tracing.borrow_mut();

                let (rays, bvh_nodes_visited) = (
                    stats.rays - stats_snapshot.0,
                    stats.bvh_nodes_visited - stats_snapshot.1,
                );

                stats.ray_count.set(x, y, rays as f32);
                stats.bvh_heatmap.set(x, y, bvh_nodes_visited as f32);
            }
        }

        self.apply_output(&mut result, &params.output);

        result.ray_tracing = state.ray_tracing.into_inner();

        result
    }
}

impl RayTracer {
    const EPSILON: f32 = 1e-3;

    /// Segue o raio de luz no sentido contrário: da câmera para a luz.
    ///
    /// ```text
    /// L = Le                                     <- shading::shade, o emitido
    ///   + Σ_luzes V · reflectancia · radiancia   <- shading::shade, o termo direto
    ///   + F(v, n) · L(raio refletido)            <- recursão
    ///   + kt      · L(raio refratado)            <- recursão
    /// ```
    fn trace(state: &State, ray: Ray, t_min: f32, t_max: f32, depth: u32, kind: RayKind) -> Rgb {
        let hit = Self::closest_hit(state, ray, t_min, t_max, false);

        Self::on_segment(state, kind, ray, hit.as_ref(), depth, t_max);

        let Some(hit) = hit else {
            return state.environment;
        };

        let material_sample = hit.surface.material;
        let normal = hit.surface.normal;

        let ray_dir = ray.direction.normalized();
        let view = -ray_dir;

        let direct_color = {
            let visibility = |surface: &Surface, light_instance: &LightInstance| {
                if !(&state.params.ray_tracing).shadows {
                    return 1.0;
                }

                if let Some((shadow_dir, t_max)) =
                    Self::shadow_ray_dir(surface.point, light_instance)
                {
                    let shadow_ray = Ray::new(
                        Self::offset_ray_origin(surface.point, hit.geometric_normal, shadow_dir),
                        shadow_dir,
                    );

                    state.ray_tracing.borrow_mut().rays += 1;

                    let blocker = Self::closest_hit(state, shadow_ray, Self::EPSILON, t_max, true);

                    Self::on_segment(
                        state,
                        RayKind::Shadow,
                        shadow_ray,
                        blocker.as_ref(),
                        depth,
                        t_max,
                    );

                    if blocker.is_some() {
                        return 0.0;
                    }
                }

                1.0
            };

            shading::shade(
                &hit.surface,
                view,
                &state.lights,
                &visibility,
                state.params.shading.reflectance_model,
            )
        };

        if depth >= state.params.ray_tracing.max_depth {
            return direct_color;
        }

        let mut result = direct_color;

        // O fresnel define quanta luz é refletida e quanta é refratada.
        let fresnel =
            shading::fresnel_schlick(shading::f0(&material_sample), normal.dot(view).abs());

        if state.params.ray_tracing.reflections && material_sample.roughness < 0.1 {
            let reflection_dir = Self::reflection_ray_dir(ray_dir, normal);
            let reflection_ray = Ray::new(
                Self::offset_ray_origin(hit.surface.point, hit.geometric_normal, reflection_dir),
                reflection_dir,
            );

            state.ray_tracing.borrow_mut().rays += 1;

            let reflection_color = Self::trace(
                state,
                reflection_ray,
                Self::EPSILON,
                f32::INFINITY,
                depth + 1,
                RayKind::Reflection,
            );

            result = result + fresnel * reflection_color;
        }

        if state.params.ray_tracing.refractions && material_sample.transparency > 0.0 {
            let refraction_dir =
                Self::refraction_ray_dir(ray_dir, normal, material_sample.refractive_index)
                    .unwrap_or_else(|| {
                        // Reflexão interna total: o raio de refração vira um raio de reflexão.
                        Self::reflection_ray_dir(ray_dir, normal)
                    });
            let refraction_ray = Ray::new(
                Self::offset_ray_origin(hit.surface.point, hit.geometric_normal, refraction_dir),
                refraction_dir,
            );

            state.ray_tracing.borrow_mut().rays += 1;

            let refraction_color = Self::trace(
                state,
                refraction_ray,
                Self::EPSILON,
                f32::INFINITY,
                depth + 1,
                RayKind::Refraction,
            );

            // Desconsiderar a luz que foi refletida (fresnel) e a luz que voltou como brilho difuso
            // (1 - transparency).
            let factor = (Rgb::WHITE - fresnel) * material_sample.transparency;
            result = result + refraction_color * factor;
        }

        result
    }

    fn on_segment(
        state: &State,
        kind: RayKind,
        ray: Ray,
        hit: Option<&SurfaceHit>,
        depth: u32,
        t_max: f32,
    ) {
        if let Some(callback) = state.params.ray_tracing.on_segment {
            callback(RaySegment {
                kind,
                ray,
                hit: hit.map(|hit| RaySegmentHit {
                    point: hit.surface.point,
                    normal: hit.surface.normal,
                }),
                depth,
                t_max,
            });
        }
    }

    /// O acerto mais próximo da origem do raio, entre `t_min` e `t_max`.
    fn closest_hit(
        state: &State,
        ray: Ray,
        t_min: f32,
        t_max: f32,
        in_shadow_ray: bool,
    ) -> Option<SurfaceHit> {
        let (i, hit) = if let Some(acceleration_structures) = &state.acceleration_structures {
            acceleration_structures.closest_hit(
                &state.geometry_instances,
                ray,
                in_shadow_ray,
                t_min,
                t_max,
                state.params.transform.method,
                &state.ray_tracing,
            )?
        } else {
            let mut t_max = t_max;
            let mut closest = None;

            for (i, geometry_instance) in state.geometry_instances.iter().enumerate() {
                if in_shadow_ray && geometry_instance.primitive.material.is_emissive() {
                    continue;
                }

                let ray_local = geometry_instance
                    .transform_chain
                    .undo_ray(ray, state.params.transform.method);

                let hit = geometry_instance.primitive.geometry.intersect(ray_local);

                state.ray_tracing.borrow_mut().primitive_tests += 1;

                if let Some(hit) = hit {
                    if hit.t > t_min && hit.t < t_max {
                        t_max = hit.t;
                        closest = Some((i, hit));
                    }
                }
            }

            closest?
        };

        let geometry_instance = &state.geometry_instances[i];
        let transform_method = state.params.transform.method;

        let material = &geometry_instance.primitive.material;
        let normal = material.apply_normal_map(
            hit.uv,
            // Sem footprint por enquanto: os mipmaps são ignorados.
            None,
            state.params.shading.texture_filter,
            geometry_instance
                .transform_chain
                .apply_normal(hit.shading_normal, transform_method),
            geometry_instance
                .transform_chain
                .apply_tangent(hit.tangent, transform_method),
        );

        Some(SurfaceHit {
            surface: Surface {
                point: ray.at(hit.t),
                normal,
                uv: hit.uv,
                material: material.sample(hit.uv, None, state.params.shading.texture_filter),
            },
            geometric_normal: geometry_instance
                .transform_chain
                .apply_normal(hit.geometric_normal, transform_method),
        })
    }

    /// Monta um raio primário que passa pelo centro do pixel `(x, y)`.
    fn primary_ray(
        camera_instance: &CameraInstance,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Ray {
        let (right, up, forward) = camera_instance.basis;

        // Calcula o centro do pixel e converte pra [-1, 1].
        let u = 2.0 * (x as f32 + 0.5) / width as f32 - 1.0;
        let v = 1.0 - 2.0 * (y as f32 + 0.5) / height as f32;

        let aspect = width as f32 / height as f32;

        match camera_instance.local {
            Camera::Perspective { fov, .. } => {
                let half_height = (fov.to_radians() * 0.5).tan();
                let direction =
                    forward + right * (u * half_height * aspect) + up * (v * half_height);

                Ray::new(camera_instance.position, direction)
            }

            Camera::Orthographic {
                height: view_height,
                ..
            } => {
                let half_height = view_height * 0.5;
                let origem = camera_instance.position
                    + right * (u * half_height * aspect)
                    + up * (v * half_height);

                Ray::new(origem, forward)
            }
        }
    }

    /// Monta um raio de sombra, da superfície até a luz.
    fn shadow_ray_dir(point: Vec3, light_instance: &LightInstance) -> Option<(Vec3, f32)> {
        let (direction, t_max) = match light_instance.local {
            Light::Point { .. } | Light::Spot { .. } => {
                let to_light = light_instance.position - point;
                (to_light.normalized(), to_light.length())
            }

            Light::Directional { .. } => (-light_instance.forward(), f32::INFINITY),

            Light::Environment { .. } => return None,
        };

        Some((direction, t_max))
    }

    /// Monta um raio de refração usando a lei de Snell.
    fn refraction_ray_dir(direction: Vec3, normal: Vec3, refractive_index: f32) -> Option<Vec3> {
        let cos_i = -direction.dot(normal);

        // cos_i negativo é o raio saindo de dentro do objeto: a normal aponta pro
        // lado oposto de -d, e os dois meios trocam de papel.
        let (normal, eta, cos_i) = if cos_i < 0.0 {
            (-normal, refractive_index, -cos_i)
        } else {
            (normal, 1.0 / refractive_index, cos_i)
        };

        // sin²θ₂ = η² · (1 − cos²θ₁)
        let sen2_t = eta * eta * (1.0 - cos_i * cos_i);
        if sen2_t > 1.0 {
            // Caso de refração interna total: sin θ₂ é maior que 1 (inválido)
            return None;
        }

        let cos_t = (1.0 - sen2_t).sqrt();

        // t = η·d + (η·cos θ₁ − cos θ₂)·n
        Some(direction * eta + normal * (eta * cos_i - cos_t))
    }

    /// Monta um raio de reflexão, espelhando `direction` em torno de `normal`.
    fn reflection_ray_dir(direction: Vec3, normal: Vec3) -> Vec3 {
        direction - normal * (2.0 * direction.dot(normal))
    }

    /// Afasta a origem do raio da superfície, na direção da normal.
    fn offset_ray_origin(point: Vec3, normal: Vec3, direction: Vec3) -> Vec3 {
        let sign = if direction.dot(normal) < 0.0 {
            -1.0
        } else {
            1.0
        };

        point + normal * (Self::EPSILON * sign)
    }
}

struct State<'a> {
    params: &'a Parameters<'a>,
    environment: Rgb,
    lights: Vec<LightInstance>,
    geometry_instances: Vec<GeometryInstance<'a>>,
    acceleration_structures: Option<AccelerationStructures>,
    ray_tracing: RefCell<RayTracingResult>,
}

struct SurfaceHit {
    surface: Surface,
    geometric_normal: Vec3,
}
