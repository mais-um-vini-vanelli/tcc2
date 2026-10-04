//! Descrição da cena: câmera, luzes e objetos.

use crate::color::Rgb;
use crate::math::{Aabb, Baricentrics, Ray, Tangent, Vec2, Vec3, Vertex};
use crate::parameters::{Filtering, TransformMethod};
use crate::texture::{Footprint, Texture};
use crate::transform::{Chain, Transform};

#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            objects: vec![Object::camera(camera)],
        }
    }

    pub fn with_light(mut self, light: Light) -> Self {
        self.objects.push(Object::light(light));
        self
    }

    pub fn with_object(mut self, object: Object) -> Self {
        self.objects.push(object);
        self
    }

    /// Percorre a árvore e devolve o que os renderizadores consomem: a câmera, as
    /// luzes e as instâncias.
    pub fn flatten(
        &self,
        method: TransformMethod,
    ) -> (Option<Camera>, Vec<Light>, Vec<Instance<'_>>) {
        let mut saida = (None, Vec::new(), Vec::new());
        let mut cadeia = Vec::new();

        for raiz in &self.objects {
            Self::desce(raiz, &mut cadeia, method, &mut saida);
        }

        saida
    }

    fn desce<'a>(
        no: &'a Object,
        cadeia: &mut Vec<Transform>,
        method: TransformMethod,
        saida: &mut (Option<Camera>, Vec<Light>, Vec<Instance<'a>>),
    ) {
        let (camera, lights, instances) = &mut *saida;

        cadeia.push(no.transform);

        match &no.content {
            None => {}

            Some(Content::Geometry {
                geometry,
                material,
                casts_shadows,
            }) => instances.push(Instance {
                geometry,
                material,
                casts_shadows: *casts_shadows,
                chain: Chain::from_root_first(cadeia),
            }),

            Some(Content::Camera(local)) => {
                let chain = Chain::from_root_first(cadeia);
                camera.get_or_insert_with(|| local.transformed(&chain, method));
            }

            Some(Content::Light(luz)) => {
                let chain = Chain::from_root_first(cadeia);
                lights.push(luz.transformed(&chain, method));
            }
        }

        for filho in &no.children {
            Self::desce(filho, cadeia, method, saida);
        }

        cadeia.pop();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instance<'a> {
    pub geometry: &'a Primitive,
    pub material: &'a Material,
    pub casts_shadows: bool,
    pub chain: Chain,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Camera(Camera),
    Light(Light),
    Geometry {
        geometry: Primitive,
        material: Material,
        casts_shadows: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Object {
    /// O transform deste elo, relativo ao pai.
    pub transform: Transform,
    pub content: Option<Content>,
    pub children: Vec<Object>,
}

impl Object {
    pub fn new(geometry: Primitive, material: Material) -> Self {
        Object {
            content: Some(Content::Geometry {
                geometry,
                material,
                casts_shadows: true,
            }),
            ..Object::group()
        }
    }

    /// Nó sem conteúdo: existe pelo transform, que vale pra árvore toda
    /// pendurada nele.
    pub fn group() -> Self {
        Object::default()
    }

    pub fn camera(camera: Camera) -> Self {
        Object {
            content: Some(Content::Camera(camera)),
            ..Object::group()
        }
    }

    pub fn light(light: Light) -> Self {
        Object {
            content: Some(Content::Light(light)),
            ..Object::group()
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Sem efeito num nó que não é geometria.
    pub fn with_shadows(mut self, casts_shadows: bool) -> Self {
        if let Some(Content::Geometry {
            casts_shadows: c, ..
        }) = &mut self.content
        {
            *c = casts_shadows;
        }
        self
    }

    pub fn with_child(mut self, child: Object) -> Self {
        self.children.push(child);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Camera {
    Perspective {
        /// Campo de visão vertical, em graus.
        fov: f32,
        /// Largura / altura do plano de imagem.
        aspect: f32,
        near: f32,
        far: f32,
        position: Vec3,
        look_at: Vec3,
        up: Vec3,
    },
    Orthographic {
        /// Largura do volume de visão, em unidades.
        width: f32,
        /// Altura do volume de visão, em unidades.
        height: f32,
        near: f32,
        far: f32,
        position: Vec3,
        look_at: Vec3,
        up: Vec3,
    },
}

impl Camera {
    pub fn position(&self) -> Vec3 {
        match *self {
            Camera::Perspective { position, .. } | Camera::Orthographic { position, .. } => {
                position
            }
        }
    }

    pub fn look_at(&self) -> Vec3 {
        match *self {
            Camera::Perspective { look_at, .. } | Camera::Orthographic { look_at, .. } => look_at,
        }
    }

    pub fn up(&self) -> Vec3 {
        match *self {
            Camera::Perspective { up, .. } | Camera::Orthographic { up, .. } => up,
        }
    }

    pub fn clip_planes(&self) -> (f32, f32) {
        match *self {
            Camera::Perspective { near, far, .. } | Camera::Orthographic { near, far, .. } => {
                (near, far)
            }
        }
    }

    /// Base da câmera `(direita, cima, frente)`, em espaço de mundo.
    ///
    /// O up informado pelo usuário não precisa ser exatamente perpendicular à
    /// direção de visão, porque ele é recalculado aqui.
    pub fn basis(&self) -> (Vec3, Vec3, Vec3) {
        let forward = (self.look_at() - self.position()).normalized();
        let right = forward.cross(self.up()).normalized();
        let up = right.cross(forward);

        (right, up, forward)
    }

    // Teoria: rasterizer.md#mundo-para-camera-space
    pub fn to_camera_space(&self, ponto: Vec3) -> Vec3 {
        let (direita, cima, frente) = self.basis();

        let d = ponto - self.position();
        Vec3::new(d.dot(direita), d.dot(cima), d.dot(frente))
    }

    /// Câmera -> mundo, a inversa de `to_camera_space`.
    pub fn from_camera_space(&self, p: Vec3) -> Vec3 {
        let (direita, cima, frente) = self.basis();

        self.position() + direita * p.x + cima * p.y + frente * p.z
    }

    // Teoria: rasterizer.md#camera-space-para-ndc
    /// Espaço de câmera -> tela: `(x_pixel, y_pixel, z_camera)`, com o z
    /// guardado sem projeção.
    pub fn to_screen_space(&self, ponto: Vec3, largura: u32, altura: u32) -> Option<Vec3> {
        let (x, y, z) = (ponto.x, ponto.y, ponto.z);

        let (near, _far) = self.clip_planes();
        if z < near {
            return None;
        }

        // Converter o ponto de camera space pra ndc (retângulo de [-1,-1] -> [1,1])
        let (ndc_x, ndc_y) = match *self {
            Camera::Perspective { fov, aspect, .. } => {
                let meia_altura = (fov.to_radians() * 0.5).tan();

                (x / (z * meia_altura * aspect), y / (z * meia_altura))
            }
            Camera::Orthographic { width, height, .. } => (x / (width * 0.5), y / (height * 0.5)),
        };

        // Teoria: rasterizer.md#ndc-para-pixel
        Some(Vec3::new(
            (ndc_x + 1.0) * 0.5 * largura as f32,
            (1.0 - ndc_y) * 0.5 * altura as f32,
            z,
        ))
    }

    /// A câmera com `position`/`look_at`/`up` levados pro mundo pelos transforms.
    pub fn transformed(&self, chain: &Chain, method: TransformMethod) -> Camera {
        match *self {
            Camera::Perspective {
                fov,
                aspect,
                near,
                far,
                position,
                look_at,
                up,
            } => Camera::Perspective {
                fov,
                aspect,
                near,
                far,
                position: (|p| chain.apply_point(p, method))(position),
                look_at: (|p| chain.apply_point(p, method))(look_at),
                up: (|d| chain.apply_direction(d, method))(up),
            },

            Camera::Orthographic {
                width,
                height,
                near,
                far,
                position,
                look_at,
                up,
            } => Camera::Orthographic {
                width,
                height,
                near,
                far,
                position: (|p| chain.apply_point(p, method))(position),
                look_at: (|p| chain.apply_point(p, method))(look_at),
                up: (|d| chain.apply_direction(d, method))(up),
            },
        }
    }

    /// Direção do ponto **para** a câmera.
    ///
    /// Sob perspectiva todos os raios de visão convergem na posição da câmera;
    /// sob ortográfica eles são paralelos, e a direção não depende do ponto.
    pub fn view_from(&self, point: Vec3) -> Vec3 {
        match self {
            Camera::Perspective { position, .. } => (*position - point).normalized(),
            Camera::Orthographic { .. } => -self.basis().2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// Luz pontual: irradia em todas as direções e atenua com o quadrado da
    /// distância.
    Point {
        position: Vec3,
        color: Rgb,
        intensity: f32,
    },
    /// Luz direcional: raios paralelos, sem posição e sem atenuação.
    Directional {
        direction: Vec3,
        color: Rgb,
        intensity: f32,
    },
    /// Luz pontual restrita a um cone.
    Spot {
        position: Vec3,
        /// Eixo do cone, saindo da luz.
        direction: Vec3,
        /// Meio-ângulo do cone interno, em graus.
        inner_angle: f32,
        /// Meio-ângulo do cone externo, em graus.
        outer_angle: f32,
        color: Rgb,
        intensity: f32,
    },
    /// Luz ambiente
    Ambient { color: Rgb, intensity: f32 },
}

impl Light {
    pub fn color(&self) -> Rgb {
        match *self {
            Light::Point { color, .. }
            | Light::Directional { color, .. }
            | Light::Spot { color, .. }
            | Light::Ambient { color, .. } => color,
        }
    }

    pub fn intensity(&self) -> f32 {
        match *self {
            Light::Point { intensity, .. }
            | Light::Directional { intensity, .. }
            | Light::Spot { intensity, .. }
            | Light::Ambient { intensity, .. } => intensity,
        }
    }

    /// A luz com `position`/`direction` levados pro mundo pelos transforms.
    pub fn transformed(&self, chain: &Chain, method: TransformMethod) -> Light {
        match *self {
            Light::Point {
                position,
                color,
                intensity,
            } => Light::Point {
                position: (|p| chain.apply_point(p, method))(position),
                color,
                intensity,
            },

            Light::Directional {
                direction,
                color,
                intensity,
            } => Light::Directional {
                direction: (|d| chain.apply_direction(d, method).normalized())(direction),
                color,
                intensity,
            },

            Light::Spot {
                position,
                direction,
                inner_angle,
                outer_angle,
                color,
                intensity,
            } => Light::Spot {
                position: (|p| chain.apply_point(p, method))(position),
                direction: (|d| chain.apply_direction(d, method).normalized())(direction),
                inner_angle,
                outer_angle,
                color,
                intensity,
            },

            Light::Ambient { .. } => *self,
        }
    }
}

/// Um canal do material: ou um valor constante, ou uma textura.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<T> {
    Constant(T),
    Map(Texture),
}

impl Value<Rgb> {
    pub fn sample(&self, uv: Vec2, footprint: Option<Footprint>, filtering: Filtering) -> Rgb {
        match self {
            Value::Constant(c) => *c,
            Value::Map(textura) => textura.sample(uv, footprint, filtering),
        }
    }
}

impl Value<f32> {
    /// Mapa de dado (roughness, metallic) é cinza: os três canais são iguais,
    /// então basta ler um.
    pub fn sample(&self, uv: Vec2, footprint: Option<Footprint>, filtering: Filtering) -> f32 {
        match self {
            Value::Constant(v) => *v,
            Value::Map(textura) => textura.sample(uv, footprint, filtering).r,
        }
    }
}

/// Propriedades de superfície.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Cor base. Em metal, é também a cor do reflexo especular.
    pub albedo: Value<Rgb>,
    /// 0 dielétrico, 1 metal. Na prática só 0 ou 1 — o meio do caminho não
    /// existe como material real.
    pub metallic: Value<f32>,
    /// 0 espelho, 1 difuso. É daqui que sai o expoente do especular.
    pub roughness: Value<f32>,
    // Teoria: shading.md#o-termo-emitido
    /// Luz que a superfície emite por conta própria, sem depender de luzes externas.
    pub emissive: Value<Rgb>,
    /// Mapa de normais, ou `None` pra deixar a normal da geometria como ela
    /// chegou.
    ///
    /// Cada texel guarda um vetor unitário em **espaço tangente** — a base `(T, B, N)` que
    /// acompanha a superfície.
    pub normal: Option<Texture>,

    // Modelo de Whitted (M3), ortogonais ao acima.
    /// Do que **entra** na superfície, quanto atravessa em vez de voltar
    /// espalhado: vidro ~1, tinta 0.
    pub transparency: f32,
    /// Índice de refração, usado quando `transparency > 0` (vidro ~ 1.5).
    pub refractive_index: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            albedo: Value::Constant(Rgb::gray(0.8)),
            metallic: Value::Constant(0.0),
            roughness: Value::Constant(0.5),
            emissive: Value::Constant(Rgb::BLACK),
            normal: None,
            transparency: 0.0,
            refractive_index: 1.5,
        }
    }
}

impl Material {
    /// Material difuso puro da cor dada: dielétrico e áspero.
    pub fn diffuse(albedo: Rgb) -> Self {
        Material {
            albedo: Value::Constant(albedo),
            roughness: Value::Constant(1.0),
            ..Material::default()
        }
    }

    /// Metal da cor dada. O albedo vira a cor do
    /// próprio reflexo.
    pub fn metal(albedo: Rgb, roughness: f32) -> Self {
        Material {
            albedo: Value::Constant(albedo),
            metallic: Value::Constant(1.0),
            roughness: Value::Constant(roughness),
            ..Material::default()
        }
    }

    /// Superfície que brilha com a cor dada e não reflete nada.
    ///
    /// Ela ilumina só a si mesma: aparece acesa na imagem e nos reflexos, mas
    /// não joga luz nos vizinhos.
    pub fn emissive(color: Rgb) -> Self {
        Material {
            albedo: Value::Constant(Rgb::BLACK),
            emissive: Value::Constant(color),
            ..Material::default()
        }
    }

    /// Material com textura difusa.
    pub fn textured(texture: Texture) -> Self {
        Material {
            albedo: Value::Map(texture),
            roughness: Value::Constant(1.0),
            ..Material::default()
        }
    }

    /// O material no ponto `uv`. A `footprint` é o que o pipeline sabe do pixel;
    /// sem ela os mapas são lidos no nível 0.
    pub fn sample(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        filtering: Filtering,
    ) -> MaterialSample {
        MaterialSample {
            albedo: self.albedo.sample(uv, footprint, filtering),
            metallic: self
                .metallic
                .sample(uv, footprint, filtering)
                .clamp(0.0, 1.0),
            // Roughness zero concentra o especular num delta que nenhuma
            // amostragem de pixel pega, e o highlight some.
            roughness: self
                .roughness
                .sample(uv, footprint, filtering)
                .clamp(Self::MIN_ROUGHNESS, 1.0),
            emission: self.emissive.sample(uv, footprint, filtering),
            transparency: self.transparency.clamp(0.0, 1.0),
            // Abaixo de 1 seria o material sendo o meio *menos* denso.
            refractive_index: self.refractive_index.max(1.0),
        }
    }

    /// A normal que o shading usa no ponto: a da geometria, torcida pelo mapa
    /// de normais quando existe um.
    ///
    /// `normal` e `tangent` chegam em espaço de mundo e já interpolados. A base
    /// TBN é montada aqui — `T` no eixo do u, `B` no eixo do v, `N` pra fora —
    /// e é ela que leva o vetor do mapa do espaço tangente de volta pro mundo:
    pub fn shading_normal(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        filtering: Filtering,
        normal: Vec3,
        tangent: Tangent,
    ) -> Vec3 {
        if self.normal.is_none() {
            return normal;
        }

        let tangent = {
            // Gram-Schmidt: tiramos de T a componente que está ao longo de N (normal * normal.dot(tangent.vector)) e renormalizamos.
            let vector = (tangent.vector - normal * normal.dot(tangent.vector)).normalized();

            if vector == Vec3::ZERO {
                // Se a tangente e a normal são paralelas, escolher uma tangente qualquer que seja perpendicular a N.
                Tangent::new(
                    Tangent::any_perpendicular(normal).vector,
                    tangent.handedness,
                )
            } else {
                Tangent {
                    vector,
                    handedness: tangent.handedness,
                }
            }
        };

        let texel = self
            .normal
            .as_ref()
            .unwrap()
            .sample(uv, footprint, filtering);

        // A textura guarda valores em [0,1], mas uma normal tem componentes em [-1,1]; precisamos converter.
        let n = Vec3::new(
            texel.r * 2.0 - 1.0,
            texel.g * 2.0 - 1.0,
            texel.b * 2.0 - 1.0,
        )
        .normalized();

        // Agora convertemos a normal do normal_map de espaço tangente (TBN) para espaço de mundo, multiplicando os componentes da normal
        // pela base: (T * n.x + B * n.y + N * n.z).normalized()
        let bitangente = normal.cross(tangent.vector) * tangent.handedness;
        (tangent.vector * n.x + bitangente * n.y + normal * n.z).normalized()
    }

    const MIN_ROUGHNESS: f32 = 0.045;
}

/// O material já resolvido num ponto.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialSample {
    pub albedo: Rgb,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Rgb,
    pub transparency: f32,
    pub refractive_index: f32,
}

/// Malha indexada de triângulos: é o formato que o rasterizador consome, e o
/// que todo primitivo vira depois de `Primitive::tessellate`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    /// Índices em trios: cada três entradas formam um triângulo.
    pub indices: Vec<u32>,
    /// Normais por vértice. Se None, cada face recebe uma normal calculada a
    /// partir da geometria.
    pub normals: Option<Vec<Vec3>>,
    /// Tangentes por vértice. Só são usadas no sampling do mapa de normais; Se None, a
    /// base do espaço tangente sai da uv do próprio triângulo.
    pub tangents: Option<Vec<Tangent>>,
    pub uvs: Option<Vec<Vec2>>,
}

impl Mesh {
    /// Malha sem normais nem uv.
    pub fn new(vertices: Vec<Vec3>, indices: Vec<u32>) -> Self {
        Mesh {
            vertices,
            indices,
            normals: None,
            tangents: None,
            uvs: None,
        }
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// A face de índice `i`, com os atributos já resolvidos por vértice.
    ///
    /// Índice fora da lista descarta a face inteira em vez de entrar em pânico.
    pub fn face(&self, i: usize) -> Option<[Vertex; 3]> {
        let trio = [
            *self.indices.get(i * 3)? as usize,
            *self.indices.get(i * 3 + 1)? as usize,
            *self.indices.get(i * 3 + 2)? as usize,
        ];

        fn tres<T: Copy>(valores: &[T], indices: [usize; 3]) -> Option<[T; 3]> {
            Some([
                *valores.get(indices[0])?,
                *valores.get(indices[1])?,
                *valores.get(indices[2])?,
            ])
        }

        let pontos = tres(&self.vertices, trio)?;

        Some(monta_triangulo(
            pontos,
            self.normals.as_deref().and_then(|n| tres(n, trio)),
            self.tangents.as_deref().and_then(|t| tres(t, trio)),
            self.uvs.as_deref().and_then(|u| tres(u, trio)),
        ))
    }
}

/// O que um raio encontra num primitivo, em espaço local.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub t: f32,
    /// A normal do shading: interpolada entre os vértices nos triângulos.
    pub normal: Vec3,
    /// A normal da superfície de verdade, usada pro deslocamento dos raios secundários precisa.
    pub geometric_normal: Vec3,
    pub tangent: Tangent,
    pub uv: Vec2,
}

/// Geometria renderizável, em espaço local.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Triangle {
        points: [Vec3; 3],
        /// Normais por vértice.
        normals: Option<[Vec3; 3]>,
        /// Tangentes por vértice.
        tangents: Option<[Tangent; 3]>,
        uvs: Option<[Vec2; 3]>,
    },
    Mesh(Mesh),
    Sphere {
        center: Vec3,
        radius: f32,
    },
    Cube {
        center: Vec3,
        half_edge: f32,
    },
}

