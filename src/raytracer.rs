//! Ray tracer — esqueleto de M0.5.
//!
//! Lança um raio primário por pixel, testa contra as esferas da cena e pinta o albedo
//! do material como cor sólida.
//!
//! `Triangle`/`Mesh` são ignorados por enquanto.

use crate::color::Rgb;
use crate::math::{Ray, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer};
use crate::scene::{Camera, Primitive, Scene};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RayTracer;

impl Renderer for RayTracer {
    fn name(&self) -> &'static str {
        "ray tracer"
    }

    fn render(&self, scene: &Scene, params: &Parameters, passes: &[RenderPass]) -> RenderResult {
        let mut result = RenderResult::new();
        if !passes.contains(&RenderPass::Color) {
            return result;
        }

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

        result.stats = stats;
        result.insert(RenderPass::Color, cor);
        result
    }
}

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

    // converte a coordenada do pixel pra um valor contínuo (pixel 1,1 tem centro em 1.5,1.5)
    let x = x as f32 + 0.5;
    let y = y as f32 + 0.5;

    // normaliza as coordenadas pra ndc ([-1,1] de largura e de altura).

    // Exemplo com 320x240:
    // operação	| faixa depois dela
    // x           0..320
    // / largura   0..1
    // * 2         0..2
    // -1         -1..1
    let u = 2.0 * x / largura as f32 - 1.0;

    // y cresce pra pra baixo no buffer e pra cima em NDC, então pegamos o complemento de 1
    // números baixos viram alto e vice-versa:
    //
    // y           0..240
    // / altura    0..1
    // * 2         0..2
    // 1 - y       -1..1 (invertido)

    // y = 2 (base da imagem) -> 1-2 = -1
    // y = 1 (centro)         -> 1-1 =  0
    // y = 0 (topo da imagem) -> 1-0 =  1
    let v = 1.0 - 2.0 * y / altura as f32;

    match *camera {
        Camera::Perspective {
            fov,
            aspect,
            position,
            ..
        } => {
            // Vamos imaginar uma tela virtual a 1 unidade pra frente da câmera, e o raio saindo da câmera e passando pelo meio de cada pixel

            // Divide o fov por 2 pra descobrir o ângulo entre o topo e a origem (ou a base e a origem, mesmo
            // valor). E depois pega a tangente desse ângulo, que diz *quanto a altura aumenta a cada unidade que
            // a gente avança em z*, já que tan() = oposto/adjacente

            //                         ^  topo da tela
            //                       / |
            //                     /   |
            //       fov/2       /     |  meia_altura = tan(fov/2) * 1 (distância da câmera até a tela virtual)
            // câmera  ) - - - - - - - +  centro da tela
            //           \       \     |
            //             \       \   |
            //               \       \ |
            //                         v  base da tela
            //       |<---- 1.0 ------>|
            let meia_altura = (fov.to_radians() * 0.5).tan();

            let deslocamento_pra_frente = frente;

            // pensando em 2d:
            // u e v variam entre -1 e 1 conforme raio_primario() é chamado pra cada pixel da imagem
            // a gente só multiplica por meia_altura, que daí a coordenada do pixel com y = 1 no plano virtual = 1 * meia_altura =
            // meia_altura, e y = -1 no plano virtual é igual a -meia_altura (e os valores intermediários interpolados linearmente)

            // daí é só multiplicar o vetor 'cima' por esse valor pra descobrir o deslocamento necessário no eixo 'cima'
            let deslocamento_pra_cima = cima * (v * meia_altura);

            // aspect = width / height -> width = aspect * height
            // tan(fov_h / 2) = W / z                     (definição do fov horizontal)
            //                = (H × aspect) / z          (porque W = H × aspect)
            //                = (H / z) × aspect          (só reagrupando a multiplicação)
            //                = tan(fov_v / 2) × aspect   (porque H/z é o tan vertical)
            let deslocamento_pra_direita = direita * (u * meia_altura * aspect);

            // somamos os deslocamentos nos 3 eixos da câmera, pra formar um único vetor apontando pro pixel
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
            // Na projeção ortográfica, a origem varia e a direção dos raios primários é a mesma em todos os pixels.
            // pra cada pixel, a gente calcula a origem alinhada com o pixel, e a direção é o vetor de frente da câmera pra todos.

            // u e v vão de -1..1 (tamanho = 2, 1 unidade pra cada lado nos eixos x e y), mas o volume de visão é largura x altura
            // (largura/2 pra cada lado no eixo x e altura/2 pra cada lado no eixo y), então multiplicamos u e v por altura/2 e largura/2
            // pra converter a coordenada normalizada no deslocamento necessário em cada eixo.

            // Exemplo com width = 320:
            //  u  | u * width * 0.5
            // −1   −160 (borda esquerda)
            //  0      0 (centro)
            // +1   +160 (borda direita)

            // Mas esse valor é a coordenada 2d do pixel em camera space, então precisamos multiplicar pelos vetores direita e cima, que
            // que guardam a informação de orientação
            let origem = position + direita * (u * width * 0.5) + cima * (v * height * 0.5);
            Ray::new(origem, frente)
        }
    }
}

