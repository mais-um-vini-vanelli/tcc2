mod comum;

use std::error::Error;
use std::f32::consts::TAU;

use tcc2::glb;
use tcc2::prelude::*;

const ARQUIVO: &str = "./renders/cena.tcc2.glb";
const LARGURA: u32 = 480;
const ALTURA: u32 = 360;

fn main() -> Result<(), Box<dyn Error>> {
    let cena = cena()?;
    let params = Parameters {
        output: OutputOptions {
            background: Some(Rgb::new(0.02, 0.03, 0.05)),
            exposure: 1.0,
            tonemap: Tonemap::Reinhard,
            ..Parameters::with_resolution(LARGURA, ALTURA).output
        },
        ..Parameters::default()
    };

    let bytes = glb::encode(&cena, Some(&params))?;
    std::fs::create_dir_all("./renders")?;
    std::fs::write(ARQUIVO, &bytes)?;
    println!("{ARQUIVO}: {} KiB", bytes.len() / 1024);

    let (volta, params_volta) = glb::decode(&std::fs::read(ARQUIVO)?)?;

    for (nome, cena, params) in [
        ("ida", &cena, &params),
        ("volta", &volta, &params_volta.unwrap()),
    ] {
        let resultado = RayTracer.render(cena, &params);
        let buffer = resultado.color.as_ref().ok_or("o ray tracer não deu cor")?;

        comum::grava_textura(
            &format!("./renders/formato_{nome}.png"),
            buffer,
            Rgb::to_srgb,
        )?;
    }
    println!("\nrenders/formato_ida.png e renders/formato_volta.png");

    Ok(())
}

fn cena() -> Result<Scene, Box<dyn Error>> {
    let chao = Mesh {
        vertices: vec![
            Vec3::new(-6.0, 0.0, -6.0),
            Vec3::new(6.0, 0.0, -6.0),
            Vec3::new(6.0, 0.0, 6.0),
            Vec3::new(-6.0, 0.0, 6.0),
        ],
        indices: vec![0, 2, 1, 0, 3, 2],
        normals: Some(vec![Vec3::new(0.0, 1.0, 0.0); 4]),
        // Sem tangentes de propósito: o `None` tem que voltar `None`, senão o
        // renderer deixa de montar a base pela uv do triângulo.
        tangents: None,
        uvs: Some(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(0.0, 3.0),
        ]),
    };

    let cena = Scene::new(Camera::Perspective {
        fov: 42.0,
        aspect: LARGURA as f32 / ALTURA as f32,
        near: 0.1,
        far: 60.0,
        position: Vec3::new(4.5, 3.2, 7.0),
        look_at: Vec3::new(0.0, 1.0, 0.0),
        up: Vec3::new(0.0, 1.0, 0.0),
    })
    .with_light(Light::Point {
        position: Vec3::new(3.0, 6.0, 4.0),
        color: Rgb::WHITE,
        intensity: 220.0,
    })
    .with_light(Light::Directional {
        direction: Vec3::new(-0.5, -0.8, -0.4),
        color: Rgb::new(0.6, 0.7, 1.0),
        intensity: 0.7,
    })
    .with_light(Light::Spot {
        position: Vec3::new(-4.0, 5.0, 2.0),
        direction: Vec3::new(0.6, -1.0, -0.2),
        inner_angle: 12.0,
        outer_angle: 26.0,
        color: Rgb::new(1.0, 0.9, 0.7),
        intensity: 180.0,
    })
    .with_light(Light::Ambient {
        color: Rgb::new(0.5, 0.6, 0.8),
        intensity: 0.12,
    })
    // Chão: malha de verdade, com textura de cor e mapa de normais.
    .with_object(Object::new(
        Primitive::Mesh(chao),
        Material {
            albedo: Value::Map(xadrez(64, Rgb::gray(0.72), Rgb::new(0.18, 0.2, 0.26))?),
            roughness: Value::Constant(0.85),
            normal: Some(ondulado(64)?),
            ..Material::default()
        },
    ))
    // Esfera analítica de metal: tudo constante, tem que voltar idêntica.
    .with_object(
        Object::new(
            Primitive::sphere(Vec3::ZERO, 1.0),
            Material::metal(Rgb::new(0.95, 0.78, 0.35), 0.12),
        )
        .with_transform(Transform {
            translation: Vec3::new(-1.9, 1.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }),
    )
    // Cubo analítico com rotação nos três eixos e escala não uniforme: é o
    // caminho do Euler XYZ ↔ quatérnio.
    .with_object(
        Object::new(
            Primitive::cube(Vec3::ZERO, 0.8),
            Material {
                albedo: Value::Map(xadrez(
                    32,
                    Rgb::new(0.8, 0.25, 0.2),
                    Rgb::new(0.95, 0.9, 0.8),
                )?),
                roughness: Value::Map(gradiente(32)?),
                ..Material::default()
            },
        )
        .with_transform(Transform {
            translation: Vec3::new(1.4, 0.8, 0.6),
            rotation: Vec3::new(18.0, 35.0, 12.0),
            scale: Vec3::new(1.0, 1.3, 1.0),
        }),
    )
    // Vidro, com transmissão e índice de refração.
    .with_object(
        Object::new(
            Primitive::sphere(Vec3::ZERO, 0.7),
            Material {
                albedo: Value::Constant(Rgb::gray(0.06)),
                roughness: Value::Constant(0.03),
                transparency: 0.95,
                refractive_index: 1.52,
                ..Material::default()
            },
        )
        .with_transform(Transform {
            translation: Vec3::new(0.0, 0.7, 2.4),
            ..Transform::IDENTITY
        }),
    )
    // Emissivo acima de 1, que o `emissiveFactor` sozinho cortaria, e sem
    // sombra própria.
    .with_object(
        Object::new(
            Primitive::sphere(Vec3::ZERO, 0.28),
            Material::emissive(Rgb::new(2.6, 1.7, 0.8)),
        )
        .with_transform(Transform {
            translation: Vec3::new(2.6, 2.4, -1.2),
            ..Transform::IDENTITY
        })
        .with_shadows(false),
    )
    // Triângulo solto, pra checar que ele não volta como malha.
    .with_object(Object::new(
        Primitive::Triangle {
            points: [
                Vec3::new(-4.2, 0.02, -1.0),
                Vec3::new(-2.6, 0.02, -2.2),
                Vec3::new(-3.4, 2.4, -1.6),
            ],
            normals: None,
            tangents: None,
            uvs: Some([
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.5, 0.0),
            ]),
        },
        Material::diffuse(Rgb::new(0.35, 0.7, 0.45)),
    ));

    Ok(cena)
}