impl Primitive {
    /// Triângulo sem normais nem uv
    pub fn triangle(points: [Vec3; 3]) -> Self {
        Primitive::Triangle {
            points,
            normals: None,
            tangents: None,
            uvs: None,
        }
    }

    /// Malha sem normais nem uv.
    /// As faces são trios consecutivos de `indices`, cada índice apontando pra uma entrada de `vertices`.
    pub fn mesh(vertices: Vec<Vec3>, indices: Vec<u32>) -> Self {
        Primitive::Mesh(Mesh::new(vertices, indices))
    }

    pub fn sphere(center: Vec3, radius: f32) -> Self {
        Primitive::Sphere { center, radius }
    }

    /// Cubo centrado em `center`, com aresta `2 * half_edge`.
    pub fn cube(center: Vec3, half_edge: f32) -> Self {
        Primitive::Cube { center, half_edge }
    }

    /// Converte o primitivo em malha de triângulos.
    pub fn tessellate(&self) -> Mesh {
        match self {
            Primitive::Triangle {
                points,
                normals,
                tangents,
                uvs,
            } => Mesh {
                vertices: points.to_vec(),
                indices: vec![0, 1, 2],
                normals: normals.map(|n| n.to_vec()),
                tangents: tangents.map(|t| t.to_vec()),
                uvs: uvs.map(|u| u.to_vec()),
            },

            Primitive::Mesh(malha) => malha.clone(),

            Primitive::Sphere { center, radius } => Self::esfera_uv(*center, *radius),

            Primitive::Cube { center, half_edge } => Self::cubo(*center, *half_edge),
        }
    }

