//! Textura, a pirâmide de mipmaps (Williams, 1983) e os filtros.
//!
//! Teoria: texturas.md

use std::sync::OnceLock;

use crate::color::Rgb;
use crate::math::Vec2;
use crate::parameters::Filtering;
use crate::render::ColorBuffer;

/// Imagem de textura em memória.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    base: ColorBuffer,
    mipmaps: OnceLock<Mipmaps>,
}

// Teoria: texturas.md#a-pirâmide
#[derive(Debug, Clone, PartialEq)]
pub struct Mipmaps {
    /// O nível 0 primeiro. Cada um tem metade dos lados do anterior, até 1x1.
    levels: Vec<ColorBuffer>,
}

// Teoria: texturas.md#a-footprint-do-pixel
/// Quanto a uv anda quando o pixel anda um passo em x (`dx`) e um em y (`dy`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Footprint {
    pub dx: Vec2,
    pub dy: Vec2,
}

impl Texture {
    /// Erro se `texels` não tiver exatamente `width * height` elementos.
    pub fn new(width: u32, height: u32, texels: Vec<Rgb>) -> Result<Self, TextureError> {
        let (expected, found) = (width as usize * height as usize, texels.len());

        let base = ColorBuffer::from_pixels(width, height, texels)
            .ok_or(TextureError::WrongPixelCount { expected, found })?;

        Ok(Texture {
            base,
            mipmaps: OnceLock::new(),
        })
    }

    pub fn width(&self) -> u32 {
        self.base.width()
    }

    pub fn height(&self) -> u32 {
        self.base.height()
    }

    /// Os texels do nível 0.
    pub fn texels(&self) -> &[Rgb] {
        self.base.pixels()
    }

    /// Um texel do nível 0.
    pub fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        self.base.get(x, y)
    }

    /// A pirâmide, construída na primeira chamada.
    pub fn generate_mipmaps(&self) {
        self.mipmaps.get_or_init(|| Mipmaps::new(&self.base));
    }

    pub fn miplevels(&self) -> usize {
        self.mipmaps.get().unwrap().levels.len()
    }

    /// Lê a textura com o filtro pedido. Os filtros com mipmap escolhem o nível
    /// pela footprint do pixel; os outros dois, ficam no 0.
    pub fn sample(&self, uv: Vec2, footprint: Option<Footprint>, filtering: Filtering) -> Rgb {
        // Teoria: texturas.md#os-filtros
        match (filtering, footprint) {
            (Filtering::Nearest, _) => nearest(&self.base, uv),
            // Sem footprint não há nível pra escolher, e os dois filtros com
            // mipmap caem no 0, que é a própria imagem: é o caso do ray tracer,
            // que assim nunca constrói a pirâmide.
            (Filtering::Bilinear, _) | (_, None) => bilinear(&self.base, uv),
            (Filtering::MipmapBilinear, Some(footprint)) => {
                self.mipmaps.get().unwrap().mipmap_bilinear(uv, footprint)
            }
            (Filtering::Trilinear, Some(footprint)) => {
                self.mipmaps.get().unwrap().trilinear(uv, footprint)
            }
        }
    }
}

impl Mipmaps {
    fn new(base: &ColorBuffer) -> Self {
        let mut levels = vec![base.clone()];

        while let Some(anterior) = levels.last().filter(|n| n.width() > 1 || n.height() > 1) {
            levels.push(reduz(anterior));
        }

        Mipmaps { levels }
    }

    // Teoria: texturas.md#do-footprint-para-o-nível
    /// O nível `λ` em que um texel cobre o que o pixel cobre.
    pub fn level_of_detail(&self, footprint: Footprint) -> f32 {
        // A uv vai de [0, 1], então multiplicar por width/height dá o deslocamento
        // em texels.
        let texels_em_dx = Vec2::new(
            footprint.dx.x * self.levels[0].width() as f32,
            footprint.dx.y * self.levels[0].height() as f32,
        )
        .length();

        let texels_em_dy = Vec2::new(
            footprint.dy.x * self.levels[0].width() as f32,
            footprint.dy.y * self.levels[0].height() as f32,
        )
        .length();

        // O nível que a gente quer é o log2 do maior dos dois deslocamentos.
        self.clamp_miplevel(texels_em_dx.max(texels_em_dy).log2())
    }

