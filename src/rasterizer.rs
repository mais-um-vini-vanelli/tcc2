//! Rasterizador.
//!
//! Teoria: rasterizer.md#a-pipeline
//!
//! O rasterizador trabalha só com malha: todo objeto da cena passa por
//! `Primitive::tessellate` na entrada.
//!
//! Ainda **não** existem aqui: wireframe, msaa, textura e shading. A cor
//! continua sendo o albedo do material, sólido — normal e uv já são
//! interpolados, mas só aparecem nos passes `Normal` e `Uv`.

use crate::color::Rgb;
use crate::math::{Baricentrics, Plane, Vec2, Vec3, Vertex};
use crate::parameters::Parameters;
use crate::render::{ColorBuffer, RenderResult, RenderStats, Renderer, ScalarBuffer};
use crate::scene::{Camera, Material, Scene};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer;

impl Renderer for Rasterizer {
    fn name(&self) -> &'static str {
        "rasterizador"
    }

    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult {
        let (largura, altura) = (params.output.width, params.output.height);

        let fundo = params.output.background.unwrap_or(Rgb::BLACK);

        let mut resultado = RenderResult {
            color: Some(ColorBuffer::filled(largura, altura, fundo)),
            depth: Some(ScalarBuffer::filled(largura, altura, f32::INFINITY)),
            normal: Some(ColorBuffer::filled(largura, altura, Rgb::BLACK)),
            uv: Some(ColorBuffer::filled(largura, altura, Rgb::BLACK)),
            ..RenderResult::new()
        };

        let mut stats = RenderStats::default();

        let base = scene.camera.basis();
        let planos = frustum(&scene.camera);
        let perspectiva = matches!(scene.camera, Camera::Perspective { .. });

        for objeto in &scene.objects {
            let malha = objeto.tessellate();

            for indice in 0..malha.face_count() {
                let Some(face) = malha.face(indice) else {
                    continue;
                };

                let triangulo = face.map(|v| Vertex {
                    position: para_camera(&scene.camera, base, v.position),
                    ..v
                });

                let poligono = if params.clipping.enabled {
                    recorta(triangulo, &planos, &mut stats)
                } else {
                    triangulo.to_vec()
                };

                // O recorte devolve um polígono convexo de N vértices; desenhar com triangle fan.
                for i in 1..poligono.len().saturating_sub(1) {
                    let tela = [poligono[0], poligono[i], poligono[i + 1]]
                        .map(|v| para_tela(&scene.camera, v, largura, altura));

                    let [Some(a), Some(b), Some(c)] = tela else {
                        // Só entra aqui com clipping desligado. Daí o triângulo que cruza o near plane
                        // é descartado.
                        break;
                    };

                    let area = borda(a.position.xy(), b.position.xy(), c.position.xy());
                    if area == 0.0 {
                        // Triângulo sem área (uma linha)
                        continue;
                    }

                    // Teoria: rasterizer.md#backface-culling
                    // Backface culling:
                    // Descartamos aqui fora porque um mesmo triângulo pode virar vários durante o clipping
                    // daí descartar dentro do preenche poderia trazer uma contagem de triângulos dercartados
                    // estranha para o usuário.

                    if params.rasterization.backface_cull && area > 0.0 {
                        stats.backfaces_culled += 1;
                        break;
                    }

                    preenche(
                        &mut resultado,
                        [a, b, c],
                        area,
                        &malha.material,
                        perspectiva,
                        params,
                        &mut stats,
                    );
                }
            }
        }

        resultado.stats = stats;
        resultado
    }
}

