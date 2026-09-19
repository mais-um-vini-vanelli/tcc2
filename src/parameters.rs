//! Parâmetros de renderização, agrupados por **estágio do pipeline**.
//!
//! Não é uma lista solta de bools: cada grupo corresponde a uma etapa do
//! pipeline, então a própria struct documenta o pipeline para quem está
//! aprendendo.

use crate::color::Rgb;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parameters {
    pub transform: TransformOptions,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransformOptions {
    pub method: TransformMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMethod {
    /// Escala, depois uma rotação por eixo (Euler X, Y, Z), depois translação,
    /// cada etapa aplicada separadamente.
    #[default]
    Stepwise,
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
            filtering: Filtering::Trilinear,
        }
    }
}

/// Filtro de texture sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filtering {
    /// Texel mais próximo, no nível de mip mais próximo. Barato, visivelmente
    /// serrilhado.
    Nearest,
    /// Interpolação entre os quatro texels vizinhos, no nível de mip mais
    /// próximo.
    Bilinear,
    /// Bilinear em dois níveis de mip, interpolados entre si. Evita mudanças bruscas de resolução
    /// na fronteira entre mip levels.
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
    /// Onde a construção corta cada nó da BVH.
    pub split: BvhSplit,
}

impl Default for AccelerationOptions {
    fn default() -> Self {
        AccelerationOptions {
            bvh: true,
            split: BvhSplit::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BvhSplit {
    /// Metade dos itens pra cada lado, no eixo mais comprido. A árvore sai
    /// balanceada, mas cega ao tamanho das caixas.
    Median,
    /// O corte de menor custo esperado pela *Surface Area Heuristic*
    /// (MacDonald e Booth, 1990), que também decide quando parar de dividir.
    #[default]
    Sah,
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

/// Saída da imagem: dimensões, cor de fundo e como a radiância calculada, que
/// não tem teto, vira uma cor entre 0 e 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutputOptions {
    pub width: u32,
    pub height: u32,
    /// Cor dos pixels que nenhum primitivo cobriu.
    pub background: Option<Rgb>,
    /// Multiplica a radiância antes do tone mapping, como a exposição de uma
    /// câmera: clareia ou escurece a imagem sem mexer nas luzes.
    pub exposure: f32,
    pub tonemap: Tonemap,
}

impl Default for OutputOptions {
    fn default() -> Self {
        OutputOptions {
            width: 800,
            height: 600,
            background: Some(Rgb::BLACK),
            exposure: 1.0,
            tonemap: Tonemap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tonemap {
    /// Nada: o que passa de 1 é cortado canal por canal na gravação. Estoura
    /// os claros e desvia a cor
    #[default]
    None,
    /// `c / (1 + c)` em cada canal (Reinhard et al., 2002). Quase a identidade
    /// nos escuros, e os claros se aproximam de 1 sem chegar.
    Reinhard,
}