    /// Os quatro texels vizinhos, no nível de mip mais próximo.
    fn mipmap_bilinear(&self, uv: Vec2, footprint: Footprint) -> Rgb {
        bilinear(
            &self.levels[self.level_of_detail(footprint).round() as usize],
            uv,
        )
    }

    /// Bilinear nos dois níveis vizinhos, interpolados entre si.
    fn trilinear(&self, uv: Vec2, footprint: Footprint) -> Rgb {
        let lambda = self.level_of_detail(footprint);
        let d = lambda.floor() as usize;
        let frac = lambda - d as f32;

        if frac == 0.0 || d == self.levels.len() - 1 {
            bilinear(&self.levels[d], uv)
        } else {
            bilinear(&self.levels[d], uv).lerp(bilinear(&self.levels[d + 1], uv), frac)
        }
    }

    fn clamp_miplevel(&self, lambda: f32) -> f32 {
        lambda.max(0.0).min((self.levels.len() - 1) as f32)
    }
}

// Teoria: texturas.md#nearest-o-texel-que-contém-a-uv
/// O texel que contém `uv`. uv fora de [0,1] faz wrap, e `v` cresce para baixo,
/// como nas imagens.
fn nearest(nivel: &ColorBuffer, uv: Vec2) -> Rgb {
    let x = (repete(uv.x) * nivel.width() as f32) as u32;
    let y = (repete(uv.y) * nivel.height() as f32) as u32;

    // O produto acima pode chegar em width/height.
    nivel
        .get(
            x.min(nivel.width().saturating_sub(1)),
            y.min(nivel.height().saturating_sub(1)),
        )
        .unwrap_or(Rgb::BLACK)
}

// Teoria: texturas.md#bilinear-os-quatro-vizinhos
/// Os quatro texels em volta de `uv`, pesados pela distância até o centro de
/// cada um.
fn bilinear(nivel: &ColorBuffer, uv: Vec2) -> Rgb {
    if nivel.width() == 0 || nivel.height() == 0 {
        return Rgb::BLACK;
    }

    let (largura, altura) = (nivel.width() as i32, nivel.height() as i32);

    // O centro do texel i fica em (i + 0.5) / largura: daí o -0.5.
    let x = repete(uv.x) * largura as f32 - 0.5;
    let y = repete(uv.y) * altura as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);

    // O vizinho da borda dá a volta, pelo mesmo wrap da uv.
    let texel = |i: f32, j: f32| {
        let i = (i as i32).rem_euclid(largura) as u32;
        let j = (j as i32).rem_euclid(altura) as u32;
        nivel.get(i, j).unwrap_or(Rgb::BLACK)
    };

    let de_cima = texel(x0, y0).lerp(texel(x0 + 1.0, y0), fx);
    let de_baixo = texel(x0, y0 + 1.0).lerp(texel(x0 + 1.0, y0 + 1.0), fx);

    de_cima.lerp(de_baixo, fy)
}

// Teoria: texturas.md#a-pirâmide
/// O nível seguinte: cada texel é a média de um bloco 2x2 deste.
fn reduz(nivel: &ColorBuffer) -> ColorBuffer {
    let largura = (nivel.width() / 2).max(1);
    let altura = (nivel.height() / 2).max(1);

    let mut reduzido = ColorBuffer::new(largura, altura);

    for y in 0..altura {
        for x in 0..largura {
            let (i, j) = (2 * x, 2 * y);

            let texel = |i: u32, j: u32| {
                // Precisa usar esse min pra quando a textura não é quadrada: 8×2 → 4×1 → 2×1 → 1×1
                nivel
                    .get(i.min(nivel.width() - 1), j.min(nivel.height() - 1))
                    .unwrap_or(Rgb::BLACK)
            };

            reduzido.set(
                x,
                y,
                (texel(i, j) + texel(i + 1, j) + texel(i, j + 1) + texel(i + 1, j + 1)) / 4.0,
            );
        }
    }

    reduzido
}

fn repete(c: f32) -> f32 {
    c - c.floor()
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
