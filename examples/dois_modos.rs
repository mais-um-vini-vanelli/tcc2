//! Renderiza **a mesma cena** nos dois modos e grava um .ppm de cada.
//!
//! A cena tem um triângulo e uma esfera, e por enquanto o rasterizador só
//! desenha o triângulo e o ray tracer só a esfera.
//!
//! ```text
//! cargo run --example dois_modos
//! ```

use tcc2::output::write_ppm;
use tcc2::prelude::*;

fn main() -> std::io::Result<()> {
    let scene = Scene::new(Camera::Perspective {
        fov: 60.0,
        aspect: 4.0 / 3.0,
        near: 0.1,
        far: 100.0,
        position: Vec3::new(0.0, 0.0, 6.0),
        look_at: Vec3::ZERO,
        up: Vec3::new(0.0, 1.0, 0.0),
    })
    .with_light(Light::Point {
        position: Vec3::new(3.0, 4.0, 5.0),
        color: Rgb::WHITE,
        intensity: 1.0,
    })
    .with_object(Primitive::triangle(
        [
            Vec3::new(-2.6, -1.0, 0.0),
            Vec3::new(-0.6, -1.0, 0.0),
            Vec3::new(-1.6, 1.0, 0.0),
        ],
        Material::diffuse(Rgb::new(0.9, 0.3, 0.2)),
    ))
    .with_object(Primitive::sphere(
        Vec3::new(1.6, 0.0, 0.0),
        1.0,
        Material::diffuse(Rgb::new(0.2, 0.4, 0.9)),
    ));

    let params = Parameters::with_resolution(320, 240);

    renderiza(&Rasterizer, &scene, &params, "./renders/raster.ppm")?;
    renderiza(&RayTracer, &scene, &params, "./renders/raytracer.ppm")?;

    Ok(())
}

fn renderiza(
    renderer: &dyn Renderer,
    scene: &Scene,
    params: &Parameters,
    arquivo: &str,
) -> std::io::Result<()> {
    let resultado = renderer.render(scene, params, &RenderPass::SHARED);

    let imagem = resultado.color().unwrap();
    write_ppm(arquivo, imagem)?;

    println!(
        "{}: {arquivo} ({}x{})",
        renderer.name(),
        imagem.width(),
        imagem.height(),
    );

    Ok(())
}