    // Teoria: raytracer.md#o-raio-e-a-esfera
    /// Interseção do raio com o primitivo, em espaço local — o par de
    /// `tessellate` do lado do ray tracer: um resolve a superfície por
    /// aproximação em triângulos, o outro pela equação exata.
    ///
    /// A BVH do ray tracer entra por cima disto, sem mudar a assinatura: ela
    /// escolhe *quais* primitivos testar, não como.
    pub fn intersect(&self, ray: Ray) -> Option<Hit> {
        match self {
            Primitive::Sphere { center, radius } => {
                let t = intersecao_esfera(ray, *center, *radius)?;

                let normal = (ray.at(t) - *center).normalized();

                Some(Hit {
                    t,
                    normal,
                    geometric_normal: normal,
                    tangent: tangente_da_esfera(normal),
                    uv: uv_da_esfera(normal),
                })
            }

            Primitive::Triangle {
                points,
                normals,
                tangents,
                uvs,
            } => intersecao_triangulo(ray, monta_triangulo(*points, *normals, *tangents, *uvs)),

            Primitive::Mesh(malha) => (0..malha.face_count())
                .filter_map(|i| intersecao_triangulo(ray, malha.face(i)?))
                .reduce(mais_perto),

            Primitive::Cube { center, half_edge } => intersecao_cubo(ray, *center, *half_edge),
        }
    }

