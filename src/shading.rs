//! Shading: a cor de uma superfície sob as luzes da cena.
//!
//! O rasterizador chega aqui com a normal interpolada do triângulo, o ray tracer
//! com a normal analítica da esfera, e daí pra frente a conta é a mesma.
//!
//! `shade` itera as luzes e resolve atenuação e visibilidade; o modelo de
//! reflectância em si fica numa função por variante do `ReflectanceModel` —
//! hoje só `blinn_phong`. É por ali que uma BRDF microfacetada entra depois,
//! sem tocar no laço das luzes.

use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::parameters::ReflectanceModel;
use crate::scene::{Camera, Light, Material, MaterialSample};

/// O ponto que recebe o shading. Todos os vetores em espaço de mundo, e a
/// normal unitária.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

/// A cor que sai da superfície na direção da câmera.
pub fn shade(
    surface: &Surface,
    material: &Material,
    lights: &[Light],
    camera: &Camera,
    reflectance: ReflectanceModel,
) -> Rgb {
    let amostra = material.sample(surface.uv);

    let n = surface.normal;
    let v = camera.view_from(surface.point);

    let mut cor = Rgb::BLACK;

    for luz in lights {
        if let Light::Ambient { color, intensity } = *luz {
            // A ambiente não tem direção. É o termo que impede a sombra de fechar em preto chapado enquanto não há iluminação indireta.
            cor = cor + amostra.albedo * color * intensity;
            continue;
        }

        let (l, radiancia) = incidencia(luz, surface.point);

        // A escolha da BRDF. Cada modelo devolve a fração da luz que chega
        // por `l` e sai por `v`.
        let refletida = match reflectance {
            ReflectanceModel::BlinnPhong => blinn_phong(&amostra, n, v, l),
        };

        cor = cor + refletida * radiancia;
    }

    cor
}

/// Direção **para** a luz e a radiância que chega ao ponto.
fn incidencia(light: &Light, point: Vec3) -> (Vec3, Rgb) {
    match *light {
        Light::Point {
            position,
            color,
            intensity,
        } => {
            let ate_a_luz = position - point;

            // v · v = |v|^2, então d^2 = ate_a_luz.dot(ate_a_luz).
            let distancia2 = ate_a_luz.dot(ate_a_luz).max(1e-6);

            // A energia da luz se espalha na casca de uma esfera de raio d.
            // A área dessa casca é 4*PI*d2, então aumentar d aumenta a área e diminui a quantidade de energia que chega a cada ponto da
            // esfera.
            // Então a quantidade de watts por ponto da esfera (feixe de luz) é watts / 4*PI*d2.
            // Intensity já foi informada em watts / 4*PI (o quanto a luz ilumina a um metro de distância, quando d = 1 e watts / 4*PI*d2 = watts / 4*PI),
            // então watts por feixe = watts / 4*PI*d2 = intensity / d2
            (ate_a_luz.normalized(), color * (intensity / distancia2))
        }
        Light::Directional {
            direction,
            color,
            intensity,
        } => ((-direction).normalized(), color * intensity),
        Light::Ambient { .. } => unreachable!(),
    }
}

/// Blinn-Phong como ele aparece na literatura: dada uma unidade de luz chegando
/// pela direção `l`, que fração volta na direção `v`. Um termo difuso mais um
/// especular:
///
/// ```text
/// kd · albedo · (n · l)  +  ks · (n · h)^shininess
/// ```
fn blinn_phong(m: &MaterialSample, n: Vec3, v: Vec3, l: Vec3) -> Rgb {
    // Lei do cosseno de Lambert: a luz se espalha por uma área cada vez maior quanto mais inclinada ela chega na superfície;
    // então a quantidade de watts por unidade de área é inversamente proporcional à inclinação:
    //
    // angulo θ=0                    angulo θ grande
    //   | | | |                         \ \ \ \
    //   | | | |                         _\_\_\_\_
    //   ━━━━━━━━━                     ━━━━━━━━━━━━━━━━━━
    // 4 feixes em 4 unidades        4 feixes em 8 unidades
    // de superfície                 de superfície
    //
    // Pegando um feixe de luz:
    //
    //         \       \
    //          \       \
    //           \       C <- ângulo = 90º
    //            \   w / \  n
    //             \   /   \ |
    //              \ /     \|
    // ────────────────A───────B──────
    //               └── s ──┘
    //
    // w = |AC| é a distância entre as duas bordas medida perpendicularmente a elas
    // s = |AB| é o comprimento do feixe na superfície
    // Na ascii art não aparece mas o ângulo ACB = 90º: ABC é um triângulo retângulo
    // O ângulo em B = 90º - (ângulo entre l e n calculado abaixo) = θ
    //
    // No triângulo retângulo, cateto oposto = hipotenusa × seno do ângulo, então
    // w = s · sen(90° − θ) = s · cos(θ) -> s = w / cos θ
    //
    // Pra todo w, a área que o feixe cobre na superfície é w * (1/cos(θ)).
    // Watts por unidade de área = watts * w/s
    //                           = watts * w * 1/s
    //                           = watts * w * 1/(w * (1/cos(θ)))
    //                           = watts * w * (1/w) * cos(θ)
    //                           = watts * (w/w) * cos(θ)
    //                           = watts * cos(θ)

    // Como n e l são unitários, n.dot(l) devolve o cos entre eles.
    let cos_theta = n.dot(l);
    if cos_theta <= 0.0 {
        // Luz atrás da superfície não ilumina nem gera brilho.
        return Rgb::BLACK;
    }

    // kd e ks são, na literatura, dois parâmetros do material. Aqui eles saem do `metallic`: metal não devolve luz difusa,
    // e o brilho dele é da cor do próprio metal (ouro brilha amarelo) enquanto o de um dielétrico é branco (plástico vermelho
    // brilha branco).
    let kd = 1.0 - m.metallic;
    let ks = Rgb::gray(0.5).lerp(m.albedo, m.metallic);

    let difuso = m.albedo * (kd * cos_theta);

    // h é o vetor do meio do caminho entre a luz e o olho: quando ele coincide com a normal, o reflexo especular aponta exatamente
    // para a câmera. É a troca do Blinn pelo `r` refletido do Phong de 1975.
    let h = (v + l).normalized();
    let ndoth = n.dot(h).max(0.0);

    // Transformar o parâmetro roughness de PBR pra shininess de Blinn-Phong
    // Primeiro α = r² (Disney, SIGGRAPH 2012), não é exata, mas essa fórmula foi encontrada empiricamente e dá bons resultados
    // Depois n = 2/α² − 2 (Walter et al. 2007)
    //
    // [TODO: voltar aqui depois de adicionar Cook-Torrance pra provar isso]
    let alpha = m.roughness * m.roughness;
    let shininess = (2.0 / (alpha * alpha) - 2.0).max(1.0);

    let especular = ks * ndoth.powf(shininess);

    difuso + especular
}
