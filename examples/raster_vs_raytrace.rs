mod common;

use std::error::Error;
use std::sync::Arc;

use tcc2::prelude::*;

const LAMP: Vec3 = Vec3::new(-1.3, 1.6, 1.6);

fn main() -> Result<(), Box<dyn Error>> {
    let mut texels = Vec::with_capacity(512 * 512);

    for y in 0..512u32 {
        for x in 0..512u32 {
            let square = (x * 8 / 512) + (y * 8 / 512);
            texels.push(if square.is_multiple_of(2) {
                Rgb::gray(0.8)
            } else {
                Rgb::new(0.15, 0.17, 0.2)
            });
        }
    }

    let checker = Arc::new(Texture::new(512, 512, texels)?);

    let scene = Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 60.0,
                    near: 0.1,
                    far: 100.0,
                },
                Transform::looking_at(Vec3::new(0.0, 1.2, 6.0), Vec3::ZERO),
            ),
            Object::light(
                Light::Point {
                    color: Rgb::WHITE,
                    intensity: 140.0,
                },
                Transform {
                    translation: Vec3::new(3.0, 4.0, 5.0),
                    ..Transform::IDENTITY
                },
            ),
            Object::light(
                Light::Directional {
                    color: Rgb::new(0.5, 0.6, 1.0),
                    intensity: 0.35,
                },
                Transform::looking_at(Vec3::ZERO, Vec3::new(-0.6, -1.0, -0.4)),
            ),
            Object::light(
                Light::Environment {
                    color: Rgb::new(0.4, 0.5, 0.7),
                    intensity: 0.08,
                },
                Transform::IDENTITY,
            ),
            Object::light(
                Light::Point {
                    color: Rgb::new(1.0, 0.8, 0.5),
                    intensity: 8.0,
                },
                Transform {
                    translation: LAMP,
                    ..Transform::IDENTITY
                },
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::Mesh(Arc::new(Mesh {
                        vertices: vec![
                            Vec3::new(-6.0, -1.2, 4.0),
                            Vec3::new(6.0, -1.2, 4.0),
                            Vec3::new(6.0, -1.2, -8.0),
                            Vec3::new(-6.0, -1.2, -8.0),
                        ],
                        indices: vec![0, 1, 2, 0, 2, 3],
                        normals: Some(vec![Vec3::new(0.0, 1.0, 0.0); 4]),
                        tangents: None,
                        uvs: Some(vec![
                            Vec2::new(0.0, 0.0),
                            Vec2::new(4.0, 0.0),
                            Vec2::new(4.0, 4.0),
                            Vec2::new(0.0, 4.0),
                        ]),
                    })),
                    material: Material {
                        albedo: Channel::Map(checker),
                        roughness: Channel::Constant(1.0),
                        ..Material::default()
                    },
                },
                Transform::IDENTITY,
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::triangle([
                        Vec3::new(-3.4, -1.0, 0.0),
                        Vec3::new(-1.4, -1.0, 0.0),
                        Vec3::new(-2.4, 1.0, 0.0),
                    ]),
                    material: Material::diffuse(Rgb::new(0.9, 0.3, 0.2)),
                },
                Transform::IDENTITY,
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::cube(0.8),
                    material: Material {
                        albedo: Channel::Constant(Rgb::new(0.3, 0.8, 0.4)),
                        roughness: Channel::Constant(0.35),
                        ..Material::default()
                    },
                },
                Transform {
                    rotation: Vec3::new(25.0, 35.0, 0.0),
                    ..Transform::IDENTITY
                },
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::sphere(1.0),
                    material: Material {
                        albedo: Channel::Constant(Rgb::new(0.2, 0.4, 0.9)),
                        roughness: Channel::Constant(0.3),
                        ..Material::default()
                    },
                },
                Transform {
                    translation: Vec3::new(2.4, 0.0, 0.0),
                    scale: Vec3::new(1.0, 0.7, 1.0),
                    ..Transform::IDENTITY
                },
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::sphere(0.75),
                    material: Material::metal(Rgb::new(0.95, 0.93, 0.88), 0.04),
                },
                Transform {
                    translation: Vec3::new(-2.0, -0.45, 1.6),
                    ..Transform::IDENTITY
                },
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::sphere(0.35),
                    material: Material::emissive(Rgb::new(1.4, 1.0, 0.55)),
                },
                Transform {
                    translation: LAMP,
                    ..Transform::IDENTITY
                },
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::sphere(0.7),
                    material: Material {
                        albedo: Channel::Constant(Rgb::gray(0.02)),
                        roughness: Channel::Constant(0.03),
                        transparency: 0.9,
                        refractive_index: 1.5,
                        ..Material::default()
                    },
                },
                Transform {
                    translation: Vec3::new(1.0, -0.5, 2.4),
                    ..Transform::IDENTITY
                },
            ),
        ],
    };

    let params = Parameters::with_resolution(480, 320);

    write_passes(&Rasterizer.render(&scene, &params), "raster")?;
    write_passes(&RayTracer.render(&scene, &params), "raytracer")?;

    Ok(())
}

fn write_passes(result: &RenderResult, prefix: &str) -> Result<(), Box<dyn Error>> {
    for (buffer, name) in [
        (&result.color, "color"),
        (&result.rasterization.normal, "normal"),
        (&result.rasterization.uv, "uv"),
    ] {
        common::write_png(
            &format!("./examples/renders/{prefix}_{name}.png"),
            buffer,
            Rgb::to_srgb,
        )?;
    }

    for (buffer, name) in [
        (&result.rasterization.depth, "depth"),
        (&result.ray_tracing.ray_count, "ray_count"),
        (&result.ray_tracing.bvh_heatmap, "bvh_heatmap"),
    ] {
        common::write_png(
            &format!("./examples/renders/{prefix}_{name}.png"),
            &buffer.to_color(),
            Rgb::to_srgb,
        )?;
    }

    Ok(())
}
