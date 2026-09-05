//! Ray tracer — esqueleto de M0.5.
//!
//! Teoria: raytracer.md#a-pipeline
//!
//! Lança um raio primário por pixel, testa contra as esferas da cena e pinta o albedo
//! do material como cor sólida.
//!
//! `Triangle`/`Mesh`/`Cube` são ignorados por enquanto.

use crate::color::Rgb;
use crate::math::{Ray, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderResult, RenderStats, Renderer};
use crate::scene::{Camera, Primitive, Scene};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RayTracer;

impl Renderer for RayTracer {
    fn name(&self) -> &'static str {
        "ray tracer"
    }

    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult {
        let (largura, altura) = (params.output.width, params.output.height);
        let fundo = params.output.background.unwrap_or(Rgb::BLACK);
        let mut cor = ColorBuffer::filled(largura, altura, fundo);
        let mut stats = RenderStats::default();

        let base = scene.camera.basis();
        let (near, far) = scene.camera.clip_planes();

        for y in 0..altura {
            for x in 0..largura {
                let raio = raio_primario(&scene.camera, base, x, y, largura, altura);
                stats.rays += 1;

                // Pegar o objeto mais próximo da câmera que esse raio acertou
                let mut mais_proximo = far;
                let mut acertou = None;
                for objeto in &scene.objects {
                    let Primitive::Sphere {
                        center,
                        radius,
                        material,
                    } = objeto
                    else {
                        // Ignorando outras primitivas por enquanto
                        continue;
                    };
                    stats.primitive_tests += 1;
                    if let Some(t) = intersecao_esfera(raio, *center, *radius)
                        && t >= near
                        && t < mais_proximo
                    {
                        mais_proximo = t;
                        acertou = Some(material.albedo);
                    }
                }

                if let Some(albedo) = acertou {
                    cor.set(x, y, albedo);
                }
            }
        }

        RenderResult {
            color: Some(cor),
            stats,
            // depth/normal/uv chegam em M3; ray_count e bvh_heatmap em M4.
            ..RenderResult::new()
        }
    }
}

// Teoria: raytracer.md#da-tela-para-a-cena
/// Monta um raio que passa pelo centro do pixel `(x, y)`.
///
/// É aqui que os dois pipelines são diferentes: o rasterizador projeta a geometria
/// *para* a tela, o ray tracer parte da tela *para* a cena.
fn raio_primario(
    camera: &Camera,
    base: (Vec3, Vec3, Vec3),
    x: u32,
    y: u32,
    largura: u32,
    altura: u32,
) -> Ray {
    let (direita, cima, frente) = base;

    // Centro do pixel, não o canto.
    let x = x as f32 + 0.5;
    let y = y as f32 + 0.5;

    let u = 2.0 * x / largura as f32 - 1.0;
    let v = 1.0 - 2.0 * y / altura as f32;

    match *camera {
        Camera::Perspective {
            fov,
            aspect,
            position,
            ..
        } => {
            let meia_altura = (fov.to_radians() * 0.5).tan();

            let deslocamento_pra_frente = frente;
            let deslocamento_pra_cima = cima * (v * meia_altura);
            let deslocamento_pra_direita = direita * (u * meia_altura * aspect);

            let direcao =
                deslocamento_pra_frente + deslocamento_pra_direita + deslocamento_pra_cima;

            Ray::new(position, direcao)
        }
        Camera::Orthographic {
            width,
            height,
            position,
            ..
        } => {
            // Aqui a origem é que varia por pixel; a direção é a mesma pra todos.
            let origem = position + direita * (u * width * 0.5) + cima * (v * height * 0.5);
            Ray::new(origem, frente)
        }
    }
}

// Teoria: raytracer.md#o-raio-e-a-esfera
fn intersecao_esfera(r: Ray, centro: Vec3, raio: f32) -> Option<f32> {
    let oc = r.origin - centro;

    let a = 1.0;
    let b = 2.0 * oc.dot(r.direction);
    let c = oc.dot(oc) - raio * raio;

    let discriminante = b * b - 4.0 * a * c;
    if discriminante < 0.0 {
        return None;
    }

    let raiz = discriminante.sqrt();
    let t0 = (-b - raiz) / (2.0 * a);
    let t1 = (-b + raiz) / (2.0 * a);

    // t0 <= t1, e o filtro > 0 descarta acerto atrás da câmera.
    [t0, t1].into_iter().find(|&t| t > 0.0)
}
