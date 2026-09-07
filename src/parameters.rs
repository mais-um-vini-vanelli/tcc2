//! Parâmetros de renderização, agrupados por **estágio do pipeline**.
//!
//! Não é uma lista solta de bools: cada grupo corresponde a uma etapa do
//! pipeline, então a própria struct documenta o pipeline para quem está
//! aprendendo.

use crate::color::Rgb;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parameters {
    pub rasterization: RasterizationOptions,
    pub clipping: ClippingOptions,
    pub texturing: TextureOptions,
    pub shading: ShadingOptions,
    pub ray_tracing: RayTracingOptions,
    pub output: OutputOptions,
}

impl Parameters {
    /// Parâmetros padrão com a resolução de saída dada.
    pub fn with_resolution(width: u32, height: u32) -> Self {
        Parameters {
            output: OutputOptions {
                width,
                height,
                ..OutputOptions::default()
            },
            ..Parameters::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterizationOptions {
    /// Amostras por pixel do *multisampling*. `None` = uma amostra por pixel.
    pub msaa: Option<u8>,
    /// Descarta triângulos de face oposta à câmera.
    pub backface_cull: bool,
    /// Desenha só as arestas, sem preencher o interior.
    pub wireframe: bool,
    /// Correção de perspectiva na interpolação das baricêntricas. Desligar
    /// mostra a distorção clássica em triângulos inclinados.
    pub perspective_correct: bool,
    /// Teste de profundidade (z-buffer). Desligar mostra objetos distantes
    /// desenhados por cima dos próximos, conforme a ordem da cena.
    pub depth_test: bool,
}

impl Default for RasterizationOptions {
    fn default() -> Self {
        RasterizationOptions {
            msaa: None,
            backface_cull: true,
            wireframe: false,
            perspective_correct: true,
            depth_test: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClippingOptions {
    pub enabled: bool,
}

impl Default for ClippingOptions {
    fn default() -> Self {
        ClippingOptions { enabled: true }
    }
}

/// Texturização e mipmapping (Williams, 1983).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureOptions {
    /// Gera e usa a pirâmide de mipmaps. Desligar mostra o aliasing que ela
    /// existe para resolver.
    pub mipmaps: bool,
    pub filtering: Filtering,
}

impl Default for TextureOptions {
    fn default() -> Self {
        TextureOptions {
            mipmaps: true,
            filtering: Filtering::Bilinear,
        }
    }
}

/// Filtro de texture sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filtering {
    /// Texel mais próximo. Barato, visivelmente serrilhado.
    Nearest,
    /// Interpolação entre os quatro texels vizinhos, dentro de um nível de mip.
    Bilinear,
    /// Bilinear em dois níveis de mip, interpolados entre si.
    Trilinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShadingOptions {
    // Com que frequência a cor é avaliada
    pub frequency: ShadingFrequency,
    // Qual modelo de reflectância a avalia a cor
    pub reflectance: ReflectanceModel,
}

/// Onde a cor é avaliada — por face, por vértice ou por fragmento. Não diz nada
/// sobre *como* ela é calculada; isso é o `ReflectanceModel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadingFrequency {
    /// Uma cor por triângulo, da normal geométrica da face. A malha aparece
    /// facetada, e cada face da tesselação fica visível.
    Flat,
    /// Uma cor por vértice, interpolada no interior. Barato, mas o brilho
    /// especular só existe se cair em cima de um vértice.
    Gouraud,
    /// A normal é interpolada e a cor calculada por pixel. É o padrão.
    #[default]
    Phong,
}

/// Qual BRDF converte a luz que chega em luz que sai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReflectanceModel {
    #[default]
    BlinnPhong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccelerationOptions {
    /// Desligar cai para força bruta (todo raio testa todo primitivo).
    pub bvh: bool,
}

impl Default for AccelerationOptions {
    fn default() -> Self {
        AccelerationOptions { bvh: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RayTracingOptions {
    /// Profundidade máxima da recursão de reflexão/refração.
    pub max_depth: u32,
    /// Raios de sombra em direção às luzes.
    pub shadows: bool,
    /// Raios secundários de reflexão.
    pub reflections: bool,
    /// Raios secundários de refração.
    pub refractions: bool,
    pub acceleration: AccelerationOptions,
}

impl Default for RayTracingOptions {
    fn default() -> Self {
        RayTracingOptions {
            max_depth: 5,
            shadows: true,
            reflections: true,
            refractions: true,
            acceleration: AccelerationOptions::default(),
        }
    }
}

/// Saída da imagem: dimensões e cor de fundo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputOptions {
    pub width: u32,
    pub height: u32,
    /// Cor dos pixels que nenhum primitivo cobriu.
    pub background: Option<Rgb>,
}

impl Default for OutputOptions {
    fn default() -> Self {
        OutputOptions {
            width: 800,
            height: 600,
            background: Some(Rgb::BLACK),
        }
    }
}
