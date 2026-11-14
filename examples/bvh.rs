mod common;

use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use tcc2::prelude::*;

const BALL: f32 = 0.45;
const ROOM: f32 = 24.0;

fn main() -> Result<(), Box<dyn Error>> {
    let scene = Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 55.0,
                    near: 0.1,
                    far: 300.0,
                },
                Transform::looking_at(Vec3::new(0.0, 1.9, 5.0), Vec3::new(0.0, -0.2, -2.0)),
            ),
            Object::light(
                Light::Point {
                    color: Rgb::new(1.0, 0.94, 0.85),
                    intensity: 26.0,
                },
                Transform {
                    translation: Vec3::new(1.6, 2.2, 2.4),
                    ..Transform::IDENTITY
                },
            ),
            Object::light(
                Light::Directional {
                    color: Rgb::new(0.6, 0.7, 0.95),
                    intensity: 0.18,
                },
                Transform::looking_at(Vec3::ZERO, Vec3::new(-0.4, -1.0, -0.5)),
            ),
            Object::light(
                Light::Environment {
                    color: Rgb::new(0.3, 0.4, 0.55),
                    intensity: 0.15,
                },
                Transform::IDENTITY,
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::Mesh(Arc::new(stadium())),
                    material: Material {
                        albedo: Channel::Constant(Rgb::new(0.78, 0.55, 0.3)),
                        roughness: Channel::Constant(0.5),
                        ..Material::default()
                    },
                },
                Transform::IDENTITY,
            ),
        ],
    };

    let params = Parameters::with_resolution(480, 320);
    let mut heatmaps = Vec::new();

    for (acceleration, name) in [
        (Acceleration::BvhSah, "sah"),
        (Acceleration::BvhMedian, "mediana"),
        // (Acceleration::BruteForce, "sem_bvh"),
    ] {
        let params = Parameters {
            ray_tracing: RayTracingOptions {
                acceleration,
                ..params.ray_tracing
            },
            ..params.clone()
        };

        let start = Instant::now();
        let result = RayTracer.render(&scene, &params);
        let stats = result.ray_tracing;

        println!(
            "{name}: {:.1?}, {} testes de primitivo, {} caixas testadas, construída em {:.1?}",
            start.elapsed(),
            stats.primitive_tests,
            stats.bvh_nodes_visited,
            stats.bvh_build_time,
        );

        if acceleration == Acceleration::BvhSah {
            common::write_png(
                "./examples/renders/bvh_color.png",
                &result.color,
                Rgb::to_srgb,
            )?;
        }

        if acceleration != Acceleration::BruteForce {
            heatmaps.push((name, stats.bvh_heatmap));
        }
    }

    // Os dois heatmaps na mesma escala (não usar a normalização pelo próprio máximo do
    // `ScalarBuffer::to_color`.
    let peak = heatmaps
        .iter()
        .flat_map(|(_, heatmap)| heatmap.pixels())
        .fold(0.0f32, |peak, v| peak.max(*v));

    for (name, heatmap) in &heatmaps {
        let pixels = heatmap
            .pixels()
            .iter()
            .map(|v| Rgb::gray(v / peak))
            .collect();
        let buffer = ColorBuffer::from_pixels(heatmap.width(), heatmap.height(), pixels)
            .expect("o heatmap tem width * height pixels");

        common::write_png(
            &format!("./examples/renders/bvh_heatmap_{name}.png"),
            &buffer,
            Rgb::to_srgb,
        )?;
    }

    Ok(())
}

/// A chaleira no estádio: uma esfera tesselada no meio de uma sala de cinco
/// quads enormes, tudo num mesh só. É um caso ruim da mediana.
fn stadium() -> Mesh {
    let (mut vertices, mut indices) = {
        let sphere = Geometry::sphere(BALL).tessellate();

        (sphere.vertices.clone(), sphere.indices.clone())
    };

    let floor = -BALL;
    let ceiling = 6.0;

    for quad in [
        [
            Vec3::new(-ROOM, floor, -ROOM),
            Vec3::new(ROOM, floor, -ROOM),
            Vec3::new(ROOM, floor, ROOM),
            Vec3::new(-ROOM, floor, ROOM),
        ],
        [
            Vec3::new(-ROOM, ceiling, -ROOM),
            Vec3::new(-ROOM, ceiling, ROOM),
            Vec3::new(ROOM, ceiling, ROOM),
            Vec3::new(ROOM, ceiling, -ROOM),
        ],
        [
            Vec3::new(-ROOM, floor, -ROOM),
            Vec3::new(-ROOM, ceiling, -ROOM),
            Vec3::new(ROOM, ceiling, -ROOM),
            Vec3::new(ROOM, floor, -ROOM),
        ],
        [
            Vec3::new(-ROOM, floor, -ROOM),
            Vec3::new(-ROOM, floor, ROOM),
            Vec3::new(-ROOM, ceiling, ROOM),
            Vec3::new(-ROOM, ceiling, -ROOM),
        ],
        [
            Vec3::new(ROOM, floor, -ROOM),
            Vec3::new(ROOM, ceiling, -ROOM),
            Vec3::new(ROOM, ceiling, ROOM),
            Vec3::new(ROOM, floor, ROOM),
        ],
    ] {
        let first = vertices.len() as u32;
        vertices.extend(quad);
        indices.extend([first, first + 2, first + 1, first, first + 3, first + 2]);
    }

    Mesh::new(vertices, indices)
}
