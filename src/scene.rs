use crate::color::Rgb;
use crate::geometry::Geometry;
use crate::math::{Plane, Tangent, Vec2, Vec3};
use crate::parameters::{TextureFilter, TransformMethod};
use crate::texture::{Footprint, Texture};
use crate::transform::{Transform, TransformChain};

#[derive(Debug, Clone)]
pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn flatten(
        &self,
        method: TransformMethod,
    ) -> (
        Rgb,
        Option<CameraInstance>,
        Vec<LightInstance>,
        Vec<GeometryInstance<'_>>,
    ) {
        let mut result = (Rgb::BLACK, None, Vec::new(), Vec::new());

        for object in &self.objects {
            let mut transform_stack = Vec::new();
            Self::descend(object, &mut transform_stack, method, &mut result);
        }

        let (environment, _, light_instances, _) = &mut result;

        *environment = light_instances
            .iter()
            .filter_map(|light_instance| match light_instance.local {
                Light::Environment { color, intensity } => Some(color * intensity),
                _ => None,
            })
            .fold(Rgb::BLACK, |a, b| a + b);

        result
    }

    fn descend<'a>(
        object: &'a Object,
        transform_stack: &mut Vec<Transform>,
        transform_method: TransformMethod,
        result: &mut (
            Rgb,
            Option<CameraInstance>,
            Vec<LightInstance>,
            Vec<GeometryInstance<'a>>,
        ),
    ) {
        let (_, camera_instance, light_instances, geometry_instances) = &mut *result;

        transform_stack.push(object.transform);

        match &object.kind {
            Some(ObjectKind::Camera(local)) => {
                camera_instance.get_or_insert_with(|| {
                    let (position, basis) = Self::apply_position_and_basis(
                        &TransformChain::from_stack(transform_stack),
                        transform_method,
                    );

                    CameraInstance {
                        local: *local,
                        position,
                        basis,
                    }
                });
            }

            Some(ObjectKind::Light(local)) => {
                let (position, basis) = Self::apply_position_and_basis(
                    &TransformChain::from_stack(transform_stack),
                    transform_method,
                );

                light_instances.push(LightInstance {
                    local: *local,
                    position,
                    basis,
                });
            }

            Some(ObjectKind::Primitive(primitive)) => geometry_instances.push(GeometryInstance {
                primitive,
                transform_chain: TransformChain::from_stack(transform_stack),
            }),

            None => (),
        }

        for child in &object.children {
            Self::descend(child, transform_stack, transform_method, result);
        }

        transform_stack.pop();
    }

    fn apply_position_and_basis(
        transform_chain: &TransformChain,
        transform_method: TransformMethod,
    ) -> (Vec3, (Vec3, Vec3, Vec3)) {
        let forward = transform_chain
            .apply_direction(Vec3::new(0.0, 0.0, -1.0), transform_method)
            .normalized();

        let up = transform_chain.apply_direction(Vec3::new(0.0, 1.0, 0.0), transform_method);
        let right = forward.cross(up).normalized();

        (
            transform_chain.apply_point(Vec3::ZERO, transform_method),
            (right, right.cross(forward), forward),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryInstance<'a> {
    pub primitive: &'a Primitive,
    pub transform_chain: TransformChain,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraInstance {
    pub local: Camera,
    pub position: Vec3,
    pub basis: (Vec3, Vec3, Vec3),
}

impl CameraInstance {
    pub fn to_camera_space(&self, point: Vec3) -> Vec3 {
        let (right, up, forward) = self.basis;

        let d = point - self.position;
        Vec3::new(d.dot(right), d.dot(up), d.dot(forward))
    }

    pub fn from_camera_space(&self, point: Vec3) -> Vec3 {
        let (right, up, forward) = self.basis;

        self.position + right * point.x + up * point.y + forward * point.z
    }

    pub fn view_from(&self, point: Vec3) -> Vec3 {
        match self.local {
            Camera::Perspective { .. } => (self.position - point).normalized(),
            Camera::Orthographic { .. } => -self.basis.2,
        }
    }

    pub fn is_perspective(&self) -> bool {
        if let Camera::Perspective { .. } = self.local {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightInstance {
    pub local: Light,
    pub position: Vec3,
    pub basis: (Vec3, Vec3, Vec3),
}

impl LightInstance {
    pub fn forward(&self) -> Vec3 {
        self.basis.2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    Camera(Camera),
    Light(Light),
    Primitive(Primitive),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive {
    pub geometry: Geometry,
    pub material: Material,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub transform: Transform,
    pub kind: Option<ObjectKind>,
    pub children: Vec<Object>,
}

impl Object {
    pub fn primitive(primitive: Primitive, transform: Transform) -> Self {
        Object {
            transform,
            kind: Some(ObjectKind::Primitive(primitive)),
            children: Vec::new(),
        }
    }

    pub fn camera(camera: Camera, transform: Transform) -> Self {
        Object {
            transform,
            kind: Some(ObjectKind::Camera(camera)),
            children: Vec::new(),
        }
    }

    pub fn light(light: Light, transform: Transform) -> Self {
        Object {
            transform,
            kind: Some(ObjectKind::Light(light)),
            children: Vec::new(),
        }
    }

    pub fn group(transform: Transform) -> Self {
        Object {
            transform,
            kind: None,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Object>) -> Self {
        self.children = children;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Camera {
    Perspective {
        /// FOV vertical, em graus.
        fov: f32,
        /// Largura/altura da imagem.
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        /// Largura do volume de visão, em unidades.
        width: f32,
        /// Altura do volume de visão, em unidades.
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    pub fn to_screen_space(&self, point: Vec3, width: u32, height: u32) -> Option<Vec3> {
        let (x, y, z) = (point.x, point.y, point.z);

        let (near, _far) = self.clip_planes();

        if z < near {
            return None;
        }

        let (ndc_x, ndc_y) = match *self {
            Camera::Perspective { fov, aspect, .. } => {
                let half_height = (fov.to_radians() * 0.5).tan();

                (x / (z * half_height * aspect), y / (z * half_height))
            }
            Camera::Orthographic { width, height, .. } => (x / (width * 0.5), y / (height * 0.5)),
        };

        Some(Vec3::new(
            (ndc_x + 1.0) * 0.5 * width as f32,
            (1.0 - ndc_y) * 0.5 * height as f32,
            z,
        ))
    }

    pub fn clip_planes(&self) -> (f32, f32) {
        match *self {
            Camera::Perspective { near, far, .. } | Camera::Orthographic { near, far, .. } => {
                (near, far)
            }
        }
    }

    pub fn frustum(&self) -> [Plane; 6] {
        let (near, far) = self.clip_planes();

        let sides = match *self {
            Camera::Perspective { fov, aspect, .. } => {
                let half_height = (fov.to_radians() * 0.5).tan();
                let half_width = half_height * aspect;

                [
                    Plane::new(Vec3::new(1.0, 0.0, half_width), 0.0),
                    Plane::new(Vec3::new(-1.0, 0.0, half_width), 0.0),
                    Plane::new(Vec3::new(0.0, 1.0, half_height), 0.0),
                    Plane::new(Vec3::new(0.0, -1.0, half_height), 0.0),
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
            sides[0],
            sides[1],
            sides[2],
            sides[3],
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    /// Irradia luz em todas as direções e atenua com 1/d².
    Point { color: Rgb, intensity: f32 },
    /// Igual a `Point`, mas restrita a um cone, como uma lanterna.
    Spot {
        /// Meio-ângulo do cone interno, em graus.
        inner_angle: f32,
        /// Meio-ângulo do cone externo, em graus.
        outer_angle: f32,
        color: Rgb,
        intensity: f32,
    },
    /// Emite luz em raios paralelos, independente da posição e sem atenuação.
    Directional { color: Rgb, intensity: f32 },
    /// Luz que chega de todas as direções, e também a cor de fundo onde não há geometria.
    Environment { color: Rgb, intensity: f32 },
}

impl Light {
    pub fn color(&self) -> Rgb {
        match *self {
            Light::Point { color, .. }
            | Light::Directional { color, .. }
            | Light::Spot { color, .. }
            | Light::Environment { color, .. } => color,
        }
    }

    pub fn intensity(&self) -> f32 {
        match *self {
            Light::Point { intensity, .. }
            | Light::Directional { intensity, .. }
            | Light::Spot { intensity, .. }
            | Light::Environment { intensity, .. } => intensity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Channel<T> {
    Constant(T),
    Map(Texture),
}

impl<T> Channel<T> {
    pub fn texture(&self) -> Option<&Texture> {
        match self {
            Channel::Constant(_) => None,
            Channel::Map(texture) => Some(texture),
        }
    }
}

impl Channel<Rgb> {
    pub fn sample(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        texture_filtering: TextureFilter,
    ) -> Rgb {
        match self {
            Channel::Constant(color) => *color,
            Channel::Map(texture) => texture.sample(uv, footprint, texture_filtering),
        }
    }
}

impl Channel<f32> {
    pub fn sample(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        texture_filtering: TextureFilter,
    ) -> f32 {
        match self {
            Channel::Constant(value) => *value,
            Channel::Map(texture) => texture.sample(uv, footprint, texture_filtering).r,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialSample {
    pub albedo: Rgb,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: Rgb,
    pub transparency: f32,
    pub refractive_index: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Cor da superfície. Para dielétricos é a cor do brilho difuso; para metais é a cor do
    /// brilho especular.
    pub albedo: Channel<Rgb>,
    /// 0 para dielétricos, 1 metal para metais.
    pub metallic: Channel<f32>,
    /// 0 para superfícies regulares (espelhos), 1 para superfícies rugosas (difusas).
    pub roughness: Channel<f32>,
    /// Luz que a superfície emite, independente de luzes externas.
    pub emissive: Channel<Rgb>,
    /// Mapa de normais, ou `None` para usar a normal geométrica.
    pub normal: Option<Texture>,
    /// ~1 para superfícies transparentes, 0 para superfícies opacas.
    pub transparency: f32,
    /// Índice de refração. ~1.5 para vidro.
    pub refractive_index: f32,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            albedo: Channel::Constant(Rgb::gray(0.8)),
            metallic: Channel::Constant(0.0),
            roughness: Channel::Constant(0.5),
            emissive: Channel::Constant(Rgb::BLACK),
            normal: None,
            transparency: 0.0,
            refractive_index: 1.5,
        }
    }
}

impl Material {
    pub fn diffuse(albedo: Rgb) -> Self {
        Material {
            albedo: Channel::Constant(albedo),
            roughness: Channel::Constant(1.0),
            ..Material::default()
        }
    }

    pub fn metal(albedo: Rgb, roughness: f32) -> Self {
        Material {
            albedo: Channel::Constant(albedo),
            metallic: Channel::Constant(1.0),
            roughness: Channel::Constant(roughness),
            ..Material::default()
        }
    }

    pub fn emissive(color: Rgb) -> Self {
        Material {
            albedo: Channel::Constant(Rgb::BLACK),
            emissive: Channel::Constant(color),
            ..Material::default()
        }
    }

    pub fn is_emissive(&self) -> bool {
        match &self.emissive {
            Channel::Constant(color) => *color != Rgb::BLACK,
            Channel::Map(_) => true,
        }
    }

    pub fn sample(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        texture_filtering: TextureFilter,
    ) -> MaterialSample {
        MaterialSample {
            albedo: self.albedo.sample(uv, footprint, texture_filtering),
            metallic: self
                .metallic
                .sample(uv, footprint, texture_filtering)
                .clamp(0.0, 1.0),
            roughness: self
                .roughness
                .sample(uv, footprint, texture_filtering)
                .clamp(0.045, 1.0),
            emission: self.emissive.sample(uv, footprint, texture_filtering),
            transparency: self.transparency.clamp(0.0, 1.0),
            // Abaixo de 1 seria o material sendo o meio menos denso.
            refractive_index: self.refractive_index.max(1.0),
        }
    }

    /// Calcula a normal que o shading usa no ponto: a normal dos vértices distorcida pelo
    /// normal map.
    pub fn apply_normal_map(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        texture_filtering: TextureFilter,
        normal: Vec3,
        tangent: Tangent,
    ) -> Vec3 {
        if self.normal.is_none() {
            return normal;
        }

        let tangent = {
            // Gram-Schmidt: tirar de T a componente que está ao longo de N e normalizar.
            let vector = (tangent.direction - normal * normal.dot(tangent.direction)).normalized();

            if vector == Vec3::ZERO {
                // A tangente e a normal são paralelas.
                Tangent::new(
                    Tangent::any_perpendicular(normal).direction,
                    tangent.handedness,
                )
            } else {
                Tangent {
                    direction: vector,
                    handedness: tangent.handedness,
                }
            }
        };

        // Converter os componentes em [0,1] da textura para [-1,1].
        let sampled_normal = {
            let texel = self
                .normal
                .as_ref()
                .unwrap()
                .sample(uv, footprint, texture_filtering);

            Vec3::new(
                texel.r * 2.0 - 1.0,
                texel.g * 2.0 - 1.0,
                texel.b * 2.0 - 1.0,
            )
            .normalized()
        };

        let bitangent = normal.cross(tangent.direction) * tangent.handedness;

        (tangent.direction * sampled_normal.x
            + bitangent * sampled_normal.y
            + normal * sampled_normal.z)
            .normalized()
    }
}
