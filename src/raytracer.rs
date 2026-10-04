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
//! Com `acceleration.bvh` ligado, quem escolhe quais objetos e triângulos cada
//! raio testa é a BVH de `bvh.rs`; desligado, todo raio testa a cena inteira.

use std::cell::Cell;
use std::time::Instant;

use crate::color::Rgb;
use crate::math::{Ray, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderResult, RenderStats, Renderer, ScalarBuffer};
use crate::scene::{self, Camera, Hit, Instance, Light, Material, Primitive, Scene};
use crate::shading::{self, Surface};

use crate::bvh::Aceleracao;
use crate::overlay;

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
        let fundo = params.output.background;
        let mut cor = ColorBuffer::filled(largura, altura, fundo);
        let mut raios = ScalarBuffer::new(largura, altura);

        let contador = Contador::default();

        let (camera, lights, instancias) = scene.flatten(params.transform.method);

        let Some(camera) = camera else {
            // Sem câmera não há de onde partir raio: sobra o fundo.
            return RenderResult {
                color: Some(cor),
                ray_count: Some(raios),
                ..RenderResult::new()
            };
        };

        let aceleracao = params.ray_tracing.acceleration.bvh.then(|| {
            let inicio = Instant::now();
            let aceleracao = Aceleracao::build(
                &instancias,
                params.ray_tracing.acceleration.split,
                params.transform.method,
            );

            contador.atualiza(|s| {
                s.bvh_build_time = inicio.elapsed();
                (s.tlas_depth, s.blas_depth) = aceleracao.depth();
            });

            aceleracao
        });

        let mut calor = aceleracao
            .is_some()
            .then(|| ScalarBuffer::new(largura, altura));

        let contexto = Contexto {
            lights,
            instancias,
            params,
            aceleracao,
            contador,
        };

        let base = camera.basis();
        let (near, far) = camera.clip_planes();

        for y in 0..altura {
            for x in 0..largura {
                let antes = contexto.contador.stats();

                let raio = raio_primario(&camera, base, x, y, largura, altura);
                contexto.contador.raio();

                cor.set(x, y, contexto.trace(raio, near, far, 0));

                // Os passes de custo são o quanto os contadores andaram durante
                // o pixel: com sombra e recursão, não só o raio primário.
                let depois = contexto.contador.stats();

                raios.set(x, y, (depois.rays - antes.rays) as f32);

                if let Some(calor) = &mut calor {
                    let nos = depois.bvh_nodes_visited - antes.bvh_nodes_visited;
                    calor.set(x, y, nos as f32);
                }
            }
        }

        cor.apply_output(&params.output);

        if let Some(profundidade) = params.ray_tracing.acceleration.wireframe {
            if let Some(aceleracao) = &contexto.aceleracao {
                let aabbs =
                    aceleracao.aabbs(&contexto.instancias, params.transform.method, profundidade);

                const CORES: [Rgb; 6] = [
                    Rgb::new(1.0, 1.00, 1.0),
                    Rgb::new(1.0, 0.85, 0.2),
                    Rgb::new(0.3, 1.00, 0.4),
                    Rgb::new(0.3, 0.80, 1.0),
                    Rgb::new(0.8, 0.40, 1.0),
                    Rgb::new(1.0, 0.40, 0.4),
                ];

                for (nivel, cantos) in aabbs {
                    overlay::caixa(
                        &mut cor,
                        &camera,
                        cantos,
                        CORES[(nivel.max(1) - 1) as usize % CORES.len()],
                    );
                }
            }
        }

        RenderResult {
            color: Some(cor),
            ray_count: Some(raios),
            bvh_heatmap: calor,
            stats: contexto.contador.stats(),
            // depth/normal/uv chegam em M5.
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
pub(crate) struct Contador(Cell<RenderStats>);

impl Contador {
    pub(crate) fn raio(&self) {
        self.atualiza(|s| s.rays += 1);
    }

    pub(crate) fn teste(&self) {
        self.testes(1);
    }

    pub(crate) fn testes(&self, quantos: u64) {
        self.atualiza(|s| s.primitive_tests += quantos);
    }

    pub(crate) fn no(&self) {
        self.atualiza(|s| s.bvh_nodes_visited += 1);
    }

    fn stats(&self) -> RenderStats {
        self.0.get()
    }

    fn atualiza(&self, f: impl FnOnce(&mut RenderStats)) {
        let mut stats = self.0.get();
        f(&mut stats);
        self.0.set(stats);
    }
}

/// O que todo raio do render consulta.
struct Contexto<'a> {
    lights: Vec<Light>,
    instancias: Vec<Instance<'a>>,
    params: &'a Parameters,
    /// `None` com a BVH desligada: aí todo raio testa todo objeto.
    aceleracao: Option<Aceleracao>,
    contador: Contador,
}

