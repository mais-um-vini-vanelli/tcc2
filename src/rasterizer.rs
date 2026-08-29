//! Rasterizador.
//!
//! Faz só o caminho mais curto do pipeline: projeta os vértices do triângulo,
//! percorre a *bounding box* em espaço de tela e testa cobertura por edge
//! functions (Pineda, 1988), pintando o albedo do material como cor sólida.
//!
//! Ainda **não** existem aqui: clipping, z-buffer, backface culling,
//! baricêntricas, textura e shading.
//!
//! `Sphere` não é desenhada: superfície implícita não tem como ser rasterizada
//! sem tesselar. Objeto não suportado é pulado, não é erro — a mesma cena vale
//! pros dois modos, cada um desenha o que sabe.

use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer};
use crate::scene::{Camera, Primitive, Scene};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer;

impl Renderer for Rasterizer {
    fn name(&self) -> &'static str {
        "rasterizador"
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
        for objeto in &scene.objects {
            let Primitive::Triangle {
                points, material, ..
            } = objeto
            else {
                continue;
            };
            let projetados = points.map(|p| projeta(&scene.camera, base, p, largura, altura));
            let [a, b, c] = projetados;
            let (Some(a), Some(b), Some(c)) = (a, b, c) else {
                // Por enquanto sem clipping: triângulo que algum ponto cruza o near plane cai fora inteiro
                continue;
            };
            preenche(&mut cor, [a, b, c], material.albedo, &mut stats);
        }

        result.stats = stats;
        result.insert(RenderPass::Color, cor);
        result
    }
}

/// Projeção mundo -> tela.
fn projeta(
    camera: &Camera,
    base: (Vec3, Vec3, Vec3),
    ponto: Vec3,
    largura: u32,
    altura: u32,
) -> Option<Vec3> {
    let (direita, cima, frente) = base;

    // Converter o ponto de world space pra camera space.
    // - montar o vetor que vai da origem da câmera até o ponto. Esse vetor também pode ser visto como as coordenadas do
    //   ponto em um sistema de coordenadas com a câmera em (0,0,0)
    // - projetar esse vetor em cada eixo da câmera pra descobrir a coordenada do ponto naquele eixo
    //   como os 3 vetores base da câmera são normalizados, o resultado de cada produto é o comprimento do cateto adjacente de
    //   um triângulo retângulo que vai de origem -> ponto -> projeção do ponto naquele eixo.
    let d = ponto - camera.position();
    let (x, y, z) = (d.dot(direita), d.dot(cima), d.dot(frente));

    let (near, _far) = camera.clip_planes();
    if z < near {
        return None;
    }

    // Converter o ponto de camera space pra ndc (retângulo de [-1,-1] -> [1,1])
    let (ndc_x, ndc_y) = match *camera {
        Camera::Perspective { fov, aspect, .. } => {
            // A área visível é um cone saindo da câmera, e abrindo mais conforme se afasta no eixo z:
            //
            //                               ┌── topo da tela
            //                      ┌─────────┘
            //             ┌─────────┘
            //   câmera ●──┴───────────────────────── eixo de visão (centro da tela)
            //             └─────────┐
            //                      └─────────┐
            //                               └── base da tela
            //          z=1        z=3        z=6
            //
            // Perto da câmera cabe pouco mundo na tela; longe cabe muito. Objetos próximos ficam maiores,
            // objetos distantes ficam menores (as coordenadas x,y dos pontos ficam mais próximas de 0)

            // Divide o fov por 2 pra descobrir o ângulo entre o topo e a origem (ou a base e a origem, mesmo
            // valor). E depois pega a tangente desse ângulo, que diz *quanto a altura aumenta a cada unidade que
            // a gente avança em z*, já que tan() = oposto/adjacente
            let meia_altura = (fov.to_radians() * 0.5).tan();

            // Agora a conversão dos pontos em camera space pra ndc
            // Para o eixo y:
            //   - o ponto está em (x,y,z); y unidades de altura
            //   - no eixo y, o topo da tela fica a z * meia_altura unidades
            //   - então a coordenada y projetada vai ficar em y / topo da tela = y / (z * meia_altura)

            // Para o eixo x, não podemos usar só tan(fov/2), porque o fov é o ângulo vertical e a tela geralmente tem uma
            // largura diferente da sua altura (a quantidade de unidades que a caixa de visão aumenta com a distância é maior
            // que tan(fov/2))

            // O aspect já diz a proporção largura/altura, então multiplicamos tan(fov/2) (taxa de aumento da altura/unidade de z) por
            // aspect pra descobrir taxa de aumento da largura/unidade de z.
            // Se a largura da tela é 2x a altura, faz sentido o tamanho da caixa de visão crescer 2x mais rápido que a altura

            // Prova:
            // Num triângulo retângulo, tan(ângulo) = oposto / adjacente
            //
            //              │ ← H  (meia-altura visível: o "oposto")
            // câmera ●─────┘
            //         ← z →   (a distância: o "adjacente")

            // tan(fov_v / 2) = height / z

            // aspect = width / height -> width = aspect * height
            // tan(fov_h / 2) = W / z                     (definição do fov horizontal)
            //                = (H × aspect) / z          (porque W = H × aspect)
            //                = (H / z) × aspect          (só reagrupando a multiplicação)
            //                = tan(fov_v / 2) × aspect   (porque H/z é o tan vertical)
            (x / (z * meia_altura * aspect), y / (z * meia_altura))
        }

        Camera::Orthographic { width, height, .. } => {
            // A área visível é uma caixa de width * height
            // Usando o eixo x de exemplo, o centro da área fica em x = 0, e os limites em -width/2 e +width/2

            // Como a gente quer um fator f que converte um ponto com x = -width/2 em x = -1 em ndc E um ponto com
            // x = +width/2 em x = 1 em ndc, dá de transformar em sistema de equação e resolver:

            // A) -width/2 * f = -1
            // B) +width/2 * f =  1
            //
            // Isolando f na equação B, tem f = 1/(width/2). Trocando f na equação A tem -width/2 * 1/(width/2) = -1, que é
            // uma equação verdadeira. Então dá de usar 1/(width/2) pra converter coordenadas do eixo x de camera space em ndc,
            // e a mesma ideia vale pro eixo y.

            // Na projeção ortográfica o tamanho dos objetos não muda com a distância da câmera, então z não é usado na projeção
            (x / (width * 0.5), y / (height * 0.5))
        }
    };

    // Agora sim, só converter as coordenadas x, y = [-1, 1] em uma matrix de pixels x = [0, resolução (largura)] e
    // y = [0, resolução (altura)]

    // Exemplo com 320x240:

    // operação	| faixa depois dela | o que virou
    //     x     -1 … 1
    // + 1.0	  0 … 2	              tira o negativo, origem vai pra borda esquerda
    // * 0.5	  0 … 1	              vira fração da tela
    // * 320	  0 … 320	          vira index do pixel

    // y cresce pra cima em NDC e pra baixo no buffer, então a gente primeiro inverte a direção com 1-y:

    // y = +1 (topo)   -> 1-1 = 0 (base)
    // y =  0 (centro) -> 1-0 = 1 (centro)
    // y = -1 (base)   -> 1-(-1) = 2 (topo)

    // e depois as mesmas etapas pra converter 0… 2 -> 0...1 e 0...1 -> 0 … 240
    Some(Vec3::new(
        (ndc_x + 1.0) * 0.5 * largura as f32,
        (1.0 - ndc_y) * 0.5 * altura as f32,
        z, // a posição z do ponto fica salva sem projeção
    ))
}

