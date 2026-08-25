//! Cria uma cena básica, escreve as cores dos pixels na mão e grava
//! o resultado (um gradiente) em um .ppm.
//!
//! ```text
//! cargo run --example gradiente
//! ```

use tcc2::output::write_ppm;
use tcc2::prelude::*;

fn main() -> std::io::Result<()> {
    let scene = Scene::new(Camera::Perspective {
        fov: 60.0,
        aspect: 4.0 / 3.0,
        near: 0.1,
        far: 100.0,
        position: Vec3::new(0.0, 0.0, 5.0),
        look_at: Vec3::ZERO,
        up: Vec3::new(0.0, 1.0, 0.0),
    })
    .with_light(Light::Ambient {
        color: Rgb::WHITE,
        intensity: 0.2,
    })
    .with_object(Primitive::triangle(
        [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        Material::diffuse(Rgb::new(0.9, 0.3, 0.2)),
    ))
    .with_object(Primitive::sphere(
        Vec3::new(2.0, 0.0, 0.0),
        1.0,
        Material::diffuse(Rgb::new(0.2, 0.4, 0.9)),
    ));

    let params = Parameters::with_resolution(256, 192);
    let (w, h) = (params.output.width, params.output.height);

    let mut imagem = ColorBuffer::filled(w, h, params.output.background.unwrap());
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / (w - 1) as f32;
            let v = y as f32 / (h - 1) as f32;
            imagem.set(x, y, Rgb::new(u, v, 0.25));
        }
    }

    write_ppm("gradiente.ppm", &imagem)?;

    println!(
        "cena: {} objeto(s), {} luz(es)",
        scene.objects.len(),
        scene.lights.len(),
    );
    println!("gravado gradiente.ppm ({w}x{h})");
    Ok(())
}
