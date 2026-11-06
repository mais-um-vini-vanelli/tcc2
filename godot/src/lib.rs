use std::cell::{Cell, RefCell};

use godot::classes::ArrayMesh;
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::prelude::*;

use tcc2::io::Reader;
use tcc2::prelude::{
    Acceleration, AccelerationStructures, Geometry, Mesh as Tcc2Mesh, Rasterizer, RayKind,
    RaySegment, RayTracer, RayTracingResult, RenderResult, Renderer, Rgb, Vec2, Vec3, Vertex,
};

const BOX_EDGES: [(usize, usize); 12] = [
    // Ao longo de x
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    // Ao longo de y
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    // Ao longo de z
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];

struct Tcc2Godot;

#[gdextension]
unsafe impl ExtensionLibrary for Tcc2Godot {}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Tessellate;

#[godot_api]
impl Tcc2Tessellate {
    #[func]
    fn sphere(radius: f32) -> Gd<ArrayMesh> {
        surface(&Geometry::sphere(radius).tessellate())
    }

    #[func]
    fn cube(half_edge: f32) -> Gd<ArrayMesh> {
        surface(&Geometry::cube(half_edge).tessellate())
    }

    #[func]
    fn triangle(a: Vector3, b: Vector3, c: Vector3) -> Gd<ArrayMesh> {
        let points = [a, b, c].map(|v| Vec3::new(v.x, v.y, v.z));
        surface(&Geometry::triangle(points).tessellate())
    }
}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Bvh;

#[godot_api]
impl Tcc2Bvh {
    #[func]
    fn aabbs(bytes: PackedByteArray, max_depth: u32, mut errors: Array<GString>) -> VarArray {
        let (scene, parameters) = match Reader::read(bytes.as_slice()) {
            Ok(read) => read,
            Err(error) => {
                errors.push(&GString::from(&error.to_string()));
                return VarArray::new();
            }
        };

        let transform_method = parameters.transform.method;
        let (_, _, _, geometry_instances) = scene.flatten(transform_method);

        let stats = RefCell::new(RayTracingResult::new(
            parameters.output.width,
            parameters.output.height,
        ));

        let acceleration = match parameters.ray_tracing.acceleration {
            Acceleration::BruteForce => Acceleration::default(),
            acceleration => acceleration,
        };

        let acceleration_structures = AccelerationStructures::build(
            &geometry_instances,
            acceleration,
            transform_method,
            &stats,
        );

        let mut levels: Vec<PackedVector3Array> = Vec::new();

        for (level, corners) in
            acceleration_structures.aabbs(&geometry_instances, transform_method, max_depth)
        {
            let i = level.max(1) as usize - 1;

            if levels.len() <= i {
                levels.resize(i + 1, PackedVector3Array::new());
            }

            for (a, b) in BOX_EDGES {
                levels[i].push(vector3(corners[a]));
                levels[i].push(vector3(corners[b]));
            }
        }

        let mut out = VarArray::new();

        for (i, lines) in levels.into_iter().enumerate() {
            if lines.is_empty() {
                continue;
            }

            out.push(
                &vdict! {
                    "level" => i as u32 + 1,
                    "lines" => &lines,
                }
                .to_variant(),
            );
        }

        out
    }
}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Rays;

#[derive(Default)]
struct Lines {
    hit: PackedVector3Array,
    miss: PackedVector3Array,
}

