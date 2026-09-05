//! Renderiza **a mesma cena** nos dois modos e grava um .png de cada passe que
//! o renderer produziu.
//!
//! A cena tem um triângulo, um cubo e uma esfera. O rasterizador tessela tudo
//! em triângulos e desenha os três; o ray tracer, por enquanto, só intercepta a
//! esfera.
//!
//! ```text
//! cargo run --example dois_modos
//! ```

use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;

use tcc2::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
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
    .with_object(Primitive::cube(
        Vec3::new(0.0, 0.0, 0.0),
        0.8,
        Material::diffuse(Rgb::new(0.3, 0.8, 0.4)),
    ))
    .with_object(Primitive::sphere(
        Vec3::new(2.2, 0.0, 0.0),
        1.0,
        Material::diffuse(Rgb::new(0.2, 0.4, 0.9)),
    ));

    let params = Parameters::with_resolution(320, 240);

    renderiza(&Rasterizer, &scene, &params, "raster")?;
    renderiza(&RayTracer, &scene, &params, "raytracer")?;

    Ok(())
}

fn renderiza(
    renderer: &dyn Renderer,
    scene: &Scene,
    params: &Parameters,
    prefixo: &str,
) -> Result<(), Box<dyn Error>> {
    let resultado = renderer.render(scene, params);

    fs::create_dir_all("./renders")?;

    let mut gravados = Vec::new();

    for (passe, nome) in [
        (RenderPass::Color, "color"),
        (RenderPass::Depth, "depth"),
        (RenderPass::Normal, "normal"),
        (RenderPass::Uv, "uv"),
        (RenderPass::RayCount, "ray_count"),
        (RenderPass::BvhHeatmap, "bvh_heatmap"),
    ] {
        let Some(buffer) = resultado.get(passe) else {
            continue;
        };

        let arquivo = format!("./renders/{prefixo}_{nome}.png");
        grava_png(&arquivo, &buffer.to_color())?;

        gravados.push(nome);
    }

    let (largura, altura) = resultado
        .color
        .as_ref()
        .map(|b| (b.width(), b.height()))
        .unwrap_or((0, 0));

    println!(
        "{}: {largura}x{altura}, {} backfaces descartadas, passes: {}",
        renderer.name(),
        resultado.stats.backfaces_culled,
        gravados.join(", "),
    );

    Ok(())
}

fn grava_png(caminho: &str, buffer: &ColorBuffer) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::with_capacity(buffer.pixels().len() * 3);
    for pixel in buffer.pixels() {
        bytes.extend_from_slice(&pixel.to_bytes());
    }

    let mut encoder = png::Encoder::new(
        BufWriter::new(File::create(caminho)?),
        buffer.width(),
        buffer.height(),
    );
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut escritor = encoder.write_header()?;
    escritor.write_image_data(&bytes)?;
    escritor.finish()?;

    Ok(())
}
