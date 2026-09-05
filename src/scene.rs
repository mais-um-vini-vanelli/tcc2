//! Descrição da cena: câmera, luzes e objetos.

use crate::color::Rgb;
use crate::math::{Vec2, Vec3, Vertex};
use crate::transform::Transform;

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Scene {
            camera,
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn with_light(mut self, light: Light) -> Self {
        self.lights.push(light);
        self
    }

    pub fn with_object(mut self, object: Object) -> Self {
        self.objects.push(object);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub geometry: Primitive,
    pub material: Material,
    pub transform: Transform,
}

impl Object {
    pub fn new(geometry: Primitive, material: Material) -> Self {
        Object {
            geometry,
            material,
            transform: Transform::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// Luz pontual
    Point {
        position: Vec3,
        color: Rgb,
        intensity: f32,
    },
    /// Luz direcional
    Directional {
        direction: Vec3,
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
            | Light::Ambient { color, .. } => color,
        }
    }

    pub fn intensity(&self) -> f32 {
        match *self {
            Light::Point { intensity, .. }
            | Light::Directional { intensity, .. }
            | Light::Ambient { intensity, .. } => intensity,
        }
    }
}

/// Propriedades de superfície.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Cor difusa base.
    pub albedo: Rgb,
    /// Fração de luz refletida especularmente (raio de reflexão).
    pub reflectivity: f32,
    /// Fração de luz transmitida (raio de refração).
    pub transparency: f32,
    /// Índice de refração, usado quando `transparency > 0` (vidro ~ 1.5).
    pub refractive_index: f32,
    /// Expoente especular do termo de Phong.
    pub shininess: f32,
    /// Textura difusa opcional
    pub texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            albedo: Rgb::gray(0.8),
            reflectivity: 0.0,
            transparency: 0.0,
            refractive_index: 1.0,
            shininess: 32.0,
            texture: None,
        }
    }
}

impl Material {
    /// Material difuso puro da cor dada.
    pub fn diffuse(albedo: Rgb) -> Self {
        Material {
            albedo,
            ..Material::default()
        }
    }
}

/// Imagem de textura em memória, com cor em float como o resto do renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    width: u32,
    height: u32,
    texels: Vec<Rgb>,
}

impl Texture {
    /// Erro se `texels` não tiver exatamente `width * height` elementos.
    pub fn new(width: u32, height: u32, texels: Vec<Rgb>) -> Result<Self, TextureError> {
        let expected = width as usize * height as usize;
        if texels.len() != expected {
            return Err(TextureError::WrongPixelCount {
                expected,
                found: texels.len(),
            });
        }
        Ok(Texture {
            width,
            height,
            texels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn texels(&self) -> &[Rgb] {
        &self.texels
    }

    pub fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.texels[(y * self.width + x) as usize])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureError {
    WrongPixelCount { expected: usize, found: usize },
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::WrongPixelCount { expected, found } => {
                write!(f, "textura espera {expected} pixels, recebeu {found}")
            }
        }
    }
}

impl std::error::Error for TextureError {}

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
    pub uvs: Option<Vec<Vec2>>,
}

impl Mesh {
    /// Malha sem normais nem uv.
    pub fn new(vertices: Vec<Vec3>, indices: Vec<u32>) -> Self {
        Mesh {
            vertices,
            indices,
            normals: None,
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

        let normais = self
            .normals
            .as_deref()
            .and_then(|n| tres(n, trio))
            .unwrap_or_else(|| {
                let n = (pontos[1] - pontos[0])
                    .cross(pontos[2] - pontos[0])
                    .normalized();
                [n; 3]
            });

        let uvs = self
            .uvs
            .as_deref()
            .and_then(|u| tres(u, trio))
            .unwrap_or([Vec2::ZERO; 3]);

        Some([0usize, 1, 2].map(|k| Vertex {
            position: pontos[k],
            normal: normais[k],
            uv: uvs[k],
        }))
    }
}

/// Geometria renderizável, em espaço local.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Triangle {
        points: [Vec3; 3],
        /// Normais por vértice.
        normals: Option<[Vec3; 3]>,
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
                uvs,
            } => Mesh {
                vertices: points.to_vec(),
                indices: vec![0, 1, 2],
                normals: normals.map(|n| n.to_vec()),
                uvs: uvs.map(|u| u.to_vec()),
            },

            Primitive::Mesh(malha) => malha.clone(),

            Primitive::Sphere { center, radius } => Self::esfera_uv(*center, *radius),

            Primitive::Cube { center, half_edge } => {
                let center = *center;
                let h = *half_edge;

                let vertices = [
                    Vec3::new(-h, -h, -h),
                    Vec3::new(h, -h, -h),
                    Vec3::new(h, h, -h),
                    Vec3::new(-h, h, -h),
                    Vec3::new(-h, -h, h),
                    Vec3::new(h, -h, h),
                    Vec3::new(h, h, h),
                    Vec3::new(-h, h, h),
                ]
                .map(|v| v + center)
                .to_vec();

                #[rustfmt::skip]
                let indices = vec![
                    4, 5, 6,  4, 6, 7, // frente   (+z)
                    0, 3, 2,  0, 2, 1, // fundo    (-z)
                    1, 2, 6,  1, 6, 5, // direita  (+x)
                    0, 4, 7,  0, 7, 3, // esquerda (-x)
                    3, 7, 6,  3, 6, 2, // topo     (+y)
                    0, 1, 5,  0, 5, 4, // base     (-y)
                ];

                Mesh::new(vertices, indices)
            }
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
            uvs: Some(uvs),
        }
    }
}
