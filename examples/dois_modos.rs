//! Renderiza **a mesma cena** nos dois modos e grava um .png de cada passe que
//! o renderer produziu.
//!
//! A cena tem um chão de xadrez, um triângulo, um cubo e uma esfera. O
//! rasterizador tessela tudo em triângulos e desenha; o ray tracer,
//! por enquanto, só intercepta a esfera — mas os dois passam pelo mesmo
//! `shade()`.
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
        position: Vec3::new(0.0, 1.2, 6.0),
        look_at: Vec3::ZERO,
        up: Vec3::new(0.0, 1.0, 0.0),
    })
    .with_light(Light::Point {
        position: Vec3::new(3.0, 4.0, 5.0),
        color: Rgb::WHITE,
        intensity: 45.0,
    })
    .with_light(Light::Directional {
        direction: Vec3::new(-0.6, -1.0, -0.4),
        color: Rgb::new(0.5, 0.6, 1.0),
        intensity: 0.11,
    })
    .with_light(Light::Ambient {
        color: Rgb::new(0.4, 0.5, 0.7),
        intensity: 0.08,
    })
    .with_object(Object::new(chao(), Material::textured(xadrez(64, 8))))
    .with_object(Object::new(
        Primitive::triangle([
            Vec3::new(-3.4, -1.0, 0.0),
            Vec3::new(-1.4, -1.0, 0.0),
            Vec3::new(-2.4, 1.0, 0.0),
        ]),
        Material::diffuse(Rgb::new(0.9, 0.3, 0.2)),
    ))
    .with_object(
        Object::new(
            Primitive::cube(Vec3::ZERO, 0.8),
            Material {
                albedo: Value::Constant(Rgb::new(0.3, 0.8, 0.4)),
                roughness: Value::Constant(0.35),
                ..Material::default()
            },
        )
        .with_transform(Transform {
            rotation: Vec3::new(25.0, 35.0, 0.0),
            ..Transform::default()
        }),
    )
    .with_object(
        Object::new(
            Primitive::sphere(Vec3::ZERO, 1.0),
            Material {
                albedo: Value::Constant(Rgb::new(0.2, 0.4, 0.9)),
                roughness: Value::Constant(0.3),
                ..Material::default()
            },
        )
        .with_transform(Transform {
            translation: Vec3::new(2.4, 0.0, 0.0),
            scale: Vec3::new(1.0, 0.7, 1.0),
            ..Transform::default()
        }),
    );

    let params = Parameters::with_resolution(320, 240);

    renderiza(&Rasterizer, &scene, &params, "raster")?;
    renderiza(&RayTracer, &scene, &params, "raytracer")?;

    for (frequencia, nome) in [
        (ShadingFrequency::Flat, "flat"),
        (ShadingFrequency::Gouraud, "gouraud"),
        (ShadingFrequency::Phong, "phong"),
    ] {
        let params = Parameters {
            shading: ShadingOptions {
                frequency: frequencia,
                ..params.shading
            },
            ..params.clone()
        };

        let resultado = Rasterizer.render(&scene, &params);
        if let Some(buffer) = resultado.color.as_ref() {
            grava_png(&format!("./renders/shading_{nome}.png"), buffer)?;
        }
    }

    Ok(())
}

// Um quadrado no plano y = -1.2, com uv de 0 a 4 pra textura repetir quatro
// vezes em cada direção.
fn chao() -> Primitive {
    let y = -1.2;

    Primitive::Mesh(Mesh {
        vertices: vec![
            Vec3::new(-6.0, y, 4.0),
            Vec3::new(6.0, y, 4.0),
            Vec3::new(6.0, y, -8.0),
            Vec3::new(-6.0, y, -8.0),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        normals: Some(vec![Vec3::new(0.0, 1.0, 0.0); 4]),
        uvs: Some(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(0.0, 4.0),
        ]),
    })
}

fn xadrez(lado: u32, quadrados: u32) -> Texture {
    let mut texels = Vec::with_capacity((lado * lado) as usize);

    for y in 0..lado {
        for x in 0..lado {
            let casa = (x * quadrados / lado) + (y * quadrados / lado);
            texels.push(if casa.is_multiple_of(2) {
                Rgb::gray(0.8)
            } else {
                Rgb::new(0.15, 0.17, 0.2)
            });
        }
    }

    Texture::new(lado, lado, texels).unwrap()
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
    fs::create_dir_all("./renders")?;

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