fn xadrez(lado: u32, clara: Rgb, escura: Rgb) -> Result<Texture, TextureError> {
    let texels = (0..lado * lado)
        .map(|i| {
            let (x, y) = (i % lado, i / lado);
            if (x / 8 + y / 8) % 2 == 0 {
                clara
            } else {
                escura
            }
        })
        .collect();

    Texture::new(lado, lado, texels)
}

/// Roughness variando na diagonal: um `Value<f32>` que é mapa, ao lado de um
/// `metallic` que é constante — o caso que o empacotamento em uma textura só
/// tem que desfazer.
fn gradiente(lado: u32) -> Result<Texture, TextureError> {
    let texels = (0..lado * lado)
        .map(|i| {
            let (x, y) = (i % lado, i / lado);
            Rgb::gray((x + y) as f32 / (2 * lado) as f32)
        })
        .collect();

    Texture::new(lado, lado, texels)
}

/// Mapa de normais de uma superfície ondulada, em espaço tangente: o `z` fica
/// perto de 1 e o `x`/`y` oscilam em torno de 0,5, que é o zero codificado.
fn ondulado(lado: u32) -> Result<Texture, TextureError> {
    let texels = (0..lado * lado)
        .map(|i| {
            let (x, y) = (i % lado, i / lado);
            let (u, v) = (x as f32 / lado as f32, y as f32 / lado as f32);

            let n = Vec3::new(
                0.25 * (u * TAU * 4.0).sin(),
                0.25 * (v * TAU * 4.0).sin(),
                1.0,
            )
            .normalized();

            Rgb::new(n.x * 0.5 + 0.5, n.y * 0.5 + 0.5, n.z * 0.5 + 0.5)
        })
        .collect();

    Texture::new(lado, lado, texels)
}
