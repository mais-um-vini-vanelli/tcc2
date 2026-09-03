//! Rasterizador.
//!
//! A pipeline até agora:
//! -> world space
//! -> camera space
//! -> recorte contra o frustum (Sutherland e Hodgman, 1974)
//! -> screen space
//! -> teste de cobertura por edge functions (Pineda, 1988) dentro da bounding box
//! -> teste de profundidade contra o z-buffer
//!
//! Ainda **não** existem aqui: backface culling, wireframe, msaa, interpolação
//! de uv/normal, textura e shading. A cor continua sendo o albedo do material,
//! sólido.
//!
//! `Sphere` não é desenhada: superfície implícita não tem como ser rasterizada
//! sem tesselar. Objeto não suportado é pulado, não é erro — a mesma cena vale
//! pros dois modos, cada um desenha o que sabe.

use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer, ScalarBuffer};
use crate::scene::{Camera, Primitive, Scene};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer;

impl Renderer for Rasterizer {
    fn name(&self) -> &'static str {
        "rasterizador"
    }

    fn render(&self, scene: &Scene, params: &Parameters, passes: &[RenderPass]) -> RenderResult {
        let mut result = RenderResult::new();
        let quer_cor = passes.contains(&RenderPass::Color);
        let quer_profundidade = passes.contains(&RenderPass::Depth);
        if !quer_cor && !quer_profundidade {
            return result;
        }

        let (largura, altura) = (params.output.width, params.output.height);
        let fundo = params.output.background.unwrap_or(Rgb::BLACK);
        let mut alvos = Alvos {
            cor: ColorBuffer::filled(largura, altura, fundo),
            profundidade: ScalarBuffer::filled(largura, altura, f32::INFINITY),
        };
        let mut stats = RenderStats::default();

        let base = scene.camera.basis();
        let planos = frustum(&scene.camera);
        let perspectiva = matches!(scene.camera, Camera::Perspective { .. });

        for objeto in &scene.objects {
            let Primitive::Triangle {
                points, material, ..
            } = objeto
            else {
                continue;
            };

            let triangulo = points.map(|p| para_camera(&scene.camera, base, p));
            let poligono = if params.clipping.enabled {
                recorta(triangulo, &planos, &mut stats)
            } else {
                triangulo.to_vec()
            };

            // O recorte devolve um polígono convexo de N vértices; desenhar com triangle fan.
            for i in 1..poligono.len().saturating_sub(1) {
                let tela = [poligono[0], poligono[i], poligono[i + 1]]
                    .map(|p| para_tela(&scene.camera, p, largura, altura));

                let [Some(a), Some(b), Some(c)] = tela else {
                    // Só entra aqui com clipping desligado. Daí o triângulo que cruza o near plane
                    // é descartado.
                    break;
                };

                preenche(
                    &mut alvos,
                    [a, b, c],
                    material.albedo,
                    perspectiva,
                    params.rasterization.depth_test,
                    &mut stats,
                );
            }
        }

        result.stats = stats;
        if quer_cor {
            result.insert(RenderPass::Color, alvos.cor);
        }
        if quer_profundidade {
            result.insert(RenderPass::Depth, alvos.profundidade);
        }
        result
    }
}

/// Os dois buffers que o rasterizador escreve em par, um valor de cada por pixel.
struct Alvos {
    cor: ColorBuffer,
    profundidade: ScalarBuffer,
}

/// Um plano em 3D pode ser definido por um vetor normal `n` e um ponto `p0` que pertence ao plano.
///
/// Um ponto só está no plano se:
/// > n · (p - p0) = 0
/// > n · p - n · p0 = 0
///
/// Definindo deslocamento = -n · p0:
///
/// > n · p + deslocamento = 0
///
/// A gente consegue usar o resultado da fórmula do plano pra saber de qual lado do plano está um ponto P.
///
/// Voltando pra primeira fórmula, e trocando (p - p0) por v:
///
/// > n · v = 0
///
/// Perceba que é um produto escalar entre v e n. Isso quer dizer que o resultado é a projeção do vetor v em cima da normal
/// do plano.
///
/// Quando um ponto está no plano, essa distância é 0, porque v e n são perpendiculares.
///
/// Se um ponto está 'pra cima' do plano (fora do plano e no lado da normal), a projeção de v em n é positiva.
/// Se um ponto está 'pra baixo' do plano (fora do plano e no lado oposto da normal), a projeção de v em n é negativa.
#[derive(Debug, Clone, Copy)]
struct Plano {
    normal: Vec3,
    deslocamento: f32,
}

