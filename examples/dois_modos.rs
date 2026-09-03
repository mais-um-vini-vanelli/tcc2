//! Renderiza **a mesma cena** nos dois modos e grava um .ppm de cada.
//!
//! A cena tem um triângulo, um cubo (`Mesh`) e uma esfera; por enquanto o
//! rasterizador desenha os dois primeiros e o ray tracer só a esfera.
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
    .with_object(cubo(
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

    if let Some(normais) = resultado.get(RenderPass::Normal) {
        write_ppm(arquivo.replace(".ppm", "_normal.ppm"), &normais.to_color())?;
    }

    println!(
        "{}: {arquivo} ({}x{}), {} backfaces culled",
        renderer.name(),
        imagem.width(),
        imagem.height(),
        resultado.stats.backfaces_culled,
    );

    Ok(())
}

/// Cubo centrado em `centro`, com aresta `2 * meia_aresta`.
///
/// A ordem dos índices de cada face é anti-horária **vista de fora**, que é o
/// que o backface culling espera: face virada pra câmera fica, face de costas
/// some.
fn cubo(centro: Vec3, meia_aresta: f32, material: Material) -> Primitive {
    let h = meia_aresta;
    let vertices = [
        Vec3::new(-h, -h, -h),
        Vec3::new(h, -h, -h),
        Vec3::new(h, h, -h),
        Vec3::new(-h, h, -h),
        Vec3::new(-h, -h, h),
        Vec3::new(h, -h, h),
        Vec3::new(h, h, h),
        Vec3::new(-h, h, h),
    ]
    .map(|v| v + centro)
    .to_vec();

    #[rustfmt::skip]
    let indices = vec![
        4, 5, 6,  4, 6, 7, // frente (+z)
        0, 3, 2,  0, 2, 1, // fundo  (-z)
        1, 2, 6,  1, 6, 5, // direita (+x)
        0, 4, 7,  0, 7, 3, // esquerda (-x)
        3, 7, 6,  3, 6, 2, // topo (+y)
        0, 1, 5,  0, 5, 4, // base (-y)
    ];

    Primitive::mesh(vertices, indices, material)
}
