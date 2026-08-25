//! Cor em ponto flutuante.
//!
//! Internamente a cor é `f32`. Converte pra `u8` só na saída.

use std::ops::{Add, Div, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0.0, 0.0, 0.0);
    pub const WHITE: Rgb = Rgb::new(1.0, 1.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Rgb { r, g, b }
    }

    /// Pra grayscale: usa a mesma intensidade nos três canais.
    pub const fn gray(v: f32) -> Self {
        Rgb::new(v, v, v)
    }

    pub fn clamped(self) -> Rgb {
        Rgb::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
        )
    }

    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        Rgb::new(
            bytes[0] as f32 / 255.0,
            bytes[1] as f32 / 255.0,
            bytes[2] as f32 / 255.0,
        )
    }

    pub fn to_bytes(self) -> [u8; 3] {
        let q = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        [q(self.r), q(self.g), q(self.b)]
    }
}

impl Add for Rgb {
    type Output = Rgb;
    fn add(self, o: Rgb) -> Rgb {
        Rgb::new(self.r + o.r, self.g + o.g, self.b + o.b)
    }
}

impl Mul<f32> for Rgb {
    type Output = Rgb;
    fn mul(self, s: f32) -> Rgb {
        Rgb::new(self.r * s, self.g * s, self.b * s)
    }
}

impl Mul<Rgb> for f32 {
    type Output = Rgb;
    fn mul(self, c: Rgb) -> Rgb {
        c * self
    }
}

impl Div<f32> for Rgb {
    type Output = Rgb;
    fn div(self, s: f32) -> Rgb {
        Rgb::new(self.r / s, self.g / s, self.b / s)
    }
}
