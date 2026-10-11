//! Rasterizador.
//!
//! Teoria: rasterizer.md#a-pipeline
//!
//! O rasterizador trabalha só com malha: todo objeto da cena passa por
//! `Primitive::tessellate` na entrada.
//!
//! Ainda **não** existem aqui: wireframe, msaa e sombra projetada.

use crate::color::Rgb;
use crate::math::{Baricentrics, Plane, Vec2, Vec3, Vertex};
use crate::parameters::{Parameters, ShadingFrequency};
use crate::render::{ColorBuffer, RenderResult, RenderStats, Renderer, ScalarBuffer};
use crate::scene::{Camera, Light, Material, Primitive, Scene};
use crate::shading::{self, Surface};
use crate::texture::Footprint;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rasterizer;

impl Renderer for Rasterizer {
    fn name(&self) -> &'static str {
        "rasterizador"
    }

    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult {
        let (largura, altura) = (params.output.width, params.output.height);

        let fundo = params.output.background;

        let mut resultado = RenderResult {
            color: Some(ColorBuffer::filled(largura, altura, fundo)),
            depth: Some(ScalarBuffer::filled(largura, altura, f32::INFINITY)),
            normal: Some(ColorBuffer::filled(largura, altura, Rgb::BLACK)),
            uv: Some(ColorBuffer::filled(largura, altura, Rgb::BLACK)),
            ..RenderResult::new()
        };

        let mut stats = RenderStats::default();

        let (camera, lights, instances) = scene.flatten(params.transform.method);

        let Some(camera) = camera else {
            // Sem câmera não há de onde olhar: sobra o fundo.
            return resultado;
        };

        let planos = frustum(&camera);
        let perspectiva = matches!(camera, Camera::Perspective { .. });

        let contexto = Contexto {
            camera: &camera,
            lights: &lights,
            params,
            perspectiva,
        };

        for (id, instancia) in instances.iter().enumerate() {
            let malha = instancia.geometry.tessellate();

            for indice in 0..malha.face_count() {
                let Some(face) = malha.face(indice) else {
                    continue;
                };

                if let Some(on_triangle) = params.rasterization.on_triangle {
                    if let Primitive::Mesh(_) = instancia.geometry {
                        on_triangle(id, indice);
                    }
                }

                // Local -> mundo -> câmera. Aqui o rasterizador transforma a
                // *geometria*; o ray tracer resolve o mesmo problema no sentido
                // contrário, transformando o raio.
                let triangulo = face.map(|v| Vertex {
                    position: camera.to_camera_space(
                        instancia
                            .chain
                            .apply_point(v.position, params.transform.method),
                    ),
                    normal: instancia
                        .chain
                        .apply_normal(v.normal, params.transform.method),
                    tangent: instancia
                        .chain
                        .apply_tangent(v.tangent, params.transform.method),
                    uv: v.uv,
                });

                let poligono = recorta(triangulo, &planos, &mut stats);

                // O recorte devolve um polígono convexo de N vértices; desenhar com triangle fan.
                for i in 1..poligono.len().saturating_sub(1) {
                    let recortado = [poligono[0], poligono[i], poligono[i + 1]];
                    let tela = recortado.map(|v| {
                        Some(Vertex {
                            position: camera.to_screen_space(v.position, largura, altura)?,
                            ..v
                        })
                    });

                    let [Some(a), Some(b), Some(c)] = tela else {
                        // O clipping já garante z >= near; não deve entrar aqui.
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

                    // O shading acontece no espaço de mundo.
                    let mundo = recortado.map(|v| camera.from_camera_space(v.position));

                    let shading = contexto.shade_vertex(
                        instancia.material,
                        mundo,
                        recortado.map(|v| v.normal),
                        recortado.map(|v| v.uv),
                    );

                    preenche(
                        &mut resultado,
                        &Triangulo {
                            tela: [a, b, c],
                            mundo,
                            area,
                            shading,
                            material: instancia.material,
                        },
                        &contexto,
                        &mut stats,
                    );
                }
            }
        }

        if let Some(cor) = &mut resultado.color {
            cor.apply_output(&params.output);
        }

        resultado.stats = stats;
        resultado
    }
}

/// O que o shading precisa saber da cena e que não muda de um triângulo para o
/// outro.
struct Contexto<'a> {
    camera: &'a Camera,
    lights: &'a [Light],
    params: &'a Parameters,
    perspectiva: bool,
}