#[godot_api]
impl Tcc2Rays {
    #[func]
    fn trace_pixel(
        bytes: PackedByteArray,
        x: u32,
        y: u32,
        mut errors: Array<GString>,
    ) -> VarDictionary {
        let recording = Cell::new(false);
        let segments = RefCell::new(Vec::new());

        let (scene, mut parameters) = match Reader::read(bytes.as_slice()) {
            Ok(read) => read,
            Err(error) => {
                errors.push(&GString::from(&error.to_string()));
                return VarDictionary::new();
            }
        };

        if x >= parameters.output.width || y >= parameters.output.height {
            errors.push(&GString::from("pixel fora da imagem"));
            return VarDictionary::new();
        }

        let on_ray = |ray_x: u32, ray_y: u32| recording.set((ray_x, ray_y) == (x, y));
        parameters.ray_tracing.on_ray = Some(&on_ray);

        let on_segment = |segment: RaySegment| {
            if recording.get() {
                segments.borrow_mut().push(segment);
            }
        };
        parameters.ray_tracing.on_segment = Some(&on_segment);

        let far = {
            let (_, camera_instance, _, _) = scene.flatten(parameters.transform.method);

            let Some(camera_instance) = camera_instance else {
                errors.push(&GString::from("a cena não tem câmera"));
                return VarDictionary::new();
            };

            let (_, far) = camera_instance.local.clip_planes();
            far
        };

        let result = RayTracer.render(&scene, &parameters);

        let segments = segments.into_inner();

        let mut primary = Lines::default();
        let mut shadow = Lines::default();
        let mut reflection = Lines::default();
        let mut refraction = Lines::default();
        let mut normals = PackedVector3Array::new();

        for segment in segments {
            let lines = match segment.kind {
                RayKind::Primary => &mut primary,
                RayKind::Shadow => &mut shadow,
                RayKind::Reflection => &mut reflection,
                RayKind::Refraction => &mut refraction,
            };

            let end = segment.ray.at(if segment.t_max.is_finite() {
                segment.t_max
            } else {
                far
            });

            match segment.hit {
                Some(hit) => {
                    if segment.kind == RayKind::Shadow {
                        lines.hit.push(vector3(segment.ray.origin));
                        lines.hit.push(vector3(end));
                    } else {
                        normals.push(vector3(hit.point));
                        normals.push(vector3(hit.point + hit.normal));

                        lines.hit.push(vector3(segment.ray.origin));
                        lines.hit.push(vector3(hit.point));
                    }
                }

                None => {
                    lines.miss.push(vector3(segment.ray.origin));
                    lines.miss.push(vector3(end));
                }
            }
        }

        let color = result.color.get(x, y).unwrap_or(Rgb::BLACK);

        vdict! {
            "primary" => &vdict! { "hit" => &primary.hit, "miss" => &primary.miss },
            "shadow" => &vdict! { "hit" => &shadow.hit, "miss" => &shadow.miss },
            "reflection" => &vdict! { "hit" => &reflection.hit, "miss" => &reflection.miss },
            "refraction" => &vdict! { "hit" => &refraction.hit, "miss" => &refraction.miss },
            "normals" => &normals,
            "color" => Color::from_rgb(color.r, color.g, color.b),
        }
    }
}

fn vector3(v: Vec3) -> Vector3 {
    Vector3::new(v.x, v.y, v.z)
}

fn surface(mesh: &Tcc2Mesh) -> Gd<ArrayMesh> {
    let mut vertices = PackedVector3Array::new();
    let mut normals = PackedVector3Array::new();
    let mut tangents = PackedFloat32Array::new();
    let mut uvs = PackedVector2Array::new();

    for i in 0..mesh.face_count() {
        let Some(face) = mesh.face(i) else {
            continue;
        };

        for vertex in [face[0], face[2], face[1]] {
            vertices.push(Vector3::new(
                vertex.position.x,
                vertex.position.y,
                vertex.position.z,
            ));
            normals.push(Vector3::new(
                vertex.normal.x,
                vertex.normal.y,
                vertex.normal.z,
            ));
            tangents.push(vertex.tangent.direction.x);
            tangents.push(vertex.tangent.direction.y);
            tangents.push(vertex.tangent.direction.z);
            tangents.push(vertex.tangent.handedness);
            uvs.push(Vector2::new(vertex.uv.x, vertex.uv.y));
        }
    }

    let mut arrays = VarArray::new();
    arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
    arrays.set(ArrayType::VERTEX.ord() as usize, &vertices.to_variant());
    arrays.set(ArrayType::NORMAL.ord() as usize, &normals.to_variant());
    arrays.set(ArrayType::TANGENT.ord() as usize, &tangents.to_variant());
    arrays.set(ArrayType::TEX_UV.ord() as usize, &uvs.to_variant());

    let mut mesh = ArrayMesh::new_gd();
    mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
    mesh
}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Bake;

#[godot_api]
impl Tcc2Bake {
    #[func]
    fn apply_rgb_factor(data: PackedByteArray, factor: Color) -> PackedByteArray {
        let source = data.as_slice();
        let factor = Rgb::new(factor.r, factor.g, factor.b);

        let mut result = Vec::with_capacity(source.len());

        for texel in source.chunks_exact(4) {
            let color = Rgb::from_srgb([texel[0], texel[1], texel[2]]) * factor;

            result.extend_from_slice(&color.to_srgb());
            result.push(texel[3]);
        }

        PackedByteArray::from(result.as_slice())
    }