impl Plano {
    fn novo(normal: Vec3, deslocamento: f32) -> Self {
        Plano {
            normal,
            deslocamento,
        }
    }

    /// Retorna zero se p está em cima do plano, positivo se está no lado da normal e negativo se está do lado contrário da
    /// normal.
    fn distancia(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.deslocamento
    }
}

/// As seis faces do frustum, em espaço de câmera.
fn frustum(camera: &Camera) -> [Plano; 6] {
    let (near, far) = camera.clip_planes();

    let laterais = match *camera {
        Camera::Perspective { fov, aspect, .. } => {
            // A caixa de visão é uma pirâmide partindo da origem, ou seja deslocamento = 0

            // Olhando só pro plano de cima da pirâmide:
            // tan(fov/2) = oposto / adjacente, com adjacente = z
            // A cada unidade de z que avanaça, o plano superior da pirâmide/o maior y visível aumenta em tan(fov/2)
            // Isso quer dizer que pra um ponto p estar 'dentro' na caixa ele precisa ter:
            // p.y <= p.z * meia_altura
            //
            // Revisando e abrindo uma equação de plano:
            // > n · p + d >= 0
            // > n.x * p.x + n.y * p.y + n.z * p.z + d >= 0

            // Colocando o nosso plano na mesma forma:
            // > p.y <= p.z * meia_altura
            // > -p.y + p.z * meia_altura >= 0
            // > 0 * p.x + (-1 * p.y) + meia_altura * p.z >= 0

            // Comparando com a fórmula aberta da equação de plano, a gente pode ver que a normal precisa ser (0, -1, meia_altura) pra
            // cumprir a inequação

            // Então o plano de cima da caixa é n = (0, -1, meia_altura), d = 0
            // E podemos seguir a mesma lógica pras outras faces, trocando tan(fov/2) por aspect * tan(fov/2) pras faces horizontais
            let meia_altura = (fov.to_radians() * 0.5).tan();
            let meia_largura = meia_altura * aspect;

            [
                Plano::novo(Vec3::new(1.0, 0.0, meia_largura), 0.0),
                Plano::novo(Vec3::new(-1.0, 0.0, meia_largura), 0.0),
                Plano::novo(Vec3::new(0.0, 1.0, meia_altura), 0.0),
                Plano::novo(Vec3::new(0.0, -1.0, meia_altura), 0.0),
            ]
        }

        // Na ortográfica a caixa não abre com a distância: as laterais são
        // paralelas ao eixo de visão, em x = ±width/2 e y = ±height/2.
        Camera::Orthographic { width, height, .. } => [
            Plano::novo(Vec3::new(1.0, 0.0, 0.0), width * 0.5),
            Plano::novo(Vec3::new(-1.0, 0.0, 0.0), width * 0.5),
            Plano::novo(Vec3::new(0.0, 1.0, 0.0), height * 0.5),
            Plano::novo(Vec3::new(0.0, -1.0, 0.0), height * 0.5),
        ],
    };

    [
        // Pro near plane queremos a normal apontando pra frente (+z) (0, 0, 1.0)
        // Além disso, qualquer ponto do near plane tem a forma (0, 0, near)

        // O deslocamento do plano é:
        // deslocamento = -(n · p0) = - ((0, 0, 1.0) * (0, 0, near)) = -near

        // O deslocamento é o resultado de distancia() com p = 0:
        // distancia(0) = normal · (0,0,0) + d = d
        // Ele diz o quão pra trás ou pra frente a origem está do plano. A origem sendo a câmera no nosso caso
        Plano::novo(Vec3::new(0.0, 0.0, 1.0), -near),
        Plano::novo(Vec3::new(0.0, 0.0, -1.0), far),
        laterais[0],
        laterais[1],
        laterais[2],
        laterais[3],
    ]
}

