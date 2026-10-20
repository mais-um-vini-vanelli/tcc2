use crate::math::{Ray, Tangent, Vec3};
use crate::parameters::TransformMethod;

/// Escala, rotação e translação, aplicadas nessa ordem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    /// Ângulos de Euler em graus, aplicados na ordem X, Y, Z.
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        translation: Vec3::ZERO,
        rotation: Vec3::ZERO,
        scale: Vec3::ONE,
    };

    /// Monta um transform que leva o objeto pra `position` e olhando pra `target`.
    pub fn looking_at(position: Vec3, target: Vec3) -> Transform {
        let forward = (target - position).normalized();

        Transform {
            translation: position,
            // Transforma os deslocamentos de forward em ângulos de rotação com asin() e atan2()
            rotation: Vec3::new(
                forward.y.asin().to_degrees(),
                (-forward.x).atan2(-forward.z).to_degrees(),
                0.0,
            ),
            scale: Vec3::ONE,
        }
    }

    pub fn apply_point(&self, point: Vec3, transform_method: TransformMethod) -> Vec3 {
        Self::rotate(
            Self::scale(point, self.scale),
            self.rotation,
            transform_method,
        ) + self.translation
    }

    /// Como `apply_point`, sem a translação: direção não tem posição.
    pub fn apply_direction(&self, direction: Vec3, transform_method: TransformMethod) -> Vec3 {
        Self::rotate(
            Self::scale(direction, self.scale),
            self.rotation,
            transform_method,
        )
    }

    /// Como `apply_direction`, mas usando a escala inversa.
    pub fn apply_normal(&self, normal: Vec3, transform_method: TransformMethod) -> Vec3 {
        Self::rotate(
            Self::inverted_scale(normal, self.scale),
            self.rotation,
            transform_method,
        )
        .normalized()
    }

    pub fn apply_tangent(&self, tangent: Tangent, transform_method: TransformMethod) -> Tangent {
        // Uma escala com número ímpar de eixos negativos espelha o espaço, então precisa
        // inverter a tangente.
        let invert_handedness = self.scale.x * self.scale.y * self.scale.z < 0.0;

        Tangent::new(
            self.apply_direction(tangent.direction, transform_method)
                .normalized(),
            if invert_handedness {
                -tangent.handedness
            } else {
                tangent.handedness
            },
        )
    }

    /// O inverso de `apply_point`.
    pub fn undo_point(&self, point: Vec3, transform_method: TransformMethod) -> Vec3 {
        Self::inverted_scale(
            Self::inverted_rotate(point - self.translation, self.rotation, transform_method),
            self.scale,
        )
    }

    /// Como `undo_point`, sem a translação.
    pub fn undo_direction(&self, direction: Vec3, transform_method: TransformMethod) -> Vec3 {
        Self::inverted_scale(
            Self::inverted_rotate(direction, self.rotation, transform_method),
            self.scale,
        )
    }

    // Leva o raio pro espaço local do objeto.
    pub fn undo_ray(&self, ray: Ray, transform_method: TransformMethod) -> Ray {
        Ray {
            origin: self.undo_point(ray.origin, transform_method),
            direction: self.undo_direction(ray.direction, transform_method),
        }
    }

    fn scale(vec: Vec3, scale: Vec3) -> Vec3 {
        let scale = Self::clamp_scale(scale);
        Vec3::new(vec.x * scale.x, vec.y * scale.y, vec.z * scale.z)
    }

    fn inverted_scale(vec: Vec3, scale: Vec3) -> Vec3 {
        let scale = Self::clamp_scale(scale);
        Vec3::new(vec.x / scale.x, vec.y / scale.y, vec.z / scale.z)
    }

    fn clamp_scale(scale: Vec3) -> Vec3 {
        let clamp = |value: f32| {
            const EPSILON: f32 = 1e-6;

            if value.abs() < EPSILON {
                // Uma escala 0.0 em um eixo achata o espaço em um plano, e não tem como descobrir
                // o inverso, então limitamos a um epsilon.
                EPSILON.copysign(value)
            } else {
                value
            }
        };

        Vec3::new(clamp(scale.x), clamp(scale.y), clamp(scale.z))
    }

    fn rotate(vec: Vec3, angle: Vec3, trasnform_method: TransformMethod) -> Vec3 {
        match trasnform_method {
            TransformMethod::Stepwise => Self::rotate_z(
                Self::rotate_y(
                    Self::rotate_x(vec, angle.x.to_radians()),
                    angle.y.to_radians(),
                ),
                angle.z.to_radians(),
            ),
        }
    }

    fn inverted_rotate(vec: Vec3, angle: Vec3, transform_method: TransformMethod) -> Vec3 {
        match transform_method {
            TransformMethod::Stepwise => {
                let vec = Self::rotate_z(vec, -angle.z.to_radians());
                let vec = Self::rotate_y(vec, -angle.y.to_radians());
                Self::rotate_x(vec, -angle.x.to_radians())
            }
        }
    }

    fn rotate_x(vec: Vec3, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        Vec3::new(vec.x, vec.y * cos - vec.z * sin, vec.y * sin + vec.z * cos)
    }

    fn rotate_y(vec: Vec3, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        Vec3::new(vec.x * cos + vec.z * sin, vec.y, -vec.x * sin + vec.z * cos)
    }

    fn rotate_z(vec: Vec3, angle: f32) -> Vec3 {
        let (sin, cos) = angle.sin_cos();
        Vec3::new(vec.x * cos - vec.y * sin, vec.x * sin + vec.y * cos, vec.z)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TransformChain(Vec<Transform>);

impl TransformChain {
    pub const IDENTITY: TransformChain = TransformChain(Vec::new());

    /// Monta a cadeia a partir da pilha de transforms do objeto no scene graph.
    pub fn from_stack(stack: &[Transform]) -> TransformChain {
        TransformChain(stack.iter().rev().copied().collect())
    }

    pub fn apply_point(&self, point: Vec3, transform_method: TransformMethod) -> Vec3 {
        self.0.iter().fold(point, |point, link| {
            link.apply_point(point, transform_method)
        })
    }

    pub fn apply_direction(&self, direction: Vec3, transform_method: TransformMethod) -> Vec3 {
        self.0.iter().fold(direction, |direction, link| {
            link.apply_direction(direction, transform_method)
        })
    }

    pub fn apply_normal(&self, normal: Vec3, transform_method: TransformMethod) -> Vec3 {
        self.0.iter().fold(normal, |normal, link| {
            link.apply_normal(normal, transform_method)
        })
    }

    pub fn apply_tangent(&self, tangent: Tangent, transform_method: TransformMethod) -> Tangent {
        self.0.iter().fold(tangent, |tangent, link| {
            link.apply_tangent(tangent, transform_method)
        })
    }

    pub fn undo_point(&self, point: Vec3, transform_method: TransformMethod) -> Vec3 {
        self.0.iter().rev().fold(point, |point, link| {
            link.undo_point(point, transform_method)
        })
    }

    pub fn undo_direction(&self, direction: Vec3, transform_method: TransformMethod) -> Vec3 {
        self.0.iter().rev().fold(direction, |direction, link| {
            link.undo_direction(direction, transform_method)
        })
    }

    pub fn undo_ray(&self, ray: Ray, transform_method: TransformMethod) -> Ray {
        Ray {
            origin: self.undo_point(ray.origin, transform_method),
            direction: self.undo_direction(ray.direction, transform_method),
        }
    }
}