impl Contexto<'_> {
    fn shade_fragment(
        &self,
        triangulo: &Triangulo,
        baricentricas: &Baricentrics,
        normal: Vec3,
        uv: Vec2,
        footprint: Footprint,
    ) -> Rgb {
        match triangulo.shading {
            Shading::Face(cor) => cor,
            Shading::Vertices(cores) => baricentricas.interpolate_rgb(cores),
            Shading::Pixel => {
                let superficie = Surface {
                    point: baricentricas.interpolate_vec3(triangulo.mundo),
                    normal,
                    uv,
                };

                let filtering = self.params.shading.filtering;
                let amostra = triangulo.material.sample(uv, Some(footprint), filtering);

                shading::shade(
                    &superficie,
                    &amostra,
                    self.lights,
                    self.camera.view_from(superficie.point),
                    self.params.shading.reflectance,
                    &shading::always_visible,
                )
            }
        }
    }

    fn shade_vertex(
        &self,
        material: &Material,
        mundo: [Vec3; 3],
        normais: [Vec3; 3],
        uvs: [Vec2; 3],
    ) -> Shading {
        match self.params.shading.frequency {
            ShadingFrequency::Flat => {
                let normal = (mundo[1] - mundo[0])
                    .cross(mundo[2] - mundo[0])
                    .normalized();
                let centro = (mundo[0] + mundo[1] + mundo[2]) / 3.0;
                let uv = (uvs[0] + uvs[1] + uvs[2]) / 3.0;

                Shading::Face({
                    let superficie = Surface {
                        point: centro,
                        normal,
                        uv,
                    };
                    let amostra = material.sample(uv, None, self.params.shading.filtering);
                    shading::shade(
                        &superficie,
                        &amostra,
                        self.lights,
                        self.camera.view_from(centro),
                        self.params.shading.reflectance,
                        &shading::always_visible,
                    )
                })
            }
            ShadingFrequency::Gouraud => Shading::Vertices([0, 1, 2].map(|k| {
                let ponto = mundo[k];
                let normal = normais[k];
                let uv = uvs[k];

                let superficie = Surface {
                    point: ponto,
                    normal,
                    uv,
                };
                let amostra = material.sample(uv, None, self.params.shading.filtering);
                shading::shade(
                    &superficie,
                    &amostra,
                    self.lights,
                    self.camera.view_from(ponto),
                    self.params.shading.reflectance,
                    &shading::always_visible,
                )
            })),
            ShadingFrequency::Phong => Shading::Pixel,
        }
    }
}

/// O que sobrou do modelo de shading depois de resolver o triângulo: a cor já
/// pronta, ou o aviso de que ela ainda vai ser calculada em cada pixel.
enum Shading {
    Face(Rgb),
    Vertices([Rgb; 3]),
    Pixel,
}

struct Triangulo<'a> {
    /// Posição em pixels, com o z de câmera guardado sem projeção.
    tela: [Vertex; 3],
    /// As mesmas três posições, em espaço de mundo, para o shading.
    mundo: [Vec3; 3],
    /// Área com sinal em espaço de tela: o sinal diz a orientação.
    area: f32,
    shading: Shading,
    material: &'a Material,
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

// Teoria: rasterizer.md#edge-functions
/// A edge function de Pineda (1988).
fn borda(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    let u = b - a;
    let v = p - a;

    u.cross(v)
}

fn preenche(
    destino: &mut RenderResult,
    triangulo: &Triangulo,
    contexto: &Contexto,
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

    let params = contexto.params;

    let [a, b, c] = triangulo.tela.map(|v| v.position.xy());
    let profundidades = triangulo.tela.map(|v| v.position.z);
    let normais = triangulo.tela.map(|v| v.normal);
    let tangentes = triangulo.tela.map(|v| v.tangent);
    let uvs = triangulo.tela.map(|v| v.uv);
    let area = triangulo.area;

    let (largura, altura) = (cor.width() as f32, cor.height() as f32);

    // Calcular a bounding box do triângulo
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as u32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(largura - 1.0).max(0.0) as u32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as u32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min(altura - 1.0).max(0.0) as u32;

    // Cada edge function dividida pela área do triângulo vira o peso do
    // vértice oposto àquela aresta: as coordenadas baricêntricas, que
    // somam 1 em qualquer ponto de dentro.
    let baricentricas_de = |arestas: [f32; 3]| {
        let pesos_de_tela = arestas.map(|w| w / area);

        if contexto.perspectiva && params.rasterization.perspective_correct {
            Baricentrics::perspective(pesos_de_tela, profundidades)
        } else {
            Baricentrics::orthographic(pesos_de_tela)
        }
    };

    let uv_em = |p: Vec2| {
        baricentricas_de([borda(b, c, p), borda(c, a, p), borda(a, b, p)]).interpolate_vec2(uvs)
    };

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            stats.primitive_tests += 1;

            // Usar o centro do pixel. Pixel (1,1) tem centro 1.5,1.5 por exemplo
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let [w0, w1, w2] = [borda(b, c, p), borda(c, a, p), borda(a, b, p)];

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

            if let Some(on_fragment) = params.rasterization.on_fragment {
                on_fragment(x, y);
            }

            let baricentricas = baricentricas_de([w0, w1, w2]);

            let z = baricentricas.interpolate_f32(profundidades);

            if params.rasterization.depth_test {
                let mais_proximo = profundidade.get(x, y).unwrap_or(f32::INFINITY);
                if z >= mais_proximo {
                    continue;
                }
            }

            // Teoria: triangulo.md#a-normal-interpolada-não-é-unitária
            // A normal interpolada entre três normais unitárias não sai unitária.
            let normal = baricentricas.interpolate_vec3(normais).normalized();

            let uv = baricentricas.interpolate_vec2(uvs);

            // Teoria: texturas.md#a-footprint-do-pixel
            // O pixel vizinho é a próxima iteração do laço, então a derivada da uv sai
            // por diferença finita.
            let footprint = Footprint {
                dx: uv_em(Vec2::new(p.x + 1.0, p.y)) - uv,
                dy: uv_em(Vec2::new(p.x, p.y + 1.0)) - uv,
            };

            // Aplicar os desvios do normal_map se tiver.
            let normal = triangulo.material.shading_normal(
                uv,
                Some(footprint),
                params.shading.filtering,
                normal,
                baricentricas.interpolate_tangent(tangentes),
            );

            profundidade.set(x, y, z);
            cor.set(
                x,
                y,
                contexto.shade_fragment(triangulo, &baricentricas, normal, uv, footprint),
            );
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
