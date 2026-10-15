//! Renderiza **a mesma cena** nos dois modos e grava um .png de cada passe que
//! o renderer produziu.
//!
//! A cena tem um chão de xadrez, um triângulo, um cubo, uma esfera difusa, uma
//! de metal polido, uma de vidro e uma emissiva. O rasterizador tessela tudo em
//! triângulos e desenha; o ray tracer resolve a esfera pela equação e o resto
//! pelas faces — e os dois passam pelo mesmo `shade()`.
//!
//! O metal e o vidro existem para o que só o ray tracer faz: um reflete a cena
//! e o outro a refrata, os dois por recursão do modelo de Whitted.
//!
//! A esfera emissiva é a lâmpada, e mostra o que a emissão faz e o que ela não
//! faz: a esfera aparece acesa nos dois modos e no reflexo do metal, mas quem
//! ilumina a cena é a `Light::Point` dentro dela.
//!
//! No fim, o mesmo render do ray tracer com a BVH pelo SAH, pela mediana e
//! desligada: a imagem é igual, e o que muda é o trabalho.
//!
//! ```text
//! cargo run --example dois_modos
//! ```

mod comum;

use std::error::Error;
use std::time::Instant;

use tcc2::prelude::*;

/// A esfera emissiva e a luz pontual dela ficam no mesmo ponto.
const LAMPADA: Vec3 = Vec3::new(-1.3, 1.6, 1.6);

fn main() -> Result<(), Box<dyn Error>> {
    let scene = Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 60.0,
                    aspect: 4.0 / 3.0,
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
                Light::Ambient {
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
                    translation: LAMPADA,
                    ..Transform::IDENTITY
                },
            ),
            Object::geometry(
                chao(),
                Material::textured(xadrez(512, 8)),
                Transform::IDENTITY,
            ),
            Object::geometry(
                Geometry::triangle([
                    Vec3::new(-3.4, -1.0, 0.0),
                    Vec3::new(-1.4, -1.0, 0.0),
                    Vec3::new(-2.4, 1.0, 0.0),
                ]),
                Material::diffuse(Rgb::new(0.9, 0.3, 0.2)),
                Transform::IDENTITY,
            ),
            Object::geometry(
                Geometry::cube(0.8),
                Material {
                    albedo: Value::Constant(Rgb::new(0.3, 0.8, 0.4)),
                    roughness: Value::Constant(0.35),
                    ..Material::default()
                },
                Transform {
                    rotation: Vec3::new(25.0, 35.0, 0.0),
                    ..Transform::IDENTITY
                },
            ),
            Object::geometry(
                Geometry::sphere(1.0),
                Material {
                    albedo: Value::Constant(Rgb::new(0.2, 0.4, 0.9)),
                    roughness: Value::Constant(0.3),
                    ..Material::default()
                },
                Transform {
                    translation: Vec3::new(2.4, 0.0, 0.0),
                    scale: Vec3::new(1.0, 0.7, 1.0),
                    ..Transform::IDENTITY
                },
            ),
            Object::geometry(
                Geometry::sphere(0.75),
                Material::metal(Rgb::new(0.95, 0.93, 0.88), 0.04),
                Transform {
                    translation: Vec3::new(-2.0, -0.45, 1.6),
                    ..Transform::IDENTITY
                },
            ),
            Object {
                transform: Transform {
                    translation: LAMPADA,
                    ..Transform::IDENTITY
                },
                content: Some(Content::Geometry {
                    geometry: Geometry::sphere(0.35),
                    material: Material::emissive(Rgb::new(1.4, 1.0, 0.55)),
                    casts_shadows: false,
                }),
                children: Vec::new(),
            },
            Object::geometry(
                Geometry::sphere(0.7),
                Material {
                    albedo: Value::Constant(Rgb::gray(0.02)),
                    roughness: Value::Constant(0.03),
                    transparency: 0.9,
                    refractive_index: 1.5,
                    ..Material::default()
                },
                Transform {
                    translation: Vec3::new(1.0, -0.5, 2.4),
                    ..Transform::IDENTITY
                },
            ),
        ],
    };

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

        comum::grava_textura(
            &format!("./renders/shading_{nome}.png"),
            &resultado.color,
            Rgb::to_srgb,
        )?;
    }

    // Os toggles de caixa branca do modelo de Whitted: cada render abaixo
    // desliga um termo da recursão.
    for (opcoes, nome) in [
        (
            RayTracingOptions {
                shadows: false,
                ..params.ray_tracing
            },
            "sem_sombra",
        ),
        (
            RayTracingOptions {
                reflections: false,
                ..params.ray_tracing
            },
            "sem_reflexo",
        ),
        (
            RayTracingOptions {
                refractions: false,
                ..params.ray_tracing
            },
            "sem_refracao",
        ),
        (
            RayTracingOptions {
                max_depth: 0,
                ..params.ray_tracing
            },
            "sem_recursao",
        ),
    ] {
        let params = Parameters {
            ray_tracing: opcoes,
            ..params.clone()
        };

        let resultado = RayTracer.render(&scene, &params);

        comum::grava_textura(
            &format!("./renders/whitted_{nome}.png"),
            &resultado.color,
            Rgb::to_srgb,
        )?;
    }

    for (acceleration, nome) in [
        (AccelerationOptions::default(), "bvh pelo sah"),
        (
            AccelerationOptions {
                split: BvhSplit::Median,
                ..AccelerationOptions::default()
            },
            "bvh pela mediana",
        ),
        (
            AccelerationOptions {
                bvh: false,
                ..AccelerationOptions::default()
            },
            "sem bvh",
        ),
    ] {
        let params = Parameters {
            ray_tracing: RayTracingOptions {
                acceleration,
                ..params.ray_tracing
            },
            ..params.clone()
        };

        let inicio = Instant::now();
        let stats = RayTracer.render(&scene, &params).ray_tracing;

        println!(
            "{nome}: {:.1?}, {} testes de primitivo, {} caixas testadas, profundidade {} na tlas e {} na blas mais funda, construída em {:.1?}",
            inicio.elapsed(),
            stats.primitive_tests,
            stats.bvh_nodes_visited,
            stats.tlas_depth,
            stats.blas_depth,
            stats.bvh_build_time,
        );
    }

    for (split, nome) in [(BvhSplit::Sah, "sah"), (BvhSplit::Median, "mediana")] {
        let params = Parameters {
            ray_tracing: RayTracingOptions {
                acceleration: AccelerationOptions {
                    split,
                    wireframe: Some(6),
                    ..AccelerationOptions::default()
                },
                ..params.ray_tracing
            },
            ..params.clone()
        };

        let resultado = RayTracer.render(&scene, &params);

        comum::grava_textura(
            &format!("./renders/bvh_{nome}_wireframe.png"),
            &resultado.color,
            Rgb::to_srgb,
        )?;
    }

    Ok(())
}

