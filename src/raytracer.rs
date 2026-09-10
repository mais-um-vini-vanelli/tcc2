//! Ray tracer.
//!
//! Teoria: raytracer.md#a-pipeline
//!
//! Lança um raio primário por pixel, leva o raio pro espaço local de cada
//! objeto, chama `Primitive::intersect` e resolve o acerto mais próximo com o
//! mesmo `shade()` do rasterizador.
//!
//! O que este arquivo tem e o rasterizador não é a **recursão**: o modelo de
//! Whitted (1979) soma ao termo direto o que chega pelo raio de reflexão e pelo
//! de refração, e cada um deles é um `trace()` novo a partir do ponto de
//! acerto. `shading::shade` continua sendo só o termo direto.
//!
//! Os testes de interseção são por força bruta até M3.

use std::cell::Cell;

use crate::color::Rgb;
use crate::math::{Ray, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderResult, RenderStats, Renderer};
use crate::scene::{Camera, Light, Object, Scene};
use crate::shading::{self, Surface};

// Teoria: raytracer.md#o-acne-e-o-epsilon
/// Deslocamento da origem de todo raio secundário, para que a superfície que o
/// gerou não apareça como acerto em `t` quase zero.
const EPSILON: f32 = 1e-3;

// Teoria: raytracer.md#o-limite-do-espelho-de-whitted
/// Acima desta rugosidade o lóbulo especular é largo demais para um raio só, e
/// o raio de espelho deixa de ser disparado.
const ESPELHO_MAX_ROUGHNESS: f32 = 0.1;

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

        let contador = Contador::default();

        let base = scene.camera.basis();
        let (near, far) = scene.camera.clip_planes();

        for y in 0..altura {
            for x in 0..largura {
                let raio = raio_primario(&scene.camera, base, x, y, largura, altura);
                contador.raio();

                cor.set(x, y, trace(scene, params, raio, near, far, 0, &contador));
            }
        }

        RenderResult {
            color: Some(cor),
            stats: contador.0.get(),
            // depth/normal/uv chegam em M5; ray_count e bvh_heatmap em M3.
            ..RenderResult::new()
        }
    }
}

/// Contadores do render.
///
/// `Cell` porque o callback de visibilidade que `shade` recebe é `Fn`, não
/// `FnMut`, e mesmo assim precisa contabilizar o raio de sombra que dispara.
/// `RenderStats` é `Copy`, então a célula sai barata.
#[derive(Debug, Default)]
struct Contador(Cell<RenderStats>);

impl Contador {
    fn raio(&self) {
        self.atualiza(|s| s.rays += 1);
    }

    fn teste(&self) {
        self.atualiza(|s| s.primitive_tests += 1);
    }

    fn atualiza(&self, f: impl FnOnce(&mut RenderStats)) {
        let mut stats = self.0.get();
        f(&mut stats);
        self.0.set(stats);
    }
}

/// O objeto acertado e a superfície no ponto de acerto, já em espaço de mundo.
struct Acerto<'a> {
    objeto: &'a Object,
    surface: Surface,
}