fn intersecao_esfera(r: Ray, centro: Vec3, raio: f32) -> Option<f32> {
    // A esfera é o conjunto de pontos P com |P − centro| = raio (ou |P − centro|² = raio²). Ou seja, distância até o centro = raio
    // Para encontrar o ponto P' na distância t ao longo do raio a gente usa P' = origem + direção * t (Ray::at())

    // Então substituindo P com a fórmula de P' na equação da esfera:
    //   |(origem + t*direção) − centro|² = raio²
    // > |(origem − centro) + t*direção|² = raio²

    // Identidades importantes:
    // Fórmula geométrica do produto escalar (u · v | u.dot(v)):
    // u · v = |u| × |v| × cos(θ), onde θ é o ângulo entre u e v

    // Quando tiramos o dot product de um vetor com ele mesmo isso se simplifica:
    // Como o ângulo entre um vetor e ele mesmo é 0°, e cos(0°) = 1, temos v · v = |v| × |v| × 1 = |v|²

    // Ou seja, v.dot(v) = comprimento de v ao quadrado, que a gente pode usar pra simplificar a fórmula acima.

    // oc = origem − centro
    //   |oc + t*direção|² = raio²
    // > (oc + t*direção) · (oc + t*direção) = raio²

    // Mais propriedades do dot product:
    // P1 distributiva              -> u · (v + w)  = u·v + u·w
    // P2 comutativa                -> u · v        = v · u
    // P3 escalar (k) sai para fora -> u · (k·v)    = k (u·v)

    // Agora vamos simplificar mais o lado esquerdo ((oc + t·D) · (oc + t·D))
    // Com:
    // A = oc
    // B = t·D

    // Temos:
    // (A + B)·(A + B) = A·A + A·B + B·A + B·B

    // 1)  A·A  =  oc · oc
    // 2)  A·B  =  oc · (t·D)
    // 3)  B·A  =  (t·D) · oc
    // 4)  B·B  =  (t·D) · (t·D)

    // 2 e 3 são iguais, devido a P2. Também podem ser reescritas devido à P3: oc · (t·D) = t (oc · D)

    // Então temos:
    //   (oc + t*direção) · (oc + t*direção) = raio²
    // > oc · oc + 2t(oc · direção) + t² (direção · direção) = raio²
    //   1)        2) + 3)            4)
    // > oc · oc + 2t(oc · direção) + t² (direção · direção) - raio² = 0
    // > t² (direção · direção) + 2t(oc · direção) + (oc · oc - raio²) = 0

    // Agora organizando os termos na forma at² + bt + c = 0, pra resolver com Bhaskara:
    // a = (direção · direção) = 1 (já que o compimento é 1 e v.dot(v) = |v|²)
    // b = 2(oc · direção)
    // c = oc · oc - raio²

    let oc = r.origin - centro;

    let a = 1.0;
    let b = 2.0 * oc.dot(r.direction);
    let c = oc.dot(oc) - raio * raio;

    // E daí é Bhaskara normal (finalmente): t = (−b ± √(b²−4ac)) / 2a

    let discriminante = b * b - 4.0 * a * c;
    if discriminante < 0.0 {
        // Não existe ponto P onde o raio intercepta a esfera
        return None;
    }

    let raiz = discriminante.sqrt();
    let t0 = (-b - raiz) / (2.0 * a);
    let t1 = (-b + raiz) / (2.0 * a);

    // como raiz precisa ser > 0, t1 <= t2
    // o raio intercepta a superfície mais próxima, então pegamos o menor deles
    // filtro de > 0 pra não escolher um ponto atrás da câmera
    [t0, t1].into_iter().find(|&t| t > 0.0)
}
