use std::error::Error;
use std::f32::consts::TAU;
use std::sync::Arc;
use std::time::Instant;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use tcc2::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    let mut texels = Vec::with_capacity(512 * 512);

    for y in 0..512u32 {
        for x in 0..512u32 {
            let square = (x * 16 / 512) + (y * 16 / 512);
            texels.push(if square.is_multiple_of(2) {
                Rgb::gray(0.9)
            } else {
                Rgb::gray(0.05)
            });
        }
    }

    let corners = [
        Vec3::new(-500.0, 0.0, 500.0),
        Vec3::new(500.0, 0.0, 500.0),
        Vec3::new(500.0, 0.0, -500.0),
        Vec3::new(-500.0, 0.0, -500.0),
    ];

    let mut scene = Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 50.0,
                    near: 0.1,
                    far: 1000.0,
                },
                Transform::IDENTITY,
            ),
            Object::light(
                Light::Environment {
                    color: Rgb::WHITE,
                    intensity: 1.0,
                },
                Transform::IDENTITY,
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::Mesh(Arc::new(Mesh {
                        vertices: corners.to_vec(),
                        indices: vec![0, 1, 2, 0, 2, 3],
                        normals: Some(vec![Vec3::new(0.0, 1.0, 0.0); 4]),
                        tangents: None,
                        uvs: Some(corners.iter().map(|p| Vec2::new(p.x, p.z) * 0.25).collect()),
                    })),
                    material: Material {
                        albedo: Channel::Map(Arc::new(Texture::new(512, 512, texels)?)),
                        roughness: Channel::Constant(1.0),
                        ..Material::default()
                    },
                },
                Transform::IDENTITY,
            ),
        ],
    };

    let mut params = Parameters::with_resolution(800, 450);
    params.shading.texture_filter = TextureFilter::Nearest;

    let width = params.output.width as usize;
    let height = params.output.height as usize;

    let mut window = Window::new(
        &format!(
            "moiré: {:?} - space to cycle",
            params.shading.texture_filter
        ),
        width,
        height,
        WindowOptions::default(),
    )?;

    let mut pixels = vec![0u32; width * height];
    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            let filter = match params.shading.texture_filter {
                TextureFilter::Nearest => TextureFilter::Bilinear,
                TextureFilter::Bilinear => TextureFilter::MipmapBilinear,
                TextureFilter::MipmapBilinear => TextureFilter::Trilinear,
                TextureFilter::Trilinear => TextureFilter::Nearest,
            };

            params.shading.texture_filter = filter;
            window.set_title(&format!("moiré: {filter:?} - space to cycle"));
        }

        // A câmera vai e volta entre -6 e 6 em x, em 16 segundos.
        let x = 6.0 * (TAU * start.elapsed().as_secs_f32() / 16.0).sin();

        scene.objects[0].transform =
            Transform::looking_at(Vec3::new(x, 1.0, 0.0), Vec3::new(x, 0.0, -10.0));

        let result = Rasterizer.render(&scene, &params);

        for (dest, color) in pixels.iter_mut().zip(result.color.pixels()) {
            let [r, g, b] = color.to_srgb();
            *dest = u32::from_be_bytes([0, r, g, b]);
        }

        window.update_with_buffer(&pixels, width, height)?;
    }

    Ok(())
}
