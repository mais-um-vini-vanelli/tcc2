mod comum;

use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use tcc2::prelude::*;

const LARGURA: u32 = 800;
const ALTURA: u32 = 450;

// Uma repetição da textura a cada 4 m de chão.
const UV_POR_METRO: f32 = 0.25;

fn main() -> Result<(), Box<dyn Error>> {
    let textura = xadrez(512, 16);

    println!(
        "pirâmide: {} níveis, de {}x{} a 1x1",
        textura.mipmaps(),
        textura.width(),
        textura.height(),
    );

    let scene = Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 50.0,
                    aspect: LARGURA as f32 / ALTURA as f32,
                    near: 0.1,
                    far: 1000.0,
                },
                Transform::looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, -10.0)),
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
                    geometry: chao(500.0),
                    material: Material {
                        albedo: Channel::Map(textura),
                        roughness: Channel::Constant(1.0),
                        ..Material::default()
                    },
                },
                Transform::IDENTITY,
            ),
        ],
    };

    let base = Parameters {
        output: OutputOptions {
            width: LARGURA,
            height: ALTURA,
            ..OutputOptions::default()
        },
        ..Parameters::default()
    };

    for (filtering, nome) in [
        (TextureFilter::Nearest, "nearest"),
        (TextureFilter::Bilinear, "bilinear"),
        (TextureFilter::MipmapBilinear, "mipmap_bilinear"),
        (TextureFilter::Trilinear, "trilinear"),
    ] {
        let params = Parameters {
            shading: ShadingOptions {
                texture_filter: filtering,
                ..base.shading
            },
            ..base.clone()
        };

        let inicio = Instant::now();
        let resultado = Rasterizer.render(&scene, &params);
        let tempo = inicio.elapsed();

        let arquivo = format!("./renders/mipmaps_{nome}.png");
        comum::grava_textura(&arquivo, &resultado.color, Rgb::to_srgb)?;

        println!("{nome}: {tempo:.1?} -> {arquivo}");
    }

    Ok(())
}

// Um quadrado em y = 0, de lado 2·alcance, com a uv crescendo junto com x e z
// pra textura repetir pelo chão todo.
fn chao(alcance: f32) -> Geometry {
    let cantos = [
        Vec3::new(-alcance, 0.0, alcance),
        Vec3::new(alcance, 0.0, alcance),
        Vec3::new(alcance, 0.0, -alcance),
        Vec3::new(-alcance, 0.0, -alcance),
    ];

    Geometry::Mesh(Arc::new(Mesh {
        vertices: cantos.to_vec(),
        indices: vec![0, 1, 2, 0, 2, 3],
        normals: Some(vec![Vec3::new(0.0, 1.0, 0.0); 4]),
        tangents: None,
        uvs: Some(
            cantos
                .iter()
                .map(|p| Vec2::new(p.x, p.z) * UV_POR_METRO)
                .collect(),
        ),
    }))
}

fn xadrez(lado: u32, quadrados: u32) -> Arc<Texture> {
    let mut texels = Vec::with_capacity((lado * lado) as usize);

    for y in 0..lado {
        for x in 0..lado {
            let casa = (x * quadrados / lado) + (y * quadrados / lado);
            texels.push(if casa.is_multiple_of(2) {
                Rgb::gray(0.9)
            } else {
                Rgb::gray(0.05)
            });
        }
    }

    Arc::new(Texture::new(lado, lado, texels).unwrap())
}