// Teoria: raytracer.md#a-recursao-de-whitted
/// A cor que chega pela direção contrária ao raio.
///
/// ```text
/// L = Σ_luzes V · reflectancia · radiancia   <- shading::shade, o termo direto
///   + F(v, n) · L(raio refletido)            <- recursão
///   + kt      · L(raio refratado)            <- recursão
/// ```
fn trace(
    scene: &Scene,
    params: &Parameters,
    raio: Ray,
    t_min: f32,
    t_max: f32,
    profundidade: u32,
    contador: &Contador,
) -> Rgb {
    let opcoes = &params.ray_tracing;

    let Some(acerto) = mais_proximo(scene, raio, t_min, t_max, contador) else {
        return params.output.background.unwrap_or(Rgb::BLACK);
    };

    let superficie = acerto.surface;
    let material = &acerto.objeto.material;
    let amostra = material.sample(superficie.uv);

    let n = superficie.normal;
    let direcao = raio.direction.normalized();

    // A direção de visão sai do próprio raio, e não da câmera: dentro de um
    // reflexo quem "olha" para o ponto é o espelho.
    let v = -direcao;

    let direto = {
        let visibilidade = |s: &Surface, luz: &Light| {
            if !opcoes.shadows {
                return 1.0;
            }

            if ocluido(scene, s, luz, contador) {
                0.0
            } else {
                1.0
            }
        };

        shading::shade(
            &superficie,
            material,
            &scene.lights,
            v,
            params.shading.reflectance,
            &visibilidade,
        )
    };

    if profundidade >= opcoes.max_depth {
        return direto;
    }

    let mut cor = direto;

    // Um único Fresnel divide os dois raios secundários: o que ele manda pro
    // espelho é o que falta pro que entra na superfície.
    //
    // O `abs` é o acerto pela face de dentro, na saída do vidro: ali a normal
    // aponta contra quem olha, e sem ele o cosseno negativo vira F = 1 e fecha a
    // refração inteira. Teoria: raytracer.md#entrando-e-saindo
    let fresnel = shading::fresnel_schlick(shading::f0(&amostra), n.dot(v).abs());

    // Teoria: raytracer.md#o-peso-do-raio-de-espelho
    if opcoes.reflections && amostra.roughness < ESPELHO_MAX_ROUGHNESS {
        let refletida = reflete(direcao, n);

        contador.raio();

        let vindo_do_espelho = trace(
            scene,
            params,
            Ray::new(desloca(superficie.point, n, refletida), refletida),
            EPSILON,
            f32::INFINITY,
            profundidade + 1,
            contador,
        );

        cor = cor + fresnel * vindo_do_espelho;
    }

    // Teoria: raytracer.md#o-raio-refratado
    if opcoes.refractions && amostra.transparency > 0.0 {
        // Reflexão interna total: não há caminho de saída, e a energia que
        // seria transmitida volta como reflexo.
        let transmitida =
            refrata(direcao, n, material.refractive_index).unwrap_or_else(|| reflete(direcao, n));

        contador.raio();

        let vindo_de_dentro = trace(
            scene,
            params,
            Ray::new(desloca(superficie.point, n, transmitida), transmitida),
            EPSILON,
            f32::INFINITY,
            profundidade + 1,
            contador,
        );

        // O que não voltou pelo espelho é o que entrou na superfície, e
        // `transparency` diz quanto disso atravessa em vez de virar difuso.
        let peso = (Rgb::WHITE - fresnel) * amostra.transparency;

        cor = cor + vindo_de_dentro * peso;
    }

    cor
}

/// O acerto mais próximo da origem do raio, entre `t_min` e `t_max`.
fn mais_proximo<'a>(
    scene: &'a Scene,
    raio: Ray,
    t_min: f32,
    t_max: f32,
    contador: &Contador,
) -> Option<Acerto<'a>> {
    let mut limite = t_max;
    let mut acertou = None;

    for objeto in &scene.objects {
        contador.teste();

        // Teoria: transform.md#o-raio-vai-para-o-espaco-local
        // O rasterizador leva a geometria pro mundo; aqui é o raio que vem pro
        // espaço local do objeto. Isso permite usar equações arbitrárias tipo a
        // da esfera pro teste de interseção.
        let local = objeto.transform.undo_ray(raio);

        // O `t` é igual nos espaços local e de mundo.
        if let Some(hit) = objeto.geometry.intersect(local)
            && hit.t >= t_min
            && hit.t < limite
        {
            limite = hit.t;

            acertou = Some(Acerto {
                objeto,
                surface: Surface {
                    point: raio.at(hit.t),
                    normal: objeto.transform.apply_normal(hit.normal),
                    uv: hit.uv,
                },
            });
        }
    }

    acertou
}

// Teoria: raytracer.md#o-raio-de-sombra
/// Existe alguma coisa entre o ponto e a luz?
fn ocluido(scene: &Scene, superficie: &Surface, luz: &Light, contador: &Contador) -> bool {
    let (direcao, distancia) = match *luz {
        Light::Point { position, .. } => {
            let ate_a_luz = position - superficie.point;
            (ate_a_luz.normalized(), ate_a_luz.length())
        }
        // Luz direcional não tem posição: qualquer coisa no caminho bloqueia,
        // por mais longe que esteja.
        Light::Directional { direction, .. } => ((-direction).normalized(), f32::INFINITY),
        Light::Ambient { .. } => return false,
    };

    let origem = desloca(superficie.point, superficie.normal, direcao);

    contador.raio();

    // Sombra só precisa saber *se* algo bloqueia, e não qual é o mais próximo —
    // a busca poderia parar no primeiro acerto. A otimização entra junto com a
    // travessia de BVH de M3.
    mais_proximo(
        scene,
        Ray::new(origem, direcao),
        EPSILON,
        distancia,
        contador,
    )
    .is_some()
}

