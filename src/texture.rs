use std::sync::OnceLock;

use crate::color::Rgb;
use crate::math::Vec2;
use crate::parameters::TextureFilter;
use crate::render::ColorBuffer;

#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    base: ColorBuffer,
    mipmaps: OnceLock<Mipmaps>,
    pub wrap: TextureWrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureWrap {
    #[default]
    Repeat,
    Clamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mipmaps {
    levels: Vec<ColorBuffer>,
}

/// Quanto o uv anda quando o pixel anda um passo em x (`dx`) e um em y (`dy`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Footprint {
    pub dx: Vec2,
    pub dy: Vec2,
}

impl Texture {
    pub fn new(width: u32, height: u32, texels: Vec<Rgb>) -> Result<Self, TextureError> {
        let (expected, found) = (width as usize * height as usize, texels.len());

        let base = ColorBuffer::from_pixels(width, height, texels)
            .ok_or(TextureError::WrongPixelCount { expected, found })?;

        Ok(Texture {
            base,
            mipmaps: OnceLock::new(),
            wrap: TextureWrap::default(),
        })
    }

    pub fn width(&self) -> u32 {
        self.base.width()
    }

    pub fn height(&self) -> u32 {
        self.base.height()
    }

    pub fn texels(&self) -> &[Rgb] {
        self.base.pixels()
    }

    pub fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        self.base.get(x, y)
    }

    pub fn generate_mipmaps(&self) {
        self.mipmaps.get_or_init(|| Mipmaps::build(&self.base));
    }

    pub fn mipmaps(&self) -> usize {
        self.mipmaps.get().unwrap().levels.len()
    }

    pub fn sample(
        &self,
        uv: Vec2,
        footprint: Option<Footprint>,
        texture_filter: TextureFilter,
    ) -> Rgb {
        match texture_filter {
            TextureFilter::Nearest => Self::nearest(&self.base, uv, self.wrap),
            TextureFilter::Bilinear => Self::bilinear(&self.base, uv, self.wrap),

            TextureFilter::MipmapBilinear => {
                if let Some(footprint) = footprint {
                    self.mipmaps
                        .get()
                        .unwrap()
                        .bilinear(uv, footprint, self.wrap)
                } else {
                    Self::bilinear(&self.base, uv, self.wrap)
                }
            }

            TextureFilter::Trilinear => {
                if let Some(footprint) = footprint {
                    self.mipmaps
                        .get()
                        .unwrap()
                        .trilinear(uv, footprint, self.wrap)
                } else {
                    Self::bilinear(&self.base, uv, self.wrap)
                }
            }
        }
    }

    fn nearest(texture: &ColorBuffer, uv: Vec2, wrap: TextureWrap) -> Rgb {
        let (width, height) = (texture.width() as i32, texture.height() as i32);

        let x = (uv.x * width as f32).floor() as i32;
        let y = (uv.y * height as f32).floor() as i32;

        texture
            .get(
                Self::address(x, width, wrap),
                Self::address(y, height, wrap),
            )
            .unwrap_or(Rgb::BLACK)
    }

    fn bilinear(texture: &ColorBuffer, uv: Vec2, wrap: TextureWrap) -> Rgb {
        let (width, height) = (texture.width() as i32, texture.height() as i32);

        // 1.5 é o centro do texel 1, por isso o -0.5.
        let x = uv.x * width as f32 - 0.5;
        let y = uv.y * height as f32 - 0.5;

        let (x0, y0) = (x.floor(), y.floor());
        // A distância entre (x, y) do centro dos texels anteriores, no intervalo [0,1).
        let (fx, fy) = (x - x0, y - y0);

        let texel = |i: f32, j: f32| {
            texture
                .get(
                    Self::address(i as i32, width, wrap),
                    Self::address(j as i32, height, wrap),
                )
                .unwrap_or(Rgb::BLACK)
        };

        let top_row = texel(x0, y0).lerp(texel(x0 + 1.0, y0), fx);
        let bottom_row = texel(x0, y0 + 1.0).lerp(texel(x0 + 1.0, y0 + 1.0), fx);
        top_row.lerp(bottom_row, fy)
    }

    fn address(i: i32, size: i32, wrap: TextureWrap) -> u32 {
        if size <= 0 {
            return 0;
        }

        match wrap {
            TextureWrap::Repeat => i.rem_euclid(size) as u32,
            TextureWrap::Clamp => i.clamp(0, size - 1) as u32,
        }
    }
}

impl Mipmaps {
    fn build(base: &ColorBuffer) -> Self {
        let mut levels = vec![base.clone()];

        while let Some(previous) = levels.last().filter(|n| n.width() > 1 || n.height() > 1) {
            levels.push(Self::miplevel(previous));
        }

        Mipmaps { levels }
    }

    /// Monta um miplevel. Cada texel é a média de um bloco 2x2 do anterior.
    fn miplevel(previous: &ColorBuffer) -> ColorBuffer {
        let width = (previous.width() / 2).max(1);
        let height = (previous.height() / 2).max(1);

        let mut level = ColorBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let (i, j) = (2 * x, 2 * y);

                let texel = |i: u32, j: u32| {
                    previous
                        // O min() pra quando a textura não é quadrada: 8×2 → 4×1 → 2×1
                        .get(i.min(previous.width() - 1), j.min(previous.height() - 1))
                        .unwrap_or(Rgb::BLACK)
                };

                level.set(
                    x,
                    y,
                    (texel(i, j) + texel(i + 1, j) + texel(i, j + 1) + texel(i + 1, j + 1)) / 4.0,
                );
            }
        }

        level
    }

    /// O nível `λ` em que um texel tem um tamanho parecido com o pixel.
    pub fn level_of_detail(&self, footprint: Footprint) -> f32 {
        let dx_in_texels = Vec2::new(
            footprint.dx.x * self.levels[0].width() as f32,
            footprint.dx.y * self.levels[0].height() as f32,
        )
        .length();

        let dy_in_texels = Vec2::new(
            footprint.dy.x * self.levels[0].width() as f32,
            footprint.dy.y * self.levels[0].height() as f32,
        )
        .length();

        // Usa o maior dos dois deslocamentos.
        let lambda = dx_in_texels.max(dy_in_texels).log2();
        lambda.max(0.0).min(((&self).levels.len() - 1) as f32)
    }

    /// Os quatro texels vizinhos, no nível de mip mais próximo.
    fn bilinear(&self, uv: Vec2, footprint: Footprint, wrap: TextureWrap) -> Rgb {
        Texture::bilinear(
            &self.levels[self.level_of_detail(footprint).round() as usize],
            uv,
            wrap,
        )
    }

    /// Bilinear nos dois níveis vizinhos, interpolados entre si.
    fn trilinear(&self, uv: Vec2, footprint: Footprint, wrap: TextureWrap) -> Rgb {
        let lambda = self.level_of_detail(footprint);
        let d = lambda.floor() as usize;
        let frac = lambda - d as f32;

        if frac == 0.0 || d == self.levels.len() - 1 {
            Texture::bilinear(&self.levels[d], uv, wrap)
        } else {
            Texture::bilinear(&self.levels[d], uv, wrap)
                .lerp(Texture::bilinear(&self.levels[d + 1], uv, wrap), frac)
        }
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
                write!(f, "a textura espera {expected} pixels, recebeu {found}")
            }
        }
    }
}

impl std::error::Error for TextureError {}