    #[func]
    fn apply_scalar_factor(data: PackedByteArray, channel: i64, factor: f32) -> PackedByteArray {
        let source = data.as_slice();
        let mut result = Vec::with_capacity(source.len() / 4);

        for texel in source.chunks_exact(4) {
            let level = match channel {
                0..=3 => texel[channel as usize] as u32,
                _ => (texel[0] as u32 + texel[1] as u32 + texel[2] as u32) / 3,
            };

            result.push(((level as f32 / 255.0 * factor).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }

        PackedByteArray::from(result.as_slice())
    }
}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Raster;

#[godot_api]
impl Tcc2Raster {
    #[func]
    fn triangle_stages(
        bytes: PackedByteArray,
        instance: i64,
        face: i64,
        screen_depth: f32,
        mut errors: Array<GString>,
    ) -> VarDictionary {
        let (scene, parameters) = match Reader::read(bytes.as_slice()) {
            Ok(read) => read,
            Err(error) => {
                errors.push(&GString::from(&error.to_string()));
                return VarDictionary::new();
            }
        };

        let transform_method = parameters.transform.method;
        let (width, height) = (parameters.output.width, parameters.output.height);

        let (_, camera_instance, _, geometry_instances) = scene.flatten(transform_method);

        let Some(camera_instance) = camera_instance else {
            errors.push(&GString::from("a cena não tem câmera"));
            return VarDictionary::new();
        };

        let Some(geometry_instance) = usize::try_from(instance)
            .ok()
            .and_then(|i| geometry_instances.get(i))
        else {
            errors.push(&GString::from("instância fora da cena"));
            return VarDictionary::new();
        };

        let mesh = geometry_instance.primitive.geometry.tessellate();

        let Some(vertices) = usize::try_from(face).ok().and_then(|i| mesh.face(i)) else {
            errors.push(&GString::from("face fora da malha"));
            return VarDictionary::new();
        };

        let local_space_out = {
            let mut result = PackedVector3Array::new();

            for vertex in vertices {
                result.push(vector3(vertex.position));
            }

            result
        };

        let camera_space = vertices.map(|v| Vertex {
            position: camera_instance.to_camera_space(
                geometry_instance
                    .transform_chain
                    .apply_point(v.position, transform_method),
            ),
            normal: geometry_instance
                .transform_chain
                .apply_normal(v.normal, transform_method),
            tangent: geometry_instance
                .transform_chain
                .apply_tangent(v.tangent, transform_method),
            uv: v.uv,
        });

        let aspect = width as f32 / height as f32;

        let frustum_out = {
            let mut result = PackedVector3Array::new();
            let corners = camera_instance.local.frustum_corners(aspect);

            for (a, b) in BOX_EDGES {
                result.push(vector3(camera_instance.from_camera_space(corners[a])));
                result.push(vector3(camera_instance.from_camera_space(corners[b])));
            }

            result
        };

        let planes = camera_instance.local.frustum(aspect);

        let poligon = {
            let mut stats = RenderResult::new(1, 1, Rgb::BLACK).rasterization;
            Rasterizer::clip(camera_space, &planes, &mut stats)
        };

        let world_space_out = {
            let mut result = PackedVector3Array::new();

            for vertex in camera_space {
                result.push(vector3(camera_instance.from_camera_space(vertex.position)));
            }

            result
        };

        let mut clip_space_out = PackedVector3Array::new();
        let mut screen_space_out = PackedVector3Array::new();
        let mut projection_lines = PackedVector3Array::new();

        for clip_space in &poligon {
            for vertex in clip_space {
                clip_space_out.push(vector3(camera_instance.from_camera_space(vertex.position)));
            }

            let [Some(a), Some(b), Some(c)] = clip_space.map(|v| {
                camera_instance
                    .local
                    .to_screen_space(v.position, width, height)
            }) else {
                errors.push(&GString::from("triângulo na frente do near"));
                continue;
            };

            if parameters.rasterization.backface_cull && edge_function(a.xy(), b.xy(), c.xy()) > 0.0
            {
                continue;
            }

            for (vertex, pixel) in clip_space.iter().zip([a, b, c]) {
                let projected = vector3(camera_instance.from_camera_space(
                    camera_instance.local.from_screen_space(
                        Vec3::new(pixel.x, pixel.y, screen_depth),
                        width,
                        height,
                    ),
                ));

                screen_space_out.push(projected);

                projection_lines.push(vector3(camera_instance.from_camera_space(vertex.position)));
                projection_lines.push(projected);
            }
        }

        let screen_plane = {
            let mut result = PackedVector3Array::new();

            for corner in [
                Vec3::new(0.0, 0.0, screen_depth),
                Vec3::new(width as f32, 0.0, screen_depth),
                Vec3::new(width as f32, height as f32, screen_depth),
                Vec3::new(0.0, height as f32, screen_depth),
            ] {
                result.push(vector3(
                    camera_instance.from_camera_space(
                        camera_instance
                            .local
                            .from_screen_space(corner, width, height),
                    ),
                ));
            }

            result
        };

        vdict! {
            "local" => &local_space_out,
            "world" => &world_space_out,
            "frustum" => &frustum_out,
            "clipped" => &clip_space_out,
            "screen_plane" => &screen_plane,
            "screen" => &screen_space_out,
            "projection" => &projection_lines,
        }
    }
}

fn edge_function(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b - a).cross(p - a)
}