/// Recorta o triângulo contra o frustum inteiro, um plano de cada vez.
///
/// Pra um ponto estar visível pra câmera, ele precisa estar dentro dos semiespaços definidos por todos
// os 6 planos.
fn recorta(triangulo: [Vec3; 3], planos: &[Plano; 6], stats: &mut RenderStats) -> Vec<Vec3> {
    let mut poligono = triangulo.to_vec();
    let mut recortou = false;

    for plano in planos {
        // Se todos os pontos do triângulo estão dentro do plano, nem precisa recortar.
        if poligono.iter().all(|&v| plano.distancia(v) >= 0.0) {
            continue;
        }
        poligono = recorta_no_plano(&poligono, *plano);
        recortou = true;
        if poligono.len() < 3 {
            // Todos os pontos do triângulo estão fora do plano: é descartado
            break;
        }
    }

    if recortou {
        stats.polygons_clipped += 1;
    }
    poligono
}

/// Sutherland e Hodgman (1974): percorre as arestas do polígono e monta a lista
/// de saída com o que ficou do lado de dentro de um plano.
fn recorta_no_plano(poligono: &[Vec3], plano: Plano) -> Vec<Vec3> {
    let mut saida = Vec::with_capacity(poligono.len() + 1);

    for (i, &atual) in poligono.iter().enumerate() {
        let proximo = poligono[(i + 1) % poligono.len()];
        let (d_atual, d_proximo) = (plano.distancia(atual), plano.distancia(proximo));

        if d_atual >= 0.0 {
            saida.push(atual);
        }

        if (d_atual >= 0.0) != (d_proximo >= 0.0) {
            // Um ponto dentro e o outro fora do plano: a aresta atravessou o plano, então entra também o ponto
            // onde ela cruza o plano.

            // Se a gente conhece as duas extremidades de uma aresta, a gente pode andar por ela com:
            // > ponto = ponto_atual + (ponto_seguinte - ponto_atual) * t
            // Ou seja, fazendo uma interpolação linear das duas posições com t de 0 a 1:
            // > t = 0 -> devolve ponto_atual
            // > t = 1 -> devolve ponto_seguinte
            // > t = 0.5 -> devolve o ponto médio entre ponto_atual e ponto_seguinte

            // O ponto p onde a aresta cruza o plano tem:

            // 1) plano.distancia(p) = normal · p + deslocamento = 0
            // 2) p = atual + (proximo - atual) * t

            // Substituindo a fórmula de p em 1) temos:

            // plano.distancia(p) = normal · [atual + (proximo - atual) * t] + deslocamento

            // Agora vamos simplificar essa fórmula

            // Propriedades do dot product:
            // P1 distributiva              -> u · (v + w)  = u·v + u·w
            // P3 escalar (k) sai para fora -> u · (k·v)    = k (u·v)

            // Aplicando P1:
            // > normal · [atual + (proximo - atual) * t] + deslocamento
            // > normal · atual + normal · [(proximo - atual) * t] + deslocamento
            // Aplicando P2:
            // > normal · atual + t * [normal · (proximo - atual)] + deslocamento
            // Aplicando P1 denovo no [normal · (proximo - atual)]:
            // > normal · atual + t * (normal · proximo - normal · atual) + deslocamento

            // Note que normal · atual + deslocamento = plano.distancia(atual) (o d_atual que foi calculado acima), então
            // trocamos:
            // > d_atual + t * (normal · proximo - normal · atual)

            // Agora simplificando o (normal · proximo - normal · atual). Note que:
            // > d_atual = normal · atual + deslocamento -> normal · atual = d_atual - deslocamento
            // > d_proximo = normal · proximo + deslocamento -> normal · proximo = d_proximo - deslocamento

            // Trocando:
            // > d_atual + t * [(d_proximo - deslocamento) - (d_atual - deslocamento)]
            // > d_atual + t * (d_proximo - deslocamento - d_atual + deslocamento)
            // deslocamento e -deslocamento se cancelam:
            // > d_atual + t * (d_proximo - d_atual)

            // Perfeito, temos a fórmula da distancia de um ponto da aresta em função de t. Agora é igualar a 0 e
            // resolver pra achar o ponto em que a aresta cruza o plano:

            // > d_atual + t * (d_proximo - d_atual) = 0
            // > t * (d_proximo - d_atual) = -d_atual
            // > t = -d_atual / (d_proximo - d_atual)
            let t = -d_atual / (d_proximo - d_atual);

            // E agora a gente contrói o ponto com a fórmula de interpolação
            saida.push(atual + (proximo - atual) * t);
        }
    }

    saida
}

