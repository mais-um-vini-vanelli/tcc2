//! O problema que o mipmap resolve, desenhado pelo `Rasterizer`.
//!
//! Um chão de xadrez visto rasante até o horizonte, em três filtros:
//!
//! - `nearest`: um texel do nível 0. Longe da câmera o pixel cobre dezenas de
//!   texels e fica com um só, e o xadrez vira moiré.
//! - `bilinear`: com mipmap, no nível mais próximo. O moiré some, e a troca de
//!   nível aparece como faixas no chão.
//! - `trilinear`: interpola os dois níveis vizinhos, e as faixas somem.
//!
//! A cena só tem luz ambiente branca de intensidade 1, então o shading devolve
//! o albedo sem mexer: a cor do pixel é a amostra da textura, e a única coisa
//! mudando entre as imagens é o filtro.
//!
//! ```text
//! cargo run --release --example mipmaps
//! ```

mod comum;

use std::error::Error;
use std::time::Instant;

use tcc2::prelude::*;

const LARGURA: u32 = 800;
const ALTURA: u32 = 450;

const CEU: Rgb = Rgb::new(0.6, 0.75, 0.95);

// Uma repetição da textura a cada 4 m de chão.
const UV_POR_METRO: f32 = 0.25;

fn main() -> Result<(), Box<dyn Error>> {
    let textura = xadrez(512, 16);

    println!(
        "pirâmide: {} níveis, de {}x{} a 1x1",
        textura.miplevels(),
        textura.width(),
        textura.height(),
    );

    let scene = Scene::new(Camera::Perspective {
        fov: 50.0,
        aspect: LARGURA as f32 / ALTURA as f32,
        near: 0.1,
        far: 1000.0,
        position: Vec3::new(0.0, 1.0, 0.0),
        look_at: Vec3::new(0.0, 0.0, -10.0),
        up: Vec3::new(0.0, 1.0, 0.0),
    })
    .with_light(Light::Ambient {
        color: Rgb::WHITE,
        intensity: 1.0,
    })
    .with_object(Object::new(chao(500.0), Material::textured(textura)));

    let base = Parameters {
        output: OutputOptions {
            width: LARGURA,
            height: ALTURA,
            background: Some(CEU),
            ..OutputOptions::default()
        },
        ..Parameters::default()
    };

    for (texturing, nome) in [
        (
            TextureOptions {
                mipmaps: false,
                filtering: Filtering::Nearest,
            },
            "nearest",
        ),
        (
            TextureOptions {
                mipmaps: true,
                filtering: Filtering::Bilinear,
            },
            "bilinear",
        ),
        (
            TextureOptions {
                mipmaps: true,
                filtering: Filtering::Trilinear,
            },
            "trilinear",
        ),
    ] {
        let params = Parameters {
            texturing,
            ..base.clone()
        };

        let inicio = Instant::now();
        let resultado = Rasterizer.render(&scene, &params);
        let tempo = inicio.elapsed();

        let Some(imagem) = resultado.color.as_ref() else {
            continue;
        };

        let arquivo = format!("./renders/mipmaps_{nome}.png");
        comum::grava_textura(&arquivo, imagem, Rgb::to_srgb)?;

        println!("{nome}: {tempo:.1?} -> {arquivo}");
    }

    Ok(())
}

// Um quadrado em y = 0, de lado 2·alcance, com a uv crescendo junto com x e z
// pra textura repetir pelo chão todo.
fn chao(alcance: f32) -> Primitive {
    let cantos = [
        Vec3::new(-alcance, 0.0, alcance),
        Vec3::new(alcance, 0.0, alcance),
        Vec3::new(alcance, 0.0, -alcance),
        Vec3::new(-alcance, 0.0, -alcance),
    ];

    Primitive::Mesh(Mesh {
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
    })
}

fn xadrez(lado: u32, quadrados: u32) -> Texture {
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

    Texture::new(lado, lado, texels).unwrap()
}