// Um quadrado no plano y = -1.2, com uv de 0 a 4 pra textura repetir quatro
// vezes em cada direção.
fn chao() -> Geometry {
    let y = -1.2;

    Geometry::Mesh(Mesh {
        vertices: vec![
            Vec3::new(-6.0, y, 4.0),
            Vec3::new(6.0, y, 4.0),
            Vec3::new(6.0, y, -8.0),
            Vec3::new(-6.0, y, -8.0),
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

    for (buffer, nome) in [
        (&resultado.color, "color"),
        (&resultado.normal, "normal"),
        (&resultado.uv, "uv"),
    ] {
        let arquivo = format!("./renders/{prefixo}_{nome}.png");
        comum::grava_textura(&arquivo, buffer, Rgb::to_srgb)?;
    }

    for (buffer, nome) in [
        (&resultado.depth, "depth"),
        (&resultado.ray_tracing.ray_count, "ray_count"),
        (&resultado.ray_tracing.bvh_heatmap, "bvh_heatmap"),
    ] {
        let arquivo = format!("./renders/{prefixo}_{nome}.png");
        comum::grava_textura(&arquivo, &buffer.to_color(), Rgb::to_srgb)?;
    }

    let (largura, altura) = (resultado.color.width(), resultado.color.height());

    println!(
        "{}: {largura}x{altura}, {} raios, {} backfaces descartadas",
        prefixo, resultado.ray_tracing.rays, resultado.rasterization.backfaces_culled,
    );

    Ok(())
}