/// Mundo -> espaço de câmera. É onde o recorte acontece: nesse espaço o frustum
/// já está alinhado com a câmera, mas a divisão por z ainda não aconteceu.
fn para_camera(camera: &Camera, base: (Vec3, Vec3, Vec3), ponto: Vec3) -> Vec3 {
    let (direita, cima, frente) = base;

    // - montar o vetor que vai da origem da câmera até o ponto. Esse vetor também pode ser visto como as coordenadas do
    //   ponto em um sistema de coordenadas com a câmera em (0,0,0)
    // - projetar esse vetor em cada eixo da câmera pra descobrir a coordenada do ponto naquele eixo
    //   como os 3 vetores base da câmera são normalizados, o resultado de cada produto é o comprimento do cateto adjacente de
    //   um triângulo retângulo que vai de origem -> ponto -> projeção do ponto naquele eixo.
    let d = ponto - camera.position();
    Vec3::new(d.dot(direita), d.dot(cima), d.dot(frente))
}

/// Espaço de câmera -> tela. Devolve `(x_pixel, y_pixel, z_camera)`.
fn para_tela(camera: &Camera, ponto: Vec3, largura: u32, altura: u32) -> Option<Vec3> {
    let (x, y, z) = (ponto.x, ponto.y, ponto.z);

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

fn preenche(
    alvos: &mut Alvos,
    vertices: [Vec3; 3],
    albedo: Rgb,
    perspectiva: bool,
    teste_de_profundidade: bool,
    stats: &mut RenderStats,
) {
    let [a, b, c] = vertices.map(|v| Vec2::new(v.x, v.y));

    let area = borda(a, b, c);
    if area == 0.0 {
        // Triângulo sem área (linha)
        return;
    }

    let (largura, altura) = (alvos.cor.width() as f32, alvos.cor.height() as f32);

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

            if !dentro {
                continue;
            }

            // Cada edge function dividida pela área do triângulo vira o peso do
            // vértice oposto àquela aresta: as coordenadas baricêntricas, que
            // somam 1 em qualquer ponto de dentro.

            // A edge function borda(b, c, p) retorna a área do triângulo formado entre (b, c, p) * 2. A mesma coisa pra borda(c, a, p) e borda(a, b, p)

            // Prova:
            //
            // a fórmula da área de um triângulo é base x altura / 2

            // a edge function (a, b, p) usa os dois vetores u e v, com u = (a, b) e v = (a, p)
            // podemos visualizar u como sendo a base de um triângulo, e v sendo uma das arestas laterais
            // para descobrir a base desse triângulo precisamos de |u| * altura
            // altura é a distância de p até a aresta u, em outras palavras: a projeção de v no eixo perpendicular a u

            // Então pra descobrir a altura podemos:
            // 1) rotacionar u em 90º
            // 2) normalizar u_rotacionado, dividindo pelo seu comprimento |u|
            // 3) projetar v em u_rotacionado com um produto escalar
            //
            // Então a altura do triângulo a, b, p é
            // > h = (u_rotacionado / |u|) · v

            // Podemos reorganizar a fórmula:
            // > h = (u_rotacionado · v) / |u|
            // > h * |u| = u_rotacionado · v

            // Se temos um vetor u = (x, y), a fórmula desse vetor rotacionado em 90º é u'= (-y, x)
            // Isso pode ser visto em:
            // (1, 0)  →  (0, 1) "direita" vira "cima"
            // (0, 1)  →  (−1, 0) "cima" vira "esquerda"
            //
            // Então u_rotacionado = (-u.y, u.x):
            // > h * |u| = (-u.y, u.x) · v
            // Desenvolvendo o produto escalar e reorganizando:
            // > |u| * h = -u.y * v.x + u.x * v.y

            // Agora olha que legal: à esquerda da equação ficou base (|u|) * altura, e à direita ficou a fórmula da edge function
            // Considerando que a área de um triângulo é base * altura / 2, dá de ver que o resultado da edge function é o dobro da área do triângulo a, b, p

            // Se p está dentro do triângulo, a soma das áreas dos subtriângulos abp, bcp e acp precisa ser igual à area do triangulo abc
            // Isso pode ser visto intuitivamente
            // [TODO: desenho de um triângulo com P dentro e os 3 subtriângulos]

            // E quanto maior o subtriângulo formado entre uma aresta de abc e p, mais perto p está do ponto aposto à aresta, e mais longe ele está dos
            // outras 2 pontos, que formam a aresta.
            // Isso também pode ser visto intuitivamente
            // [TODO: imagem mostrando p em cima de um ponto (subtriângulo = abc), p no meio de uma aresta, p no meio do triângulo]

            // Outra forma de ver isso é comparando o subtriângulo com o triângulo completo:
            // - pedaço (b,c,p) → base bc, altura = distância de p até a reta bc
            // - triângulo (a,b,c) → base bc (a mesma!), altura = distância de a até a reta bc

            // Os dois têm a mesma base. E quando dois triângulos compartilham a base, a razão entre as áreas é a razão entre as alturas:

            //        base·h_p / 2        h_p        distância de p até a aresta bc
            //  l0 = ─────────────────  = ─────  =  ────────────────────────────────
            //        base·h_a / 2        h_a        distância de a até a aresta bc

            // Ou seja: l0 mede o quanto p está longe da aresta oposta, em relação ao quanto o vértice a está.
            // E se l0 = 1, p está tão longe de bc quanto a: p está em cima de a

            // Então podemos usar a proporção (área do subtriangulo)/(área do triângulo) para descobrir o quão perto p está de cada vértice, e usar isso como
            // coeficiente de interpolação

            let (l0, l1, l2) = (w0 / area, w1 / area, w2 / area);
            let z = profundidade(vertices, (l0, l1, l2), perspectiva);

            if teste_de_profundidade {
                let mais_proximo = alvos.profundidade.get(x, y).unwrap_or(f32::INFINITY);
                if z >= mais_proximo {
                    continue;
                }
            }

            alvos.profundidade.set(x, y, z);
            alvos.cor.set(x, y, albedo);
        }
    }
}