/// A edge function de Pineda (1988).
fn borda(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    // Essa função responde a pergunta:
    // Traçando uma reta infinita que passa por a e b, o ponto p caiu de que lado?

    // u = b - a (vetor a->b)
    let u = b - a;
    // v = p - a (vetor a->p)
    let v = p - a;

    // tiramos o produto vetorial u x v = u.x·v.y − u.y·v.x
    // o sinal da resposta diz pra qual lado v está de u
    // a magnitude da resposta diz o tamanho do ângulo entre u e v (com 90 graus entre eles, o resultado chega no ápice)
    u.x * v.y - u.y * v.x
}

fn preenche(cor: &mut ColorBuffer, vertices: [Vec3; 3], albedo: Rgb, stats: &mut RenderStats) {
    let [a, b, c] = vertices.map(|v| Vec2::new(v.x, v.y));

    let area = borda(a, b, c);
    if area == 0.0 {
        // Triângulo sem área (linha)
        return;
    }

    let (largura, altura) = (cor.width() as f32, cor.height() as f32);

    // Calcular a bounding box do triângulo
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as u32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(largura - 1.0).max(0.0) as u32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as u32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(altura - 1.0).max(0.0) as u32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            stats.primitive_tests += 1;

            // Usar o centro do pixel. Pixel (1,1) tem centro 1.5,1.5 por exemplo
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let (w0, w1, w2) = (borda(b, c, p), borda(c, a, p), borda(a, b, p));

            // Sem backface culling, aceita as duas orientações. Um ponto está dentro do triângulo se ele está *do mesmo
            // lado das 3 arestas*
            let dentro = if area > 0.0 {
                w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
            } else {
                w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
            };
            if dentro {
                cor.set(x, y, albedo);
            }
        }
    }
}
