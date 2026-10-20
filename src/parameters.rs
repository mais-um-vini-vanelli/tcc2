#[derive(Debug, Clone, Default)]
pub struct Parameters {
    pub transform: TransformOptions,
    pub rasterization: RasterizationOptions,
    pub shading: ShadingOptions,
    pub ray_tracing: RayTracingOptions,
    pub output: OutputOptions,
}

impl Parameters {
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
pub struct TransformOptions {
    pub method: TransformMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformMethod {
    /// Escala, depois uma rotação por eixo (Euler X, Y, Z), depois translação.
    #[default]
    Stepwise,
}

#[derive(Debug, Clone, Copy)]
pub struct RasterizationOptions {
    /// Descarta triângulos de costas para câmera.
    pub backface_cull: bool,
    /// Correção de perspectiva na interpolação das baricêntricas.
    pub perspective_correct: bool,
    /// Teste de profundidade (z-buffer).
    pub depth_test: bool,
    /// Chamado pra cada fragmento, antes do depth test. Não faz nada no
    /// render: existe só como lugar pra colocar um breakpoint.
    pub on_fragment: Option<fn(u32, u32)>,
    /// Chamado pra cada triângulo dos objetos com geometria, no começo da pipeline. A
    /// função recebe (index da instância conforme `Scene::flatten`, index da
    /// face na malha que `Geometry::tessellate` devolve).
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
pub struct ShadingOptions {
    pub reflectance_model: ReflectanceModel,
    pub texture_filter: TextureFilter,
}

/// Qual BRDF usar. Uma BRDF converte a luz que chega em luz que sai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReflectanceModel {
    #[default]
    BlinnPhong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccelerationOptions {
    /// Usar BVHs para economizar testes de interseção dos raios com a geometria.
    pub enabled: bool,
    /// Como dividir os itens entre os nós da BVH.
    pub bvh_split: BvhSplit,
    /// Desenhar as caixas dos nós da BVH até esta profundidade. `None` = não desenha nada.
    pub wireframe: Option<u32>,
}

impl Default for AccelerationOptions {
    fn default() -> Self {
        AccelerationOptions {
            enabled: true,
            bvh_split: BvhSplit::default(),
            wireframe: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BvhSplit {
    /// Metade dos itens pra cada lado, no eixo mais comprido.
    Median,
    /// O corte de menor custo esperado pela *Surface Area Heuristic*
    /// (MacDonald e Booth, 1990).
    #[default]
    Sah,
}

#[derive(Debug, Clone, Copy)]
pub struct RayTracingOptions {
    /// Profundidade máxima da recursão de reflexão/refração.
    pub max_depth: u32,
    /// Lançar raios de sombra.
    pub shadows: bool,
    /// Lançar raios secundários de reflexão.
    pub reflections: bool,
    /// Lançar raios secundários de refração.
    pub refractions: bool,
    pub acceleration: AccelerationOptions,
    /// Chamado uma vez por pixel, antes do raio primário. Não faz nada no
    /// render: existe só como lugar pra colocar um breakpoint.
    pub on_ray: Option<fn(u32, u32)>,
}

impl Default for RayTracingOptions {
    fn default() -> Self {
        RayTracingOptions {
            max_depth: 5,
            shadows: true,
            reflections: true,
            refractions: true,
            acceleration: AccelerationOptions::default(),
            on_ray: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
pub enum Tonemap {
    /// Nada: o que passa de 1 é cortado canal por canal na gravação.
    #[default]
    None,
    /// `c / (1 + c)` em cada canal (Reinhard et al., 2002).
    Reinhard,
}
