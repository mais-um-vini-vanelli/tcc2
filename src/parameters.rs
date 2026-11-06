use crate::render::RaySegment;

#[derive(Clone, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct Parameters<'a> {
    pub transform: TransformOptions,
    pub rasterization: RasterizationOptions,
    pub shading: ShadingOptions,
    pub ray_tracing: RayTracingOptions<'a>,
    pub output: OutputOptions,
}

impl Parameters<'_> {
    /// Parâmetros padrão com a resolução informada.
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
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct TransformOptions {
    pub method: TransformMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize))]
#[cfg_attr(feature = "io", serde(rename_all = "snake_case"))]
pub enum TransformMethod {
    /// Escala, depois uma rotação por eixo (Euler X, Y, Z), depois translação.
    #[default]
    Stepwise,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct RasterizationOptions {
    /// Descarta triângulos de costas para câmera.
    pub backface_cull: bool,
    /// Correção de perspectiva na interpolação das baricêntricas.
    pub perspective_correct: bool,
    /// Teste de profundidade (z-buffer).
    pub depth_test: bool,
    /// Chamado pra cada fragmento, antes do depth test. Não faz nada no
    /// render: existe só como lugar pra colocar um breakpoint.
    #[cfg_attr(feature = "io", serde(skip))]
    pub on_fragment: Option<fn(u32, u32)>,
    /// Chamado pra cada triângulo dos objetos com geometria, no começo da pipeline. A
    /// função recebe (index da instância conforme `Scene::flatten`, index da
    /// face na malha que `Geometry::tessellate` devolve).
    #[cfg_attr(feature = "io", serde(skip))]
    pub on_triangle: Option<fn(usize, usize)>,
}

impl Default for RasterizationOptions {
    fn default() -> Self {
        RasterizationOptions {
            backface_cull: true,
            perspective_correct: true,
            depth_test: true,
            on_fragment: None,
            on_triangle: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize))]
#[cfg_attr(feature = "io", serde(rename_all = "snake_case"))]
pub enum TextureFilter {
    /// Texel mais próximo, no miplevel 0. Barato, visivelmente serrilhado.
    Nearest,
    /// Interpolação entre os quatro texels vizinhos, no miplevel 0.
    Bilinear,
    /// Interpolação entre os quatro texels vizinhos, no nível de mip mais
    /// próximo.
    MipmapBilinear,
    /// Bilinear em dois níveis de mip, interpolados entre si. Evita mudanças
    /// bruscas de resolução na fronteira entre mip levels.
    #[default]
    Trilinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct ShadingOptions {
    pub reflectance_model: ReflectanceModel,
    pub texture_filter: TextureFilter,
}

/// Qual BRDF usar. Uma BRDF converte a luz que chega em luz que sai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize))]
#[cfg_attr(feature = "io", serde(rename_all = "snake_case"))]
pub enum ReflectanceModel {
    #[default]
    BlinnPhong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize))]
#[cfg_attr(feature = "io", serde(rename_all = "snake_case"))]
pub enum Acceleration {
    /// Testa cada raio contra todos os triângulos de todas os objetos.
    BruteForce,
    /// BVH com metade dos itens pra cada lado, no eixo mais comprido.
    BvhMedian,
    /// BVH com o corte de menor custo esperado pela *Surface Area Heuristic*
    /// (MacDonald e Booth, 1990).
    #[default]
    BvhSah,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct RayTracingOptions<'a> {
    /// Profundidade máxima da recursão de reflexão/refração.
    pub max_depth: u32,
    /// Lançar raios de sombra.
    pub shadows: bool,
    /// Lançar raios secundários de reflexão.
    pub reflections: bool,
    /// Lançar raios secundários de refração.
    pub refractions: bool,
    pub acceleration: Acceleration,
    /// Chamado uma vez por pixel, antes do raio primário. Não faz nada no
    /// render.
    #[cfg_attr(feature = "io", serde(skip))]
    pub on_ray: Option<&'a dyn Fn(u32, u32)>,
    /// Chamado pra cada raio lançado, depois do teste de interseção. Não faz
    /// nada no render.
    #[cfg_attr(feature = "io", serde(skip))]
    pub on_segment: Option<&'a dyn Fn(RaySegment)>,
}

impl Default for RayTracingOptions<'_> {
    fn default() -> Self {
        RayTracingOptions {
            max_depth: 5,
            shadows: true,
            reflections: true,
            refractions: true,
            acceleration: Acceleration::default(),
            on_ray: None,
            on_segment: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "io", derive(serde::Deserialize), serde(default))]
pub struct OutputOptions {
    pub width: u32,
    pub height: u32,
    /// Multiplica a radiância antes do tonemap, como a exposição de uma
    /// câmera: clareia ou escurece a imagem sem mexer nas luzes.
    pub exposure: f32,
    pub tonemap: Tonemap,
}

impl Default for OutputOptions {
    fn default() -> Self {
        OutputOptions {
            width: 800,
            height: 600,
            exposure: 0.0,
            tonemap: Tonemap::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "io", derive(serde::Deserialize))]
#[cfg_attr(feature = "io", serde(rename_all = "snake_case"))]
pub enum Tonemap {
    /// Nada: o que passa de 1 é cortado canal por canal na gravação.
    #[default]
    None,
    /// `c / (1 + c)` em cada canal (Reinhard et al., 2002).
    Reinhard,
}