// Teoria: rasterizer.md#o-frustum
/// As seis faces do frustum, em espaço de câmera.
fn frustum(camera: &Camera) -> [Plane; 6] {
    let (near, far) = camera.clip_planes();

    let laterais = match *camera {
        Camera::Perspective { fov, aspect, .. } => {
            let meia_altura = (fov.to_radians() * 0.5).tan();
            let meia_largura = meia_altura * aspect;

            [
                Plane::new(Vec3::new(1.0, 0.0, meia_largura), 0.0),
                Plane::new(Vec3::new(-1.0, 0.0, meia_largura), 0.0),
                Plane::new(Vec3::new(0.0, 1.0, meia_altura), 0.0),
                Plane::new(Vec3::new(0.0, -1.0, meia_altura), 0.0),
            ]
        }
        Camera::Orthographic { width, height, .. } => [
            Plane::new(Vec3::new(1.0, 0.0, 0.0), width * 0.5),
            Plane::new(Vec3::new(-1.0, 0.0, 0.0), width * 0.5),
            Plane::new(Vec3::new(0.0, 1.0, 0.0), height * 0.5),
            Plane::new(Vec3::new(0.0, -1.0, 0.0), height * 0.5),
        ],
    };

    [
        Plane::new(Vec3::new(0.0, 0.0, 1.0), -near),
        Plane::new(Vec3::new(0.0, 0.0, -1.0), far),
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
fn recorta(triangulo: [Vertex; 3], planos: &[Plane; 6], stats: &mut RenderStats) -> Vec<Vertex> {
    let mut poligono = triangulo.to_vec();
    let mut recortou = false;

    for plano in planos {
        // Se todos os pontos do triângulo estão dentro do plano, nem precisa recortar.
        if poligono.iter().all(|v| plano.distance(v.position) >= 0.0) {
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

// Teoria: rasterizer.md#recorte-no-plano
/// Sutherland e Hodgman (1974): percorre as arestas do polígono e monta a lista
/// de saída com o que ficou do lado de dentro de um plano.
fn recorta_no_plano(poligono: &[Vertex], plano: Plane) -> Vec<Vertex> {
    let mut saida = Vec::with_capacity(poligono.len() + 1);

    for (i, &atual) in poligono.iter().enumerate() {
        let proximo = poligono[(i + 1) % poligono.len()];
        let (d_atual, d_proximo) = (
            plano.distance(atual.position),
            plano.distance(proximo.position),
        );

        if d_atual >= 0.0 {
            saida.push(atual);
        }

        if (d_atual >= 0.0) != (d_proximo >= 0.0) {
            // Um ponto dentro e o outro fora do plano: a aresta atravessou o plano, então entra também o ponto
            // onde ela cruza o plano.

            let t = -d_atual / (d_proximo - d_atual);

            // E agora a gente contrói o ponto com a fórmula de interpolação. Normal e uv
            // do vértice novo saem do mesmo t: o recorte inventa um vértice que não estava
            // na malha, e ele precisa carregar atributos coerentes com a aresta original.
            saida.push(atual.lerp(proximo, t));
        }
    }

    saida
}

// Teoria: rasterizer.md#mundo-para-camera-space
/// Mundo -> espaço de câmera. É onde o recorte acontece: nesse espaço o frustum
/// já está alinhado com a câmera, mas a divisão por z ainda não aconteceu.
fn para_camera(camera: &Camera, base: (Vec3, Vec3, Vec3), ponto: Vec3) -> Vec3 {
    let (direita, cima, frente) = base;

    let d = ponto - camera.position();
    Vec3::new(d.dot(direita), d.dot(cima), d.dot(frente))
}

// Teoria: rasterizer.md#camera-space-para-ndc
/// Espaço de câmera -> tela. A posição vira `(x_pixel, y_pixel, z_camera)`; normal
/// e uv atravessam intactas.
fn para_tela(camera: &Camera, vertice: Vertex, largura: u32, altura: u32) -> Option<Vertex> {
    let (x, y, z) = (vertice.position.x, vertice.position.y, vertice.position.z);

    let (near, _far) = camera.clip_planes();
    if z < near {
        return None;
    }

    // Converter o ponto de camera space pra ndc (retângulo de [-1,-1] -> [1,1])
    let (ndc_x, ndc_y) = match *camera {
        Camera::Perspective { fov, aspect, .. } => {
            let meia_altura = (fov.to_radians() * 0.5).tan();

            (x / (z * meia_altura * aspect), y / (z * meia_altura))
        }
        Camera::Orthographic { width, height, .. } => (x / (width * 0.5), y / (height * 0.5)),
    };

    let (pixel_x, pixel_y) = para_pixel(ndc_x, ndc_y, largura, altura);

    Some(Vertex {
        position: Vec3::new(
            pixel_x, pixel_y, z, // a posição z do ponto fica salva sem projeção
        ),
        ..vertice
    })
}

// Teoria: rasterizer.md#ndc-para-pixel
fn para_pixel(ndc_x: f32, ndc_y: f32, largura: u32, altura: u32) -> (f32, f32) {
    (
        (ndc_x + 1.0) * 0.5 * largura as f32,
        (1.0 - ndc_y) * 0.5 * altura as f32,
    )
}

// Teoria: rasterizer.md#edge-functions
/// A edge function de Pineda (1988).
fn borda(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    let u = b - a;
    let v = p - a;

    u.cross(v)
}

fn preenche(
    destino: &mut RenderResult,
    vertices: [Vertex; 3],
    area: f32,
    material: &Material,
    perspectiva: bool,
    params: &Parameters,
    stats: &mut RenderStats,
) {
    let (Some(cor), Some(profundidade), Some(mapa_normal), Some(mapa_uv)) = (
        destino.color.as_mut(),
        destino.depth.as_mut(),
        destino.normal.as_mut(),
        destino.uv.as_mut(),
    ) else {
        return;
    };

    let [a, b, c] = vertices.map(|v| v.position.xy());
    let profundidades = vertices.map(|v| v.position.z);
    let normais = vertices.map(|v| v.normal);
    let uvs = vertices.map(|v| v.uv);

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

            if !dentro {
                continue;
            }

            // Cada edge function dividida pela área do triângulo vira o peso do
            // vértice oposto àquela aresta: as coordenadas baricêntricas, que
            // somam 1 em qualquer ponto de dentro.

            let pesos_de_tela = [w0 / area, w1 / area, w2 / area];
            // Na ortográfica não existe distorção a corrigir, então o toggle só tem efeito sob
            // perspectiva.
            let baricentricas = if perspectiva && params.rasterization.perspective_correct {
                Baricentrics::perspective(pesos_de_tela, profundidades)
            } else {
                Baricentrics::orthographic(pesos_de_tela)
            };

            let z = baricentricas.interpolate_f32(profundidades);

            if params.rasterization.depth_test {
                let mais_proximo = profundidade.get(x, y).unwrap_or(f32::INFINITY);
                if z >= mais_proximo {
                    continue;
                }
            }

            // A normal interpolada entre três normais unitárias não sai unitária, então precisa normalizar aqui.
            // Considere n1 = (1,0), n2 = (0,1) e t = 0.5:
            // n' = n1 * (1 - t) + n2 * t = (0,5, 0,5)
            // |n'| = √(0,5² + 0,5²) = √(0,25 + 0,25) = √0,5 ≈ 0,7071
            let normal = baricentricas.interpolate_vec3(normais).normalized();

            let uv = baricentricas.interpolate_vec2(uvs);

            profundidade.set(x, y, z);
            cor.set(x, y, material.albedo);
            // Os componentes de uma normal vão de [-1,1], mas Rgb espera [0,1]. Então fazemos remap dos componentes da normal aqui:
            mapa_normal.set(
                x,
                y,
                Rgb::new(
                    normal.x * 0.5 + 0.5,
                    normal.y * 0.5 + 0.5,
                    normal.z * 0.5 + 0.5,
                ),
            );
            mapa_uv.set(x, y, Rgb::new(uv.x, uv.y, 0.0));
        }
    }
}
