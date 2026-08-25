//! Descrição da cena: câmera, luzes e objetos.
//!

use crate::color::Rgb;
use crate::math::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub objects: Vec<Primitive>,
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

    pub fn with_object(mut self, object: Primitive) -> Self {
        self.objects.push(object);
        self
    }
}

/// Câmera.
///
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
    /// O rasterizador usa para montar a matriz de visão e o ray tracer usa para orientar os raios primários.
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
    pixels: Vec<Rgb>,
}

impl Texture {
    /// Erro se `pixels` não tiver exatamente `width * height` elementos.
    pub fn new(width: u32, height: u32, pixels: Vec<Rgb>) -> Result<Self, TextureError> {
        let expected = width as usize * height as usize;
        if pixels.len() != expected {
            return Err(TextureError::WrongPixelCount {
                expected,
                found: pixels.len(),
            });
        }
        Ok(Texture {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    pub fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(y * self.width + x) as usize])
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

/// Objetos renderizáveis.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Triangle {
        points: [Vec3; 3],
        /// Normais por vértice.
        normals: Option<[Vec3; 3]>,
        uvs: Option<[Vec2; 3]>,
        material: Material,
    },
    Mesh {
        vertices: Vec<Vec3>,
        /// Índices em trios: cada três entradas formam um triângulo.
        indices: Vec<u32>,
        normals: Option<Vec<Vec3>>,
        uvs: Option<Vec<Vec2>>,
        material: Material,
    },
    Sphere {
        center: Vec3,
        radius: f32,
        material: Material,
    },
}

impl Primitive {
    /// Triângulo sem normais nem uv
    pub fn triangle(points: [Vec3; 3], material: Material) -> Self {
        Primitive::Triangle {
            points,
            normals: None,
            uvs: None,
            material,
        }
    }

    pub fn sphere(center: Vec3, radius: f32, material: Material) -> Self {
        Primitive::Sphere {
            center,
            radius,
            material,
        }
    }

    pub fn material(&self) -> &Material {
        match self {
            Primitive::Triangle { material, .. }
            | Primitive::Mesh { material, .. }
            | Primitive::Sphere { material, .. } => material,
        }
    }
}
