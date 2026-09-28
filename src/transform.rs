//! Posição, orientação e tamanho de um objeto no mundo.
//!
//! Teoria: transform.md#o-transform
//!
//! A geometria de um `Primitive` está em **espaço local**; o `Transform` do
//! objeto é o que a leva pro mundo. Os dois pipelines usam esta struct em
//! sentidos opostos: o rasterizador transforma a geometria (`apply_*`), o ray
//! tracer transforma o raio com a inversa (`undo_*`).

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

    // Teoria: transform.md#a-ordem-das-tres-etapas
    /// Local -> mundo: escala, rotaciona, translada.
    pub fn apply_point(&self, p: Vec3, method: TransformMethod) -> Vec3 {
        Self::gira(Self::escala(p, self.scale), self.rotation, method) + self.translation
    }

    /// Como `apply_point`, sem a translação: direção não tem posição.
    pub fn apply_direction(&self, d: Vec3, method: TransformMethod) -> Vec3 {
        Self::gira(Self::escala(d, self.scale), self.rotation, method)
    }

    // Teoria: transform.md#a-normal-usa-a-escala-reciproca
    /// A normal não acompanha a escala: ela usa o recíproco dela. É a forma
    /// legível do que a inversa transposta faz.
    pub fn apply_normal(&self, n: Vec3, method: TransformMethod) -> Vec3 {
        Self::gira(
            Self::escala(n, Self::reciproco(self.scale)),
            self.rotation,
            method,
        )
        .normalized()
    }

    pub fn apply_tangent(&self, t: Tangent, method: TransformMethod) -> Tangent {
        // A transformação pode mexer no *lado* da bitangente: uma escala
        // com um número ímpar de eixos negativos espelha o espaço.
        let espelha = self.scale.x * self.scale.y * self.scale.z < 0.0;

        Tangent::new(
            self.apply_direction(t.vector, method).normalized(),
            if espelha { -t.handedness } else { t.handedness },
        )
    }

    // Teoria: transform.md#desfazendo
    /// Mundo -> local: desfaz as três etapas na ordem contrária.
    pub fn undo_point(&self, p: Vec3, method: TransformMethod) -> Vec3 {
        Self::escala(
            Self::desgira(p - self.translation, self.rotation, method),
            Self::reciproco(self.scale),
        )
    }

    /// Como `undo_point`, sem a translação.
    pub fn undo_direction(&self, d: Vec3, method: TransformMethod) -> Vec3 {
        Self::escala(
            Self::desgira(d, self.rotation, method),
            Self::reciproco(self.scale),
        )
    }

    // Teoria: transform.md#o-raio-vai-para-o-espaco-local
    /// O raio em espaço local, para intersectar com a geometria não
    /// transformada.
    ///
    /// A direção sai **sem normalizar** de propósito: é isso que faz o `t` do
    /// acerto valer também no mundo, sem conversão de volta.
    pub fn undo_ray(&self, r: Ray, method: TransformMethod) -> Ray {
        Ray {
            origin: self.undo_point(r.origin, method),
            direction: self.undo_direction(r.direction, method),
        }
    }

    fn escala(v: Vec3, s: Vec3) -> Vec3 {
        Vec3::new(v.x * s.x, v.y * s.y, v.z * s.z)
    }

    fn reciproco(s: Vec3) -> Vec3 {
        let inverte = |v: f32| {
            if v == 0.0 {
                panic!("scale não pode ser 0.0 em nenhum eixo")
            } else {
                1.0 / v
            }
        };
        Vec3::new(inverte(s.x), inverte(s.y), inverte(s.z))
    }

    fn gira(v: Vec3, graus: Vec3, method: TransformMethod) -> Vec3 {
        match method {
            TransformMethod::Stepwise => {
                let v = Self::gira_x(v, graus.x.to_radians());
                let v = Self::gira_y(v, graus.y.to_radians());
                Self::gira_z(v, graus.z.to_radians())
            }
        }
    }

    fn desgira(v: Vec3, graus: Vec3, method: TransformMethod) -> Vec3 {
        match method {
            TransformMethod::Stepwise => {
                let v = Self::gira_z(v, -graus.z.to_radians());
                let v = Self::gira_y(v, -graus.y.to_radians());
                Self::gira_x(v, -graus.x.to_radians())
            }
        }
    }

    // Teoria: transform.md#girar-em-torno-de-um-eixo
    fn gira_x(v: Vec3, a: f32) -> Vec3 {
        let (sin, cos) = a.sin_cos();
        Vec3::new(v.x, v.y * cos - v.z * sin, v.y * sin + v.z * cos)
    }

    fn gira_y(v: Vec3, a: f32) -> Vec3 {
        let (sin, cos) = a.sin_cos();
        Vec3::new(v.x * cos + v.z * sin, v.y, -v.x * sin + v.z * cos)
    }

    fn gira_z(v: Vec3, a: f32) -> Vec3 {
        let (sin, cos) = a.sin_cos();
        Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, v.z)
    }
}

/// Os transforms de um nó da árvore da cena até a raiz; o primeiro elemento é o do próprio nó, o último é o da raiz.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chain(Vec<Transform>);

impl Chain {
    pub const IDENTITY: Chain = Chain(Vec::new());

    /// Monta a cadeia a partir dos elos **da raiz até o nó** (a ordem em que a
    /// travessia da árvore os encontra).
    pub fn from_root_first(links: &[Transform]) -> Chain {
        Chain(links.iter().rev().copied().collect())
    }

    pub fn links(&self) -> &[Transform] {
        &self.0
    }

    pub fn apply_point(&self, p: Vec3, method: TransformMethod) -> Vec3 {
        self.0.iter().fold(p, |p, link| link.apply_point(p, method))
    }

    pub fn apply_direction(&self, d: Vec3, method: TransformMethod) -> Vec3 {
        self.0
            .iter()
            .fold(d, |d, link| link.apply_direction(d, method))
    }

    pub fn apply_normal(&self, n: Vec3, method: TransformMethod) -> Vec3 {
        self.0
            .iter()
            .fold(n, |n, link| link.apply_normal(n, method))
    }

    pub fn apply_tangent(&self, t: Tangent, method: TransformMethod) -> Tangent {
        self.0
            .iter()
            .fold(t, |t, link| link.apply_tangent(t, method))
    }

    pub fn undo_point(&self, p: Vec3, method: TransformMethod) -> Vec3 {
        self.0
            .iter()
            .rev()
            .fold(p, |p, link| link.undo_point(p, method))
    }

    pub fn undo_direction(&self, d: Vec3, method: TransformMethod) -> Vec3 {
        self.0
            .iter()
            .rev()
            .fold(d, |d, link| link.undo_direction(d, method))
    }

    pub fn undo_ray(&self, r: Ray, method: TransformMethod) -> Ray {
        Ray {
            origin: self.undo_point(r.origin, method),
            direction: self.undo_direction(r.direction, method),
        }
    }
}
