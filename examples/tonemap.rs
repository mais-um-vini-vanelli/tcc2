mod common;

use std::error::Error;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use tcc2::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut objects = vec![
        Object::camera(
            Camera::Perspective {
                fov: 50.0,
                near: 0.1,
                far: 1000.0,
            },
            Transform::looking_at(Vec3::new(0.0, 4.0, 14.0), Vec3::new(0.0, 1.0, 0.0)),
        ),
        Object::light(
            Light::Environment {
                color: Rgb::new(0.5, 0.6, 1.0),
                intensity: 0.02,
            },
            Transform::IDENTITY,
        ),
        Object::primitive(
            Primitive {
                geometry: Geometry::mesh(
                    vec![
                        Vec3::new(-40.0, 0.0, 40.0),
                        Vec3::new(40.0, 0.0, 40.0),
                        Vec3::new(40.0, 0.0, -40.0),
                        Vec3::new(-40.0, 0.0, -40.0),
                    ],
                    vec![0, 1, 2, 0, 2, 3],
                ),
                material: Material::diffuse(Rgb::gray(0.6)),
            },
            Transform::IDENTITY,
        ),
    ];

    for (position, color) in [
        (Vec3::new(-6.0, 1.5, 2.5), Rgb::new(1.0, 0.35, 0.08)),
        (Vec3::new(-2.0, 1.5, 2.5), Rgb::new(1.0, 0.08, 0.55)),
        (Vec3::new(2.0, 1.5, 2.5), Rgb::new(0.08, 0.7, 1.0)),
        (Vec3::new(6.0, 1.5, 2.5), Rgb::new(0.35, 1.0, 0.1)),
    ] {
        objects.push(Object::light(
            Light::Point {
                color,
                intensity: 150.0,
            },
            Transform {
                translation: position,
                ..Transform::IDENTITY
            },
        ));
    }

    for i in 0..5 {
        objects.push(Object::primitive(
            Primitive {
                geometry: Geometry::sphere(1.2),
                material: Material {
                    albedo: Channel::Constant(Rgb::gray(0.8)),
                    roughness: Channel::Constant(0.35),
                    ..Material::default()
                },
            },
            Transform {
                translation: Vec3::new(-8.0 + 4.0 * i as f32, 1.2, 0.0),
                ..Transform::IDENTITY
            },
        ));
    }

    let scene = Scene { objects };
    let mut params = Parameters::with_resolution(800, 450);

    let width = params.output.width as usize;
    let height = params.output.height as usize;

    let mut window = Window::new("tonemap", width, height, WindowOptions::default())?;
    window.set_target_fps(30);

    let mut pixels = vec![0u32; width * height];
    let mut tonemap = Tonemap::None;
    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            tonemap = match tonemap {
                Tonemap::None => Tonemap::Reinhard,
                Tonemap::Reinhard => Tonemap::None,
            };
            dirty = true;
        }

        if dirty {
            params.output.tonemap = tonemap;
            let result = Rasterizer.render(&scene, &params);

            for (dest, color) in pixels.iter_mut().zip(result.color.pixels()) {
                let [r, g, b] = color.to_srgb();
                *dest = u32::from_be_bytes([0, r, g, b]);
            }

            window.set_title(&format!("tonemap: {tonemap:?} - space to toggle"));
            dirty = false;
        }

        window.update_with_buffer(&pixels, width, height)?;
    }

    Ok(())
}
