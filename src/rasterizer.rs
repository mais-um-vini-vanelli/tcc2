use crate::color::Rgb;
use crate::math::{Baricentrics, Plane, Vec2, Vertex};
use crate::parameters::{Parameters, TextureFilter};
use crate::render::{RasterizationResult, RenderResult, Renderer};
use crate::scene::{CameraInstance, LightInstance, Material, Scene};
use crate::shading::{self, Surface};
use crate::texture::Footprint;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer;

impl Renderer for Rasterizer {
    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult {
        let (width, height) = (params.output.width, params.output.height);

        let (environment, camera_instance, light_instances, geometry_instances) =
            scene.flatten(params.transform.method);

        if let TextureFilter::MipmapBilinear | TextureFilter::Trilinear =
            params.shading.texture_filter
        {
            for inst in &geometry_instances {
                for tex in [
                    &inst.primitive.material.albedo.texture(),
                    &inst.primitive.material.metallic.texture(),
                    &inst.primitive.material.roughness.texture(),
                    &inst.primitive.material.emissive.texture(),
                    &inst.primitive.material.normal.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    tex.generate_mipmaps();
                }
            }
        }

        let mut result = RenderResult::new(width, height, environment);

        let Some(camera_instance) = camera_instance else {
            return result;
        };

        let frustum = camera_instance.local.frustum();

        let state = State {
            camera: &camera_instance,
            lights: &light_instances,
            params,
        };

        for (i, geometry_instance) in geometry_instances.iter().enumerate() {
            let mesh = geometry_instance.primitive.geometry.tessellate();

            for face_i in 0..mesh.face_count() {
                let Some(face) = mesh.face(face_i) else {
                    continue;
                };

                if let Some(on_triangle) = params.rasterization.on_triangle {
                    on_triangle(i, face_i);
                }

                // Leva os vértices de local -> world -> camera.
                let triangle = face.map(|v| Vertex {
                    position: camera_instance.to_camera_space(
                        geometry_instance
                            .transform_chain
                            .apply_point(v.position, params.transform.method),
                    ),
                    normal: geometry_instance
                        .transform_chain
                        .apply_normal(v.normal, params.transform.method),
                    tangent: geometry_instance
                        .transform_chain
                        .apply_tangent(v.tangent, params.transform.method),
                    uv: v.uv,
                });

                let triangles = Self::clip(triangle, &frustum, &mut result.rasterization);

                for triangle_camera in triangles {
                    // Posição em pixels, com o z de câmera guardado sem projeção.
                    let triangle_screen = triangle_camera.map(|v| Vertex {
                        position: camera_instance
                            .local
                            .to_screen_space(v.position, width, height)
                            .expect("o clipping já garante z >= near"),
                        ..v
                    });

                    let [a, b, c] = triangle_screen.map(|v| v.position.xy());

                    let edge_abc = Self::edge_function(a, b, c);
                    if edge_abc == 0.0 {
                        // Triângulo degenerado.
                        continue;
                    }

                    if params.rasterization.backface_cull && edge_abc > 0.0 {
                        result.rasterization.backfaces_culled += 1;
                        break;
                    }

                    let perspective_correct = camera_instance.is_perspective()
                        && params.rasterization.perspective_correct;

                    // O shading acontece no espaço de mundo.
                    let triangle_world =
                        triangle_camera.map(|v| camera_instance.from_camera_space(v.position));

                    let (min_x, min_y, max_x, max_y) = Self::bounding_box(a, b, c, width, height);

                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            // Usar o centro do pixel: o pixel (1u32, 1u32) tem centro em (1.5f32, 1.5f32).
                            let pixel = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                            let [edge_bc, edge_ca, edge_ab] = [
                                Self::edge_function(b, c, pixel),
                                Self::edge_function(c, a, pixel),
                                Self::edge_function(a, b, pixel),
                            ];

                            // Um ponto está dentro do triângulo se ele está do mesmo lado das 3 arestas.
                            let dentro = if edge_abc > 0.0 {
                                edge_bc >= 0.0 && edge_ca >= 0.0 && edge_ab >= 0.0
                            } else {
                                edge_bc <= 0.0 && edge_ca <= 0.0 && edge_ab <= 0.0
                            };

                            result.rasterization.primitive_tests += 1;

                            if !dentro {
                                continue;
                            }

                            if let Some(on_fragment) = params.rasterization.on_fragment {
                                on_fragment(x, y);
                            }

                            let screen_weights =
                                [edge_bc / edge_abc, edge_ca / edge_abc, edge_ab / edge_abc];

                            let baricentrics = if perspective_correct {
                                Baricentrics::perspective_corrected(
                                    screen_weights,
                                    triangle_screen.map(|v| v.position.z),
                                )
                            } else {
                                Baricentrics::new(screen_weights)
                            };

                            let z =
                                baricentrics.interpolate_f32(triangle_screen.map(|v| v.position.z));

                            if params.rasterization.depth_test {
                                let closest = result
                                    .rasterization
                                    .depth
                                    .get(x, y)
                                    .unwrap_or(f32::INFINITY);

                                if z >= closest {
                                    continue;
                                }
                            }

                            result.rasterization.depth.set(x, y, z);

                            let vertex = Vertex {
                                position: baricentrics.interpolate_vec3(triangle_world),
                                normal: baricentrics
                                    .interpolate_vec3(triangle_screen.map(|v| v.normal))
                                    // A normal interpolada entre três normais unitárias não sai unitária.
                                    .normalized(),
                                tangent: baricentrics
                                    .interpolate_tangent(triangle_screen.map(|v| v.tangent)),
                                uv: baricentrics.interpolate_vec2(triangle_screen.map(|v| v.uv)),
                            };

                            let surface = Self::fragment(
                                &state,
                                &geometry_instance.primitive.material,
                                triangle_screen,
                                vertex,
                                pixel,
                            );

                            result.color.set(
                                x,
                                y,
                                shading::shade(
                                    &surface,
                                    state.camera.view_from(surface.point),
                                    state.lights,
                                    &shading::always_visible,
                                    state.params.shading.reflectance_model,
                                ),
                            );

                            result.rasterization.normal.set(
                                x,
                                y,
                                // Converte os componentes da normal de [-1,1] para [0,1].
                                Rgb::new(
                                    surface.normal.x * 0.5 + 0.5,
                                    surface.normal.y * 0.5 + 0.5,
                                    surface.normal.z * 0.5 + 0.5,
                                ),
                            );

                            result.rasterization.uv.set(
                                x,
                                y,
                                Rgb::new(vertex.uv.x, vertex.uv.y, 0.0),
                            );
                        }
                    }
                }
            }
        }

        self.apply_output(&mut result, &params.output);

        result
    }
}

