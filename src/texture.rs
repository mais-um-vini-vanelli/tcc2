//! Textura, a pirâmide de mipmaps (Williams, 1983) e os filtros.

use crate::color::Rgb;
use crate::math::Vec2;
use crate::parameters::Filtering;

/// Imagem de textura em memória.
///
/// A pirâmide de mipmaps é gerada na construção.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// O nível 0 primeiro. Cada um tem metade dos lados do anterior, até 1x1.
    levels: Vec<Nivel>,
}

#[derive(Debug, Clone, PartialEq)]
struct Nivel {
    width: u32,
    height: u32,
    texels: Vec<Rgb>,
}

/// Quanto a uv anda quando o pixel anda um passo em x (`dx`) e um em y (`dy`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Footprint {
    pub dx: Vec2,
    pub dy: Vec2,
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

        let mut levels = vec![Nivel {
            width,
            height,
            texels,
        }];

        while let Some(anterior) = levels.last().filter(|n| n.width > 1 || n.height > 1) {
            levels.push(anterior.reduz());
        }

        Ok(Texture { levels })
    }

    pub fn width(&self) -> u32 {
        self.levels[0].width
    }

    pub fn height(&self) -> u32 {
        self.levels[0].height
    }

    /// Os texels do nível 0.
    pub fn texels(&self) -> &[Rgb] {
        &self.levels[0].texels
    }

    /// Um texel do nível 0.
    pub fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        self.levels[0].texel(x, y)
    }

    pub fn miplevels(&self) -> usize {
        self.levels.len()
    }

    /// O nível `λ` em que um texel cobre o que o pixel cobre.
    pub fn level_of_detail(&self, footprint: Footprint) -> f32 {
        // footprint.dx diz o deslocamento no plano UV que acontece quando x anda 1 unidade pra + ou pra -; as coordenadas uv vão
        // de [0, 1], então multiplicamos u * width e v * height pra ter o deslocamento em texels
        let texels_em_dx = Vec2::new(
            footprint.dx.x * self.levels[0].width as f32,
            footprint.dx.y * self.levels[0].height as f32,
        )
        .length();

        let texels_em_dy = Vec2::new(
            footprint.dy.x * self.levels[0].width as f32,
            footprint.dy.y * self.levels[0].height as f32,
        )
        .length();

        // com os dois deslocamentos, temos o quadrado de texels da textura original (miplevel 0) que estão dentro do pixel; agora
        // é descobrir o miplevel em que um texel tem o tamanho desse quadrado.
        //
        // cada miplevel tem metade da altura e largura do anterior:
        // > width_k = width_0 / 2^k
        // queremos o miplevel com width_k = 1:
        // > 1 = width_0 / 2^k
        // > width_0 = 2^k
        // > 2^k = width_0
        // aplicando log2 dos dois lados:
        // > k = log2(width_0)
        //
        // mas os dois vetores não precisam ter o mesmo tamanho, então usamos o k olhando apenas para o maior entre width e height
        self.clamp_miplevel(texels_em_dx.max(texels_em_dy).log2())
    }

    /// Lê a textura com o filtro pedido. Os filtros com mipmap escolhem o nível
    /// pela footprint do pixel; os outros dois, ficam no 0.
    pub fn sample(&self, uv: Vec2, footprint: Option<Footprint>, filtering: Filtering) -> Rgb {
        let nivel = || footprint.map_or(0.0, |footprint| self.level_of_detail(footprint));

        match filtering {
            Filtering::Nearest => self.levels[0].nearest(uv),
            Filtering::Bilinear => self.levels[0].bilinear(uv),
            Filtering::MipmapBilinear => self.levels[nivel().round() as usize].bilinear(uv),
            Filtering::Trilinear => {
                let lambda = nivel();
                let d = lambda.floor() as usize;
                let frac = lambda - d as f32;

                if frac == 0.0 || d == self.levels.len() - 1 {
                    self.levels[d].bilinear(uv)
                } else {
                    self.levels[d]
                        .bilinear(uv)
                        .lerp(self.levels[d + 1].bilinear(uv), frac)
                }
            }
        }
    }

    fn clamp_miplevel(&self, lambda: f32) -> f32 {
        lambda.max(0.0).min((self.levels.len() - 1) as f32)
    }
}

impl Nivel {
    fn texel(&self, x: u32, y: u32) -> Option<Rgb> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.texels[(y * self.width + x) as usize])
    }

    /// O texel que contém `uv`. uv fora de [0,1] faz wrap, e `v` cresce para
    /// baixo, como nas imagens.
    fn nearest(&self, uv: Vec2) -> Rgb {
        let x = (Self::repete(uv.x) * self.width as f32) as u32;
        let y = (Self::repete(uv.y) * self.height as f32) as u32;

        // O produto acima pode chegar em width/height.
        self.texel(
            x.min(self.width.saturating_sub(1)),
            y.min(self.height.saturating_sub(1)),
        )
        .unwrap_or(Rgb::BLACK)
    }

    /// Os quatro texels em volta de `uv`, pesados pela distância até o centro
    /// de cada um.
    fn bilinear(&self, uv: Vec2) -> Rgb {
        if self.width == 0 || self.height == 0 {
            return Rgb::BLACK;
        }

        let (largura, altura) = (self.width as i32, self.height as i32);

        // O centro do texel i fica em (i + 0.5) / largura: daí o -0.5.
        let x = Self::repete(uv.x) * largura as f32 - 0.5;
        let y = Self::repete(uv.y) * altura as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);

        // O vizinho da borda dá a volta, pelo mesmo wrap da uv.
        let texel = |i: f32, j: f32| {
            let i = (i as i32).rem_euclid(largura) as u32;
            let j = (j as i32).rem_euclid(altura) as u32;
            self.texel(i, j).unwrap_or(Rgb::BLACK)
        };

        let de_cima = texel(x0, y0).lerp(texel(x0 + 1.0, y0), fx);
        let de_baixo = texel(x0, y0 + 1.0).lerp(texel(x0 + 1.0, y0 + 1.0), fx);

        de_cima.lerp(de_baixo, fy)
    }

    /// O nível seguinte: cada texel é a média de um bloco 2x2 deste.
    fn reduz(&self) -> Nivel {
        let largura = (self.width / 2).max(1);
        let altura = (self.height / 2).max(1);

        let mut texels = Vec::with_capacity(largura as usize * altura as usize);

        for y in 0..altura {
            for x in 0..largura {
                let (i, j) = (2 * x, 2 * y);

                let texel = |i: u32, j: u32| {
                    // Precisa usar esse min pra quando a textura não é quadrada: 8×2 → 4×1 → 2×1 → 1×1
                    self.texel(i.min(self.width - 1), j.min(self.height - 1))
                        .unwrap_or(Rgb::BLACK)
                };

                texels.push(
                    (texel(i, j) + texel(i + 1, j) + texel(i, j + 1) + texel(i + 1, j + 1)) / 4.0,
                );
            }
        }

        Nivel {
            width: largura,
            height: altura,
            texels,
        }
    }

    fn repete(c: f32) -> f32 {
        c - c.floor()
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