/// O material da instância acertada e a superfície no ponto de acerto, já em
/// espaço de mundo.
struct Acerto<'a> {
    material: &'a Material,
    surface: Surface,
    normal_geometrica: Vec3,
}

impl<'a> Contexto<'a> {
    // Teoria: raytracer.md#a-recursao-de-whitted
    /// A cor que chega pela direção contrária ao raio.
    ///
    /// ```text
    /// L = Le                                     <- shading::shade, o emitido
    ///   + Σ_luzes V · reflectancia · radiancia   <- shading::shade, o termo direto
    ///   + F(v, n) · L(raio refletido)            <- recursão
    ///   + kt      · L(raio refratado)            <- recursão
    /// ```
    fn trace(&self, raio: Ray, t_min: f32, t_max: f32, profundidade: u32) -> Rgb {
        let opcoes = &self.params.ray_tracing;

        let Some(acerto) = self.mais_proximo(raio, t_min, t_max, Consulta::Visao) else {
            return self.params.output.background;
        };

        let superficie = acerto.surface;
        let geometrica = acerto.normal_geometrica;
        let material = acerto.material;
        // Um raio não sabe dos raios vizinhos, então não há footprint: a textura
        // fica no nível 0.
        let amostra = material.sample(superficie.uv, None, self.params.shading.filtering);

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

                if self.ocluido(s.point, geometrica, luz) {
                    0.0
                } else {
                    1.0
                }
            };

            shading::shade(
                &superficie,
                &amostra,
                &self.lights,
                v,
                self.params.shading.reflectance,
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

            self.contador.raio();

            let vindo_do_espelho = self.trace(
                Ray::new(desloca(superficie.point, geometrica, refletida), refletida),
                EPSILON,
                f32::INFINITY,
                profundidade + 1,
            );

            cor = cor + fresnel * vindo_do_espelho;
        }

        // Teoria: raytracer.md#o-raio-refratado
        if opcoes.refractions && amostra.transparency > 0.0 {
            // Reflexão interna total: não há caminho de saída, e a energia que
            // seria transmitida volta como reflexo.
            let transmitida = refrata(direcao, n, amostra.refractive_index)
                .unwrap_or_else(|| reflete(direcao, n));

            self.contador.raio();

            let vindo_de_dentro = self.trace(
                Ray::new(
                    desloca(superficie.point, geometrica, transmitida),
                    transmitida,
                ),
                EPSILON,
                f32::INFINITY,
                profundidade + 1,
            );

            // O que não voltou pelo espelho é o que entrou na superfície, e
            // `transparency` diz quanto disso atravessa em vez de virar difuso.
            let peso = (Rgb::WHITE - fresnel) * amostra.transparency;

            cor = cor + vindo_de_dentro * peso;
        }