    /// A AABB que contém o primitivo, em espaço local.
    pub fn bounds(&self) -> Aabb {
        match self {
            Primitive::Triangle { points, .. } => Aabb::around(*points),
            Primitive::Mesh(malha) => Aabb::around(malha.vertices.iter().copied()),

            Primitive::Sphere { center, radius } => {
                let meio = Vec3::ONE * radius.abs();
                Aabb {
                    min: *center - meio,
                    max: *center + meio,
                }
            }

            Primitive::Cube { center, half_edge } => {
                let meio = Vec3::ONE * half_edge.abs();
                Aabb {
                    min: *center - meio,
                    max: *center + meio,
                }
            }
        }
    }

    /// Quatro vértices por face, em vez dos oito cantos compartilhados: cada face
    /// precisa da própria normal e da própria uv no mesmo canto.
    fn cubo(center: Vec3, half_edge: f32) -> Mesh {
        let h = half_edge.abs();

        let mut vertices = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut tangents = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Cada face: a normal, e pra onde apontam a direita e o "pra baixo" de quem
        // olha a face de fora. A textura segue essas direções, como no `intersect`.
        #[rustfmt::skip]
        let faces = [
            (Vec3::new( 1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0, -1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0,  1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0,  1.0)),
            (Vec3::new( 0.0,  0.0,  1.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new(-1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0,  1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0, -1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0, -1.0)),
            (Vec3::new( 0.0,  0.0, -1.0), Vec3::new(-1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        ];

        for (normal, direita, baixo) in faces {
            let primeiro = vertices.len() as u32;

            // Os cantos, vistos de fora: 0 em cima à esquerda, 1 em cima à direita,
            // 2 embaixo à direita, 3 embaixo à esquerda.
            let cantos = [
                normal - direita - baixo,
                normal + direita - baixo,
                normal + direita + baixo,
                normal - direita + baixo,
            ];

            let uvs_dos_cantos = [
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ];

            // A uv da face anda pra `direita` quando o u cresce e pra `baixo`
            // quando o v cresce.
            let tangente = Tangent::from_uv_axes(normal, direita, baixo);

            for (canto, uv) in cantos.into_iter().zip(uvs_dos_cantos) {
                vertices.push(center + canto * h);
                normals.push(normal);
                tangents.push(tangente);
                uvs.push(uv);
            }

            // 0→1→2 gira no sentido horário visto de fora; a face da frente é a
            // anti-horária, 0→3→2 e 0→2→1.
            indices.extend([0, 3, 2, 0, 2, 1].map(|k| primeiro + k));
        }

        Mesh {
            vertices,
            indices,
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
        }
    }

    /// Tesselation em grade de paralelos e meridianos ("uv sphere").
    fn esfera_uv(center: Vec3, radius: f32) -> Mesh {
        /// Paralelos da esfera tesselada, com o dobro disso em meridianos.
        const RINGS: u32 = 16;

        let paralelos = RINGS.max(2);
        let meridianos = paralelos * 2;
        let por_linha = meridianos + 1;

        let total = (paralelos + 1) as usize * por_linha as usize;
        let mut vertices = Vec::with_capacity(total);
        let mut normals = Vec::with_capacity(total);
        let mut tangents = Vec::with_capacity(total);
        let mut uvs = Vec::with_capacity(total);

        for i in 0..=paralelos {
            // theta desce do polo norte (0) ao sul (pi); phi dá a volta no equador.
            let u = i as f32 / paralelos as f32;

            let (sin_theta, cos_theta) = (u * std::f32::consts::PI).sin_cos();

            for j in 0..=meridianos {
                let v = j as f32 / meridianos as f32;

                let (sin_phi, cos_phi) = (v * std::f32::consts::TAU).sin_cos();

                // Na esfera unitária centrada na origem, a posição do ponto e a
                // normal ali são o mesmo vetor.
                // A visualização disso está em teoria/figuras/esfera_uv.html
                let r = sin_theta;
                let normal = Vec3::new(r * cos_phi, cos_theta, r * sin_phi);

                // Agora a gente só precisa fazer translate e scale pra fechar com a posição e raio que o usuário pediu.
                vertices.push(center + normal * radius);
                normals.push(normal);
                tangents.push(tangente_da_esfera(normal));
                uvs.push(Vec2::new(v, u));

                // Note que nos polos (theta = 0, theta = PI) e na emenda dos meridianos (phi = TAU) a gente tem vários vértices na
                // mesma posição. Isso é preciso pra permitir uvs diferentes pra esses vértices e não ter problema na hora de aplicar
                // texturas.
            }
        }

        let mut indices = Vec::with_capacity((paralelos * meridianos * 6) as usize);

        for i in 0..paralelos {
            for j in 0..meridianos {
                // Os 4 pontos do quadrado atual (a célula de paralelo P x meridiano M)
                let a = i * por_linha + j;
                let b = a + 1;
                let c = a + por_linha;
                let d = c + 1;

                // Anti-horário visto de fora, como o cubo.
                //
                // Na fileira i = 0, a e b estão os dois na linha 0, que é θ = 0: o polo norte. Todos os vértices dessa linha ocupam
                // a mesma posição no espaço. Então:
                //    a = b   ← o mesmo ponto
                //     /\
                //    /  \
                //   c————d
                // Mesma coisa pra i = paralelos - 1, mas com c = d
                if i != 0 {
                    indices.extend_from_slice(&[a, b, c]);
                }
                if i != paralelos - 1 {
                    indices.extend_from_slice(&[b, d, c]);
                }
            }
        }

        Mesh {
            vertices,
            indices,
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
        }
    }
}

/// A uv do ponto de coordenadas `(a, b)` sobre uma face do cubo, em meias
/// arestas: [-1, 1] vira [0, 1].
///
/// Olhando a face de fora, `a` cresce pra direita e `b` pra baixo, como nas
/// imagens. Nas faces laterais o topo da textura fica pra cima (+y).
fn uv_do_cubo(a: f32, b: f32) -> Vec2 {
    Vec2::new((a + 1.0) * 0.5, (b + 1.0) * 0.5)
}

/// Resolve os atributos de um triângulo solto: sem normais informadas, a normal
/// é a da própria face; sem uv, a origem da textura; sem tangentes, a base que
/// a uv do próprio triângulo define.
fn monta_triangulo(
    pontos: [Vec3; 3],
    normais: Option<[Vec3; 3]>,
    tangentes: Option<[Tangent; 3]>,
    uvs: Option<[Vec2; 3]>,
) -> [Vertex; 3] {
    let normais = normais.unwrap_or_else(|| {
        let n = (pontos[1] - pontos[0])
            .cross(pontos[2] - pontos[0])
            .normalized();
        [n; 3]
    });

    let uvs = uvs.unwrap_or([Vec2::ZERO; 3]);

    let tangentes = tangentes.unwrap_or_else(|| match eixos_da_uv(pontos, uvs) {
        Some((du, dv)) => [0, 1, 2].map(|k| Tangent::from_uv_axes(normais[k], du, dv)),
        // Se o triângulo de uvs é degenerado, qualquer perpendicular à normal serve.
        None => normais.map(Tangent::any_perpendicular),
    });

    [0, 1, 2].map(|k| Vertex {
        position: pontos[k],
        normal: normais[k],
        tangent: tangentes[k],
        uv: uvs[k],
    })
}

/// As direções, no espaço, em que o `u` e o `v` da uv crescem sobre um
/// triângulo. `None` quando a uv é degenerada e não define direção nenhuma.
///
/// Dentro do triângulo o mapeamento é afim, então cada aresta é uma combinação
/// dos dois eixos com os pesos que a uv dá:
///
/// ```text
/// e1 = d1.x·T + d1.y·B
/// e2 = d2.x·T + d2.y·B
///
/// ┌            ┐ ┌   ┐   ┌    ┐
/// │ d1.x  d1.y │ │ T │ = │ e1 │
/// │ d2.x  d2.y │ │ B │   │ e2 │
/// └            ┘ └   ┘   └    ┘
/// ```
///
/// Com cramer, fixando x:
///
/// ```text
///       │ e1.x  d1.y │             │ d1.x  e1.x │
///       │ e2.x  d2.y │             │ d2.x  e2.x │
/// T.x = ──────────────       B.x =  ──────────────
///       │ d1.x  d1.y │             │ d1.x  d1.y │
///       │ d2.x  d2.y │             │ d2.x  d2.y │
/// ```
///
/// Agora é fazer o mesmo pra y e z.
pub fn eixos_da_uv(pontos: [Vec3; 3], uvs: [Vec2; 3]) -> Option<(Vec3, Vec3)> {
    let e1 = pontos[1] - pontos[0];
    let e2 = pontos[2] - pontos[0];

    let d1 = uvs[1] - uvs[0];
    let d2 = uvs[2] - uvs[0];

    let det = d1.x * d2.y - d2.x * d1.y;
    if det.abs() < 1e-12 {
        // determinante perto de 0 quer dizer que du e dv são paralelos; não tem eixos
        return None;
    }

    Some(((e1 * d2.y - e2 * d1.y) / det, (e2 * d1.x - e1 * d2.x) / det))
}

fn tangente_da_esfera(normal: Vec3) -> Tangent {
    // A fórmula da normal é N(θ,ϕ) = (sinθcosϕ, cosθ, sinθsinϕ).
    //
    // A tangente é a direção em que u cresce, ou seja cresce junto com ϕ, ou seja é a derivada de N com
    // relação a ϕ: dN/dϕ = (−sinθsinϕ, 0, sinθcosϕ)
    //
    // Comparando a fórmula da normal com a fórmula da derivada, dá de conseguir a derivada com:
    // (-normal.z, 0, normal.x)
    let du = Vec3::new(-normal.z, 0.0, normal.x);

    // Nos polos o paralelo tem raio zero e a direção do u não existe.
    if du.length() < 1e-6 {
        return Tangent::any_perpendicular(normal);
    }

    Tangent::new(du.normalized(), 1.0)
}

fn mais_perto(a: Hit, b: Hit) -> Hit {
    if b.t < a.t { b } else { a }
}

pub(crate) fn intersecao_triangulo(ray: Ray, face: [Vertex; 3]) -> Option<Hit> {
    let (t, pesos) = moller_trumbore(ray, face.map(|v| v.position))?;

    // No raytracer a geometria não passa por divisão por z diretamente, então podemos calcular as baricêntricas sem
    // corrigir isso.
    let baricentricas = Baricentrics::orthographic(pesos);

    let normal = baricentricas
        .interpolate_vec3(face.map(|v| v.normal))
        .normalized();

    // A normal do plano do triângulo, apontando pro mesmo lado que a normal de shading.
    let geometrica = {
        let n = (face[1].position - face[0].position)
            .cross(face[2].position - face[0].position)
            .normalized();

        if n.dot(normal) < 0.0 { -n } else { n }
    };

    Some(Hit {
        t,
        normal,
        geometric_normal: geometrica,
        tangent: baricentricas.interpolate_tangent(face.map(|v| v.tangent)),
        uv: baricentricas.interpolate_vec2(face.map(|v| v.uv)),
    })
}

// Teoria: raytracer.md#o-raio-e-o-triangulo
/// Möller e Trumbore (1997): resolve a interseção e as coordenadas baricêntricas
/// de uma vez, sem montar a equação do plano do triângulo.
fn moller_trumbore(r: Ray, p: [Vec3; 3]) -> Option<(f32, [f32; 3])> {
    let borda1 = p[1] - p[0];
    let borda2 = p[2] - p[0];

    let perpendicular = r.direction.cross(borda2);
    let determinante = borda1.dot(perpendicular);

    if determinante.abs() < 1e-8 {
        return None;
    }

    let inverso = 1.0 / determinante;
    let ate_o_vertice = r.origin - p[0];

    let u = ate_o_vertice.dot(perpendicular) * inverso;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = ate_o_vertice.cross(borda1);

    let v = r.direction.dot(q) * inverso;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = borda2.dot(q) * inverso;
    if t <= 0.0 {
        return None;
    }

    Some((t, [1.0 - u - v, u, v]))
}

// Teoria: raytracer.md#o-raio-e-a-esfera
fn intersecao_esfera(r: Ray, centro: Vec3, raio: f32) -> Option<f32> {
    let oc = r.origin - centro;

    // `a` não é 1: o raio chega aqui em espaço local, sem renormalizar, pra que
    // o `t` valha também no mundo.
    let a = r.direction.dot(r.direction);
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

/// Diferente da esfera, a normal e a uv dependem de qual face o raio acertou, então
/// o acerto já sai inteiro daqui.
fn intersecao_cubo(r: Ray, centro: Vec3, meia_aresta: f32) -> Option<Hit> {
    let h = meia_aresta.abs();
    if h == 0.0 {
        return None;
    }

    // O raio com o centro do cubo na origem: as seis faces viram os planos
    // x = ±h, y = ±h e z = ±h.
    let o = r.origin - centro;
    let d = r.direction;

    let mut perto: Option<Hit> = None;

    // Cada face: a normal, e pra onde apontam a direita e o "pra baixo" de quem
    // olha a face de fora. Mesma tabela de `cubo`, pra que a textura caia igual
    // na malha e aqui.
    #[rustfmt::skip]
    let faces = [
        (Vec3::new( 1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0, -1.0), Vec3::new(0.0, -1.0,  0.0)),
        (Vec3::new( 0.0,  1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0,  1.0)),
        (Vec3::new( 0.0,  0.0,  1.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        (Vec3::new(-1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0,  1.0), Vec3::new(0.0, -1.0,  0.0)),
        (Vec3::new( 0.0, -1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0, -1.0)),
        (Vec3::new( 0.0,  0.0, -1.0), Vec3::new(-1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
    ];

    // O plano da face é n·x = h, cruzado quando n·o + t·(n·d) = h, ou seja
    // t = (h − n·o) / (n·d). Como a normal é um eixo, o produto escalar só
    // escolhe a componente e o sinal.
    //
    // Raio paralelo ao plano dá n·d = 0, e a divisão dá ±∞ (ou NaN, com a
    // origem em cima do plano). Como em `Aabb::hit`, isso é proposital: um t
    // infinito leva o ponto pro infinito e ele cai fora do quadrado, e NaN
    // falha o t > 0.

    for (normal, direita, baixo) in faces {
        let t = (h - o.dot(normal)) / d.dot(normal);
        let p = o + d * t;

        // `(a, b)` são as coordenadas, em meias arestas, do ponto em que o raio
        // cruza o plano da face. O plano é infinito: o acerto só vale dentro do
        // quadrado a, b ∈ [-1, 1].
        let (a, b) = (p.dot(direita) / h, p.dot(baixo) / h);

        if t > 0.0 && a.abs() <= 1.0 && b.abs() <= 1.0 {
            let hit = Hit {
                t,
                normal,
                geometric_normal: normal,
                tangent: Tangent::from_uv_axes(normal, direita, baixo),
                uv: uv_do_cubo(a, b),
            };

            perto = Some(perto.map_or(hit, |p| mais_perto(p, hit)));
        }
    }

    perto
}

// Teoria: raytracer.md#o-uv-do-acerto
fn uv_da_esfera(normal: Vec3) -> Vec2 {
    // Mesma parametrização de `esfera_uv`, invertida: theta do polo, phi da
    // volta no equador.
    let theta = normal.y.acos() / std::f32::consts::PI;

    let phi = normal.z.atan2(normal.x) / std::f32::consts::TAU;

    // atan2 devolve -pi..pi, que vira -0.5..0.5 aqui.
    let phi = if phi >= 0.0 { phi } else { 1.0 + phi };

    Vec2::new(phi, theta)
}