struct State<'a> {
    params: &'a Parameters,
    camera: &'a CameraInstance,
    lights: &'a [LightInstance],
}

impl Rasterizer {
    fn clip(
        triangle: [Vertex; 3],
        frustum: &[Plane; 6],
        stats: &mut RasterizationResult,
    ) -> Vec<[Vertex; 3]> {
        let mut poligon = triangle.to_vec();
        let mut clipped = false;

        for plane in frustum {
            if poligon.iter().all(|v| plane.distance(v.position) >= 0.0) {
                // Se todos os pontos do triângulo estão dentro do plano.
                continue;
            }

            poligon = Self::sutherland_and_hodgman(&poligon, *plane);
            clipped = true;

            if poligon.len() < 3 {
                // Se todos os pontos do triângulo estão fora do plano.
                break;
            }
        }

        if clipped {
            stats.polygons_clipped += 1;
        }

        let mut triangles = Vec::with_capacity(poligon.len() - 2);

        for i in 1..poligon.len().saturating_sub(1) {
            triangles.push([poligon[0], poligon[i], poligon[i + 1]]);
        }

        triangles
    }

    fn sutherland_and_hodgman(poligon: &[Vertex], plane: Plane) -> Vec<Vertex> {
        let mut result = Vec::with_capacity(poligon.len() + 1);

        for (i, &current_vertex) in poligon.iter().enumerate() {
            let next_vertex = poligon[(i + 1) % poligon.len()];

            let (d_current, d_next) = (
                plane.distance(current_vertex.position),
                plane.distance(next_vertex.position),
            );

            if d_current >= 0.0 {
                result.push(current_vertex);
            }

            if (d_current >= 0.0) != (d_next >= 0.0) {
                // Um ponto dentro e o outro fora do plano.
                let t = -d_current / (d_next - d_current);
                result.push(current_vertex.lerp(next_vertex, t));
            }
        }

        result
    }

    fn edge_function(a: Vec2, b: Vec2, p: Vec2) -> f32 {
        let u = b - a;
        let v = p - a;

        u.cross(v)
    }

    fn bounding_box(a: Vec2, b: Vec2, c: Vec2, width: u32, height: u32) -> (u32, u32, u32, u32) {
        let (min_x, min_y) = (
            a.x.min(b.x).min(c.x).floor().max(0.0) as u32,
            a.y.min(b.y).min(c.y).floor().max(0.0) as u32,
        );

        let (max_x, max_y) = (
            a.x.max(b.x)
                .max(c.x)
                .ceil()
                .min(width as f32 - 1.0)
                .max(0.0) as u32,
            a.y.max(b.y)
                .max(c.y)
                .ceil()
                .min(height as f32 - 1.0)
                .max(0.0) as u32,
        );

        (min_x, min_y, max_x, max_y)
    }

    fn fragment(
        state: &State,
        material: &Material,
        triangle_screen: [Vertex; 3],
        vertex: Vertex,
        pixel: Vec2,
    ) -> Surface {
        let [a, b, c] = triangle_screen.map(|v| v.position.xy());

        let edge_abc = Self::edge_function(a, b, c);

        let perspective_correct = (state.camera.is_perspective()
            && state.params.rasterization.perspective_correct)
            .then(|| triangle_screen.map(|v| v.position.z));

        let footprint = {
            let uv_at = |point: Vec2| {
                let screen = [
                    Self::edge_function(b, c, point) / edge_abc,
                    Self::edge_function(c, a, point) / edge_abc,
                    Self::edge_function(a, b, point) / edge_abc,
                ];

                let baricentrics = match perspective_correct {
                    Some(depths) => Baricentrics::perspective_corrected(screen, depths),
                    None => Baricentrics::new(screen),
                };

                baricentrics.interpolate_vec2(triangle_screen.map(|v| v.uv))
            };

            Footprint {
                dx: uv_at(Vec2::new(pixel.x + 1.0, pixel.y)) - vertex.uv,
                dy: uv_at(Vec2::new(pixel.x, pixel.y + 1.0)) - vertex.uv,
            }
        };

        let normal = material.apply_normal_map(
            vertex.uv,
            Some(footprint),
            state.params.shading.texture_filter,
            vertex.normal,
            vertex.tangent,
        );

        Surface {
            point: vertex.position,
            normal,
            uv: vertex.uv,
            material: material.sample(
                vertex.uv,
                Some(footprint),
                state.params.shading.texture_filter,
            ),
        }
    }
}