        cor
    }

    // Teoria: bvh.md#dois-niveis
    /// O acerto mais próximo da origem do raio, entre `t_min` e `t_max`, entre
    /// os objetos que a `consulta` enxerga.
    ///
    /// Os dois níveis da aceleração aparecem aqui, com a mesma forma: a TLAS
    /// escolhe entre os objetos, e o teste de um objeto é uma travessia da
    /// BLAS dele entre as faces.
    fn mais_proximo(
        &self,
        raio: Ray,
        t_min: f32,
        t_max: f32,
        consulta: Consulta,
    ) -> Option<Acerto<'a>> {
        // Teoria: transform.md#o-raio-vai-para-o-espaco-local
        // O rasterizador leva a geometria pro mundo; aqui é o raio que vem pro
        // espaço local do objeto. Isso permite usar equações arbitrárias tipo a
        // da esfera pro teste de interseção. O `t` é igual nos dois espaços.
        let intersecta = |i: usize, limite: f32| -> Option<(f32, Hit)> {
            let instancia = &self.instancias[i];

            if consulta == Consulta::Sombra && !instancia.casts_shadows {
                return None;
            }

            let local = instancia.chain.undo_ray(raio, self.params.transform.method);

            let hit = match self.aceleracao.as_ref().and_then(|a| a.blas[i].as_ref()) {
                Some(blas) => {
                    let Primitive::Mesh(malha) = instancia.geometry else {
                        unreachable!();
                    };

                    let (_, hit) =
                        blas.percorre(local, t_min, limite, &self.contador, |face, _| {
                            self.contador.teste();

                            let hit = scene::intersecao_triangulo(local, malha.face(face)?)?;
                            Some((hit.t, hit))
                        })?;

                    hit
                }

                None => {
                    // Sem BLAS a malha testa face por face, e a conta tem que mostrar.
                    self.contador.testes(match instancia.geometry {
                        Primitive::Mesh(malha) => malha.face_count() as u64,
                        _ => 1,
                    });

                    instancia.geometry.intersect(local)?
                }
            };

            Some((hit.t, hit))
        };

        let (i, hit) = match &self.aceleracao {
            Some(aceleracao) => {
                aceleracao
                    .tlas
                    .percorre(raio, t_min, t_max, &self.contador, intersecta)?
            }

            // Força bruta: testa toda instância e fica com o menor t.
            None => (0..self.instancias.len())
                .filter_map(|i| Some((i, intersecta(i, t_max)?.1)))
                .filter(|(_, hit)| hit.t >= t_min && hit.t < t_max)
                .min_by(|(_, a), (_, b)| a.t.total_cmp(&b.t))?,
        };

        let instancia = &self.instancias[i];
        let method = self.params.transform.method;

        let normal = instancia.material.shading_normal(
            hit.uv,
            None,
            self.params.shading.filtering,
            instancia.chain.apply_normal(hit.normal, method),
            instancia.chain.apply_tangent(hit.tangent, method),
        );

        Some(Acerto {
            material: instancia.material,
            surface: Surface {
                point: raio.at(hit.t),
                normal,
                uv: hit.uv,
            },
            normal_geometrica: instancia.chain.apply_normal(hit.geometric_normal, method),
        })
    }

    // Teoria: raytracer.md#o-raio-de-sombra
    /// Existe alguma coisa entre o ponto e a luz?
    fn ocluido(&self, ponto: Vec3, geometrica: Vec3, luz: &Light) -> bool {
        let (direcao, distancia) = match *luz {
            Light::Point { position, .. } | Light::Spot { position, .. } => {
                let ate_a_luz = position - ponto;
                (ate_a_luz.normalized(), ate_a_luz.length())
            }
            // Luz direcional não tem posição: qualquer coisa no caminho bloqueia,
            // por mais longe que esteja.
            Light::Directional { direction, .. } => ((-direction).normalized(), f32::INFINITY),
            Light::Ambient { .. } => return false,
        };

        let sombra = Ray::new(desloca(ponto, geometrica, direcao), direcao);

        self.contador.raio();

        self.mais_proximo(sombra, EPSILON, distancia, Consulta::Sombra)
            .is_some()
    }
}

/// Pra que serve o raio. O de sombra não enxerga quem tem `casts_shadows`
/// desligado, e é só nisso que os dois diferem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Consulta {
    Visao,
    Sombra,
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
    let cos_i = -d.dot(n);

    // cos_i negativo é o raio saindo de dentro do objeto: a normal aponta pro
    // lado oposto de -d, e os dois meios trocam de papel.
    let (n, eta, cos_i) = if cos_i < 0.0 {
        (-n, indice, -cos_i)
    } else {
        (n, 1.0 / indice, cos_i)
    };

    // sin²θ₂ = η² · (1 − cos²θ₁)
    let sen2_t = eta * eta * (1.0 - cos_i * cos_i);
    if sen2_t > 1.0 {
        // Caso de refração interna total: sin θ₂ é maior que 1 (inválido)
        return None;
    }

    let cos_t = (1.0 - sen2_t).sqrt();

    // t = η·d + (η·cos θ₁ − cos θ₂)·n
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
