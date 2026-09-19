//! Cor em ponto flutuante.
//!
//! Internamente a cor é `f32` e **linear**. Converte pra `u8` só na saída, e
//! é nessa conversão que a codificação sRGB entra.

use std::ops::{Add, Div, Mul, Sub};

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

    /// A mesma função nos três canais.
    pub fn map(self, f: impl Fn(f32) -> f32) -> Rgb {
        Rgb::new(f(self.r), f(self.g), f(self.b))
    }

    pub fn clamped(self) -> Rgb {
        self.map(|c| c.clamp(0.0, 1.0))
    }

    /// A cor a `t` do caminho entre as duas.
    pub fn lerp(self, outra: Rgb, t: f32) -> Rgb {
        self + (outra - self) * t
    }

    /// Pixel sRGB -> cor linear.
    pub fn from_bytes(bytes: [u8; 3]) -> Self {
        Rgb::new(bytes[0] as f32, bytes[1] as f32, bytes[2] as f32)
            .map(|c| Self::srgb_para_linear(c / 255.0))
    }

    /// Cor linear -> pixel sRGB.
    pub fn to_bytes(self) -> [u8; 3] {
        let codificada = self.clamped().map(Self::linear_para_srgb);
        [codificada.r, codificada.g, codificada.b].map(|c| (c * 255.0 + 0.5) as u8)
    }

    // TODO: trocar pela curva sRGB completa (trecho reto + potência)
    const GAMMA: f32 = 2.2;

    fn linear_para_srgb(c: f32) -> f32 {
        c.powf(1.0 / Self::GAMMA)
    }

    fn srgb_para_linear(c: f32) -> f32 {
        c.powf(Self::GAMMA)
    }
}

impl Add for Rgb {
    type Output = Rgb;
    fn add(self, o: Rgb) -> Rgb {
        Rgb::new(self.r + o.r, self.g + o.g, self.b + o.b)
    }
}

impl Sub for Rgb {
    type Output = Rgb;
    fn sub(self, o: Rgb) -> Rgb {
        Rgb::new(self.r - o.r, self.g - o.g, self.b - o.b)
    }
}

impl Mul<Rgb> for Rgb {
    type Output = Rgb;
    fn mul(self, o: Rgb) -> Rgb {
        Rgb::new(self.r * o.r, self.g * o.g, self.b * o.b)
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