// Teoria: raytracer.md#o-acne-e-o-epsilon
/// Afasta a origem do raio secundário da superfície que o gerou, para o lado em
/// que ele vai seguir.
fn desloca(ponto: Vec3, normal: Vec3, direcao: Vec3) -> Vec3 {
    let lado = if direcao.dot(normal) < 0.0 { -1.0 } else { 1.0 };

    ponto + normal * (EPSILON * lado)
}

// Teoria: raytracer.md#o-raio-refletido
/// A direção espelhada de `d` em torno de `n`.
fn reflete(d: Vec3, n: Vec3) -> Vec3 {
    d - n * (2.0 * d.dot(n))
}

// Teoria: raytracer.md#o-raio-refratado
/// A direção transmitida pela lei de Snell, ou `None` na reflexão interna
/// total. `d` e `n` unitários, `indice` o índice de refração do objeto (o meio
/// de fora é sempre o vácuo).
fn refrata(d: Vec3, n: Vec3, indice: f32) -> Option<Vec3> {
    //             n ^
    //               |
    //         -d    |
    //           \   |
    //            \θ₁|
    //             \ |
    //     ----------+----------> û    (a superfície)
    //               | \
    //               |θ₂\
    //               |   \
    //               |    t
    let cos_i = -d.dot(n);

    // cos_i negativo é o raio saindo de dentro do objeto: a normal aponta pro
    // lado oposto de -d, e os dois meios trocam de papel.
    let (n, eta, cos_i) = if cos_i < 0.0 {
        (-n, indice, -cos_i)
    } else {
        (n, 1.0 / indice, cos_i)
    };

    // sin θ₂ = η · sin θ₁
    // Eleva tudo ao quadrado:
    // sin²θ₂ = η² · sin²θ₁
    // Com pitágoras sin²θ₁ = 1 − cos²θ₁, então trocamos:
    // sin²θ₂ = η² · (1 − cos²θ₁)
    let sen2_t = eta * eta * (1.0 - cos_i * cos_i);
    if sen2_t > 1.0 {
        // Caso de refração interna total: sin θ₂ é maior que 1 (inválido)
        return None;
    }

    // Pra descobrir cos θ₂ usamos pitágoras:
    // |t|² = (sin θ₂)² + (cos θ₂)²
    // cos θ₂ = √(1 - (sin θ₂)²)
    let cos_t = (1.0 - sen2_t).sqrt();

    // Podemos decompor d e t em componente paralelo à normal + componente tangente à superfície:
    //  d = sin θ₁ · û  +  cos θ₁ · -n
    //  t = sin θ₂ · û  +  cos θ₂ · -n
    //
    // A lei de Snell: n₁ sin θ₁ = n₂ sin θ₂ -> sin θ₂ = η · sin θ₁, com η = n₁/n₂
    // Onde n1 e n2 são os índices de refração dos dois meios
    //
    // Então:
    //  t = sin θ₂ · û  +  cos θ₂ · -n
    //  t = (η ·  sin θ₁) · û  −  cos θ₂ · n
    //  t =  η · (sin θ₁  · û) −  cos θ₂ · n
    // Podemos conseguir a componente tangente de d com: d_tan = d - (componente paralela) = d - (cos θ₁ · -n);
    // ou seja, removendo o deslocamento na direção da normal e deixando apenas o descolamento na direção da tangente:
    //  t =  η · (d + cos θ₁·n) −  cos θ₂ · n
    //  t =  η·d + η·cos θ₁·n   −  cos θ₂ · n
    //  t =  η·d + (η·cos θ₁ − cos θ₂)·n
    Some(d * eta + n * (eta * cos_i - cos_t))
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