/// Descobrir a coordenada z do ponto do triângulo que foi projetado no pixel atual. Pra isso a gente
/// interpola p.z dos 3 vértices usando as coordenadas baricênctricas que a edge function retornou
fn profundidade(vertices: [Vec3; 3], pesos: (f32, f32, f32), perspectiva: bool) -> f32 {
    let (l0, l1, l2) = pesos;
    let [a, b, c] = vertices;

    if perspectiva {
        // A projeção ortográfica é afim: transformações afins preservam razões ao longo de uma reta. O ponto médio continua o ponto médio; o ponto a 1/3 continua a 1/3.
        // Como consequência, preservam também razões entre áreas (coordenadas baricêntricas são exatamente razões entre áreas).

        // Mas a projeção perspectiva não é afim, uma vez que a gente divide cada ponto pelo seu 'z' (variável, não constante). Ela mantém as retas, mas distorce as proporções.
        // Exemplo: uma aresta de A = (x=-1, z=1) até B = (x=1, z=5). O ponto médio P antes de projetar é (fórmula de interpolação):
        // > (-1, 1) + ((1, 5) - (-1, 1)) * 0.5 = (x=0, z=3)
        // Então
        // A = (x=-1, z=1)
        // P = (x= 0, z=3)
        // B = (x= 1, z=5)

        // Projetando os 3:
        // A' = (-1.0, 1)
        // P' = ( 0.0, 3)
        // B' = ( 0.2, 5)

        // Perceba que a distância em x entre A->P e P->B era 1 nos dois casos antes da projeção, mas agora a parte mais longe da aresta ocupa um espaço menor.
        // Esse é o mesmo efeito visto na hora de fazer a projeção de perspectiva.

        // Se a gente tentar interpolar A' e B' pra encontrar onde P foi projetado, o resultado não bate com o P' calculado acima:
        // > (-1.0, 1) + ((0.2, 5) - (-1.0, 1)) * 0.5 = (x=-0.4, z=3) *X errado*

        // Interpolar com 1/z pra compensar a distorção de perspectiva
        // [TODO: demonstrar isso direito]
        1.0 / (l0 / a.z + l1 / b.z + l2 / c.z)
    } else {
        // A projeção ortográfica não distorce o espaço na conversão camera space -> ndc. É só interpolar
        // normalmente
        l0 * a.z + l1 * b.z + l2 * c.z
    }
}
