use std::ops::{Add, Div, Index, Mul, Neg, Sub};

use crate::color::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    // Produto escalar: diz o quanto dois vetores estão alinhados, e é a projeção
    // de um no outro quando um deles é unitário.
    pub fn dot(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    // Produto vetorial em 2D: simulado estendendo os dois vetores pra 3D com
    // z = 0.
    pub fn cross(self, other: Vec2) -> f32 {
        Vec3::new(self.x, self.y, 0.0)
            .cross(Vec3::new(other.x, other.y, 0.0))
            .z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;
    fn div(self, scalar: f32) -> Vec2 {
        Vec2::new(self.x / scalar, self.y / scalar)
    }
}

impl Index<usize> for Vec2 {
    type Output = f32;
    fn index(&self, axis: usize) -> &f32 {
        match axis {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("axis out of bounds: {axis}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Vec3 = Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    // Produto escalar: diz o quanto dois vetores estão alinhados, e é a projeção
    // de um no outro quando um deles é unitário.
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    // Produto vetorial: retorna um vetor perpendicular aos dois, de comprimento igual à
    // área do paralelogramo. O sentido sai da ordem, pela regra da mão direita.
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Vec3 {
        let length = self.length();
        if length == 0.0 {
            Vec3::ZERO
        } else {
            self / length
        }
    }

    pub fn min(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    pub fn max(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        other * self
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, scalar: f32) -> Vec3 {
        Vec3::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;
    fn index(&self, axis: usize) -> &f32 {
        match axis {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("axis out of bounds: {axis}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray {
            origin,
            direction: direction.normalized(),
        }
    }

    /// Devolve o ponto que está a t unidades da origem do raio.
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Axis-aligned bounding box: o menor e o maior valor em cada eixo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Uma AABB que não contém nada.
    pub const EMPTY: Aabb = Aabb {
        min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
    };

    pub fn around(points: impl IntoIterator<Item = Vec3>) -> Aabb {
        points.into_iter().fold(Aabb::EMPTY, |caixa, p| Aabb {
            min: caixa.min.min(p),
            max: caixa.max.max(p),
        })
    }

    pub fn union(self, other: Aabb) -> Aabb {
        Aabb {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Área de superfície. Infinita na `EMPTY`.
    pub fn surface_area(self) -> f32 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    pub fn corners(self) -> [Vec3; 8] {
        let (a, b) = (self.min, self.max);

        [
            Vec3::new(a.x, a.y, a.z),
            Vec3::new(b.x, a.y, a.z),
            Vec3::new(a.x, b.y, a.z),
            Vec3::new(b.x, b.y, a.z),
            Vec3::new(a.x, a.y, b.z),
            Vec3::new(b.x, a.y, b.z),
            Vec3::new(a.x, b.y, b.z),
            Vec3::new(b.x, b.y, b.z),
        ]
    }

    /// Calcula `t` em que o raio entra na caixa, se ele passa por ela em algum ponto
    /// entre `t_min` e `t_max`.
    pub fn intersect(self, ray: Ray, t_min: f32, t_max: f32) -> Option<f32> {
        // Calcula o intervalo de `t` em que o raio está entre os dois planos de um eixo.
        let slab = |origin: f32, direction: f32, min: f32, max: f32| {
            let t0 = (min - origin) / direction;
            let t1 = (max - origin) / direction;

            // Quando a direção do raio é negativa em um eixo, o raio cruza o `max` antes do `min`.
            (t0.min(t1), t0.max(t1))
        };

        let (x0, x1) = slab(ray.origin.x, ray.direction.x, self.min.x, self.max.x);
        let (y0, y1) = slab(ray.origin.y, ray.direction.y, self.min.y, self.max.y);
        let (z0, z1) = slab(ray.origin.z, ray.direction.z, self.min.z, self.max.z);

        let entry = t_min.max(x0).max(y0).max(z0);
        let exit = t_max.min(x1).min(y1).min(z1);

        (entry <= exit).then_some(entry)
    }
}

/// Plano em 3D. Tem a forma `n · p + deslocamento = 0`, com `deslocamento` = `-(n · p0)`
/// para todo ponto `p0` do plano.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub displacement: f32,
}

impl Plane {
    pub fn new(normal: Vec3, displacement: f32) -> Self {
        Plane {
            normal,
            displacement,
        }
    }

    /// Calcula a distância com sinal: zero no plano, positivo do lado da normal,
    /// negativo do lado oposto.
    pub fn distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.displacement
    }
}

/// A tangente por vértice: pra onde a superfície anda quando o `u` da uv
/// cresce.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tangent {
    pub direction: Vec3,
    /// +1 ou -1. Indica a direção da bitangente.
    pub handedness: f32,
}

impl Default for Tangent {
    fn default() -> Self {
        Tangent {
            direction: Vec3::new(1.0, 0.0, 0.0),
            handedness: 1.0,
        }
    }
}

impl Tangent {
    pub fn new(direction: Vec3, handedness: f32) -> Self {
        Tangent {
            direction,
            handedness: if handedness < 0.0 { -1.0 } else { 1.0 },
        }
    }

    /// Calcula a tangente a partir das direções `du` e `dv` da superfície.
    pub fn from_uv_axes(normal: Vec3, du: Vec3, dv: Vec3) -> Self {
        let tangent_direction = (du - normal * normal.dot(du)).normalized();

        if tangent_direction == Vec3::ZERO {
            return Tangent::any_perpendicular(normal);
        }

        Tangent::new(tangent_direction, normal.cross(tangent_direction).dot(dv))
    }

    /// Calcula uma tangente qualquer perpendicular à normal.
    pub fn any_perpendicular(normal: Vec3) -> Self {
        // Escolher um eixo que não é paralelo à normal.
        let axis = if normal.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        Tangent {
            direction: axis.cross(normal).normalized(),
            handedness: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tangent: Tangent,
    pub uv: Vec2,
}

impl Vertex {
    /// O vértice a `t` do caminho entre os dois, com os atributos interpolados também.
    pub fn lerp(self, other: Vertex, t: f32) -> Vertex {
        Vertex {
            position: self.position + (other.position - self.position) * t,
            normal: self.normal + (other.normal - self.normal) * t,
            tangent: Tangent {
                direction: self.tangent.direction
                    + (other.tangent.direction - self.tangent.direction) * t,
                handedness: self.tangent.handedness,
            },
            uv: self.uv + (other.uv - self.uv) * t,
        }
    }
}

/// Os pesos de interpolação de um ponto de dentro de um triângulo; somam 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Baricentrics {
    weights: [f32; 3],
}

impl Baricentrics {
    pub fn new(weights: [f32; 3]) -> Self {
        Baricentrics { weights }
    }

    pub fn perspective_corrected(screen_weights: [f32; 3], depths: [f32; 3]) -> Self {
        let [l0, l1, l2] = screen_weights;
        let [z0, z1, z2] = depths;

        let inverted = [l0 / z0, l1 / z1, l2 / z2];
        let z = 1.0 / (inverted[0] + inverted[1] + inverted[2]);

        Baricentrics {
            weights: inverted.map(|inverso| inverso * z),
        }
    }

    pub fn interpolate_f32(&self, values: [f32; 3]) -> f32 {
        values[0] * self.weights[0] + values[1] * self.weights[1] + values[2] * self.weights[2]
    }

    pub fn interpolate_vec2(&self, values: [Vec2; 3]) -> Vec2 {
        values[0] * self.weights[0] + values[1] * self.weights[1] + values[2] * self.weights[2]
    }

    pub fn interpolate_vec3(&self, values: [Vec3; 3]) -> Vec3 {
        values[0] * self.weights[0] + values[1] * self.weights[1] + values[2] * self.weights[2]
    }

    pub fn interpolate_tangent(&self, values: [Tangent; 3]) -> Tangent {
        Tangent {
            direction: self.interpolate_vec3(values.map(|t| t.direction)),
            handedness: values[0].handedness,
        }
    }

    pub fn interpolate_rgb(&self, values: [Rgb; 3]) -> Rgb {
        values[0] * self.weights[0] + values[1] * self.weights[1] + values[2] * self.weights[2]
    }
}
