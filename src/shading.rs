//! Shading: a cor de uma superfície sob as luzes da cena.
//!
//! O rasterizador chega aqui com a normal interpolada do triângulo, o ray tracer
//! com a normal analítica da esfera, e daí pra frente a conta é a mesma.
//!
//! `shade` itera as luzes e resolve atenuação e visibilidade; o modelo de
//! reflectância em si fica numa função por variante do `ReflectanceModel` —
//! hoje só `blinn_phong`. É por ali que uma BRDF microfacetada entra depois,
//! sem tocar no laço das luzes.
//!
//! Teoria: raytracer.md#o-termo-direto-de-whitted
//!
//! Sob o modelo de Whitted este módulo é o **termo direto** da fórmula (mais a
//! emissão do material, que não depende de luz nenhuma), e só ele: a recursão
//! de reflexão e refração é do ray tracer. O que entra aqui do
//! modelo é o fator de visibilidade `V`, que cada pipeline resolve do seu jeito
//! — raio de sombra no ray tracer, mapa de sombra (quando existir) no
//! rasterizador.

use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::parameters::ReflectanceModel;
use crate::scene::{Light, MaterialSample};

/// O ponto que recebe o shading. Todos os vetores em espaço de mundo, e as
/// normais unitárias.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

/// A cor que sai da superfície na direção `view`.
///
/// `view` aponta do ponto **para** quem olha: a câmera no acerto primário, e o
/// espelho no que o ray tracer vê dentro de um reflexo.
///
/// `visibility` devolve, por luz, a fração dela que chega ao ponto — 1 sem
/// sombra nenhuma, 0 totalmente na sombra. É o `V` de Whitted, e é o parâmetro
/// que os dois pipelines preenchem de formas diferentes.
pub fn shade(
    surface: &Surface,
    material: &MaterialSample,
    lights: &[Light],
    view: Vec3,
    reflectance: ReflectanceModel,
    visibility: &dyn Fn(&Surface, &Light) -> f32,
) -> Rgb {
    let n = surface.normal;
    let v = view;

    let mut cor = material.emission;

    for luz in lights {
        if let Light::Ambient { color, intensity } = *luz {
            // A ambiente não tem direção. É o termo que impede a sombra de fechar em preto chapado enquanto não há iluminação indireta.
            cor = cor + material.albedo * color * intensity;
            continue;
        }

        let (l, radiancia) = incidencia(luz, surface.point);

        // Luz que chega por trás da superfície não ilumina, e não vale testar a visibilidade nesse caso. O mesmo vale pra luz que não
        // chega (ponto fora do cone de um spot).
        if n.dot(l) <= 0.0 || radiancia == Rgb::BLACK {
            continue;
        }

        let visivel = visibility(surface, luz);
        if visivel <= 0.0 {
            continue;
        }

        // A escolha da BRDF. Cada modelo devolve a fração da luz que chega
        // por `l` e sai por `v`.
        let refletida = match reflectance {
            ReflectanceModel::BlinnPhong => blinn_phong(material, n, v, l),
        };

        cor = cor + refletida * radiancia * visivel;
    }

    cor
}

/// A visibilidade de quem não tem sombra: toda luz chega inteira.
pub fn always_visible(_surface: &Surface, _light: &Light) -> f32 {
    1.0
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

            // Teoria: shading.md#a-atenuação-com-a-distância
            // intensity já vem em watts / 4*PI, então watts por feixe = intensity / d2.
            (ate_a_luz.normalized(), color * (intensity / distancia2))
        }
        Light::Directional {
            direction,
            color,
            intensity,
        } => ((-direction).normalized(), color * intensity),
        Light::Spot {
            position,
            direction,
            inner_angle,
            outer_angle,
            color,
            intensity,
        } => {
            // Teoria: shading.md#o-cone-do-spot
            // Um spot é uma luz pontual com um cone por cima: mesma direção, mesma
            // atenuação por 1/d^2, e um fator a mais que depende do ângulo.
            let (l, radiancia) = incidencia(
                &Light::Point {
                    position,
                    color,
                    intensity,
                },
                point,
            );

            // cos do ângulo entre o eixo do spot e o raio que sai da luz até o
            // ponto.
            let cos_ponto = direction.normalized().dot(-l);

            // Comparar cossenos evita um acos por ponto; o cosseno
            // diminui quando o ângulo cresce, então cos_interno > cos_externo.
            let cos_interno = inner_angle.to_radians().cos();
            let cos_externo = outer_angle.to_radians().cos();

            // 0 fora do cone externo, 1 dentro do interno, rampa entre os dois.
            let t =
                ((cos_ponto - cos_externo) / (cos_interno - cos_externo).max(1e-4)).clamp(0.0, 1.0);

            // o t^2 suaviza a borda.
            (l, radiancia * (t * t))
        }
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
    // Teoria: shading.md#a-lei-do-cosseno-de-lambert
    // Como n e l são unitários, n.dot(l) devolve o cos entre eles.
    let cos_theta = n.dot(l);
    if cos_theta <= 0.0 {
        // Luz atrás da superfície não ilumina nem gera brilho.
        return Rgb::BLACK;
    }

    // Teoria: shading.md#de-roughness-para-shininess
    let alpha = m.roughness * m.roughness;
    let shininess = (2.0 / (alpha * alpha) - 2.0).max(1.0);

    // Teoria: raytracer.md#o-peso-do-raio-de-espelho
    // Um Fresnel só divide o difuso e o especular, do mesmo jeito que ele
    // divide os dois raios secundários no ray tracer: o que não volta como
    // espelho é o que entrou na superfície e sai espalhado. O `abs` cobre o
    // acerto pela face de dentro, igual ao de lá.
    let fresnel = fresnel_schlick(f0(m), n.dot(v).abs());

    // Teoria: shading.md#kd-e-ks-a-partir-de-metallic
    let kd = (Rgb::WHITE - fresnel)
        * ((1.0 - m.metallic) * (1.0 - m.transparency) / std::f32::consts::PI);
    // Teoria: shading.md#a-normalização-do-especular
    let ks = fresnel * ((shininess + 8.0) / (8.0 * std::f32::consts::PI));

    let difuso = m.albedo * kd * cos_theta;

    // h é o vetor do meio do caminho entre a luz e o olho: quando ele coincide com a normal, o reflexo especular aponta exatamente
    // para a câmera. É a troca do Blinn pelo `r` refletido do Phong de 1975.
    let h = (v + l).normalized();
    let ndoth = n.dot(h).max(0.0);

    let especular = ks * ndoth.powf(shininess);

    difuso + especular
}

// Teoria: raytracer.md#o-peso-do-raio-de-espelho
/// A refletância de frente (`f0`): quanto da luz um material devolve como
/// espelho quando ela chega perpendicular à superfície.
pub fn f0(material: &MaterialSample) -> Rgb {
    // f0 = ((n1 - n2) / (n1 + n2))², com n1 = 1 no ar.
    let n = material.refractive_index;
    let dieletrico = ((1.0 - n) / (1.0 + n)).powi(2);

    Rgb::gray(dieletrico).lerp(material.albedo, material.metallic)
}

/// Schlick (1994): a refletância cresce de `f0` até 1 conforme a luz passa a
/// raspar a superfície.
pub fn fresnel_schlick(f0: Rgb, cos: f32) -> Rgb {
    let cos = cos.clamp(0.0, 1.0);
    f0 + (Rgb::WHITE - f0) * (1.0 - cos).powi(5)
}
