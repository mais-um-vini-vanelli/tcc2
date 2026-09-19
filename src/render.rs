//! A interface comum aos dois pipelines.

use std::time::Duration;

use crate::color::Rgb;
use crate::parameters::{OutputOptions, Parameters, Tonemap};
use crate::scene::Scene;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPass {
    /// A imagem final.
    Color,
    /// Profundidade da superfície visível, em unidades.
    Depth,
    /// Normal da superfície visível (debug de shading).
    Normal,
    /// Coordenadas de textura visíveis, para conferir mapeamento.
    Uv,
    /// Raios lançados por pixel, contando sombra e recursão. Só o ray tracer
    /// produz.
    RayCount,
    /// Nós de BVH testados por pixel, por todos os raios dele. Só o ray tracer
    /// produz, e só com a BVH ligada.
    BvhHeatmap,
}

impl RenderPass {
    /// Passes que os dois modos conseguem produzir — a base da validação
    /// cruzada entre `Rasterizer` e `RayTracer`.
    pub const SHARED: [RenderPass; 4] = [
        RenderPass::Color,
        RenderPass::Depth,
        RenderPass::Normal,
        RenderPass::Uv,
    ];

    /// Formato do buffer que este passe produz.
    pub fn buffer_kind(self) -> BufferKind {
        match self {
            RenderPass::Color | RenderPass::Normal | RenderPass::Uv => BufferKind::Color,
            RenderPass::Depth | RenderPass::RayCount | RenderPass::BvhHeatmap => BufferKind::Scalar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferKind {
    Color,
    Scalar,
}

/// Imagem de saída: cor em float, um `Rgb` por pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorBuffer {
    width: u32,
    height: u32,
    pixels: Vec<Rgb>,
}

impl ColorBuffer {
    /// Buffer preenchido com uma cor sólida.
    pub fn filled(width: u32, height: u32, color: Rgb) -> Self {
        let n = width as usize * height as usize;
        ColorBuffer {
            width,
            height,
            pixels: vec![color; n],
        }
    }

    pub fn new(width: u32, height: u32) -> Self {
        ColorBuffer::filled(width, height, Rgb::BLACK)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Rgb> {
        self.index(x, y).map(|i| self.pixels[i])
    }

    pub fn set(&mut self, x: u32, y: u32, color: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = color;
        }
    }

    /// O último estágio dos dois pipelines: exposição e tone mapping em todos
    /// os pixels.
    ///
    /// É parecido com a conversão de linear -> sRGB, mas resolve outro problema:
    /// - O tonemap pega valores de radiância da cena (cores f32 no intervalo [0, ∞]) e converte pra cores f32 no intervalo [0, 1].
    /// - A conversão pra sRGB pega as cores f32 no intervalo [0, 1] e converte pra u8 [0, 255], com a maior parte dos inteiros alocados pras cores escuras
    /// (valores menores).
    ///
    /// Reinhard é uma opção simples de tonemap que cumpre o que a gente espera de uma curva de tons:
    /// - Preto continua preto: f(0) = 0.
    /// - Nunca passa de 1, por maior que seja a entrada.
    /// - É sempre crescente: o mais claro na cena continua mais claro na imagem (o clamp ingênuo falha aqui, porque 2 e 40 viram o mesmo valor).
    /// - Os escuros ficam como estão, já que eles não eram o problema. Com c pequeno, 1 + c ≈ 1, então f(c) ≈ c.
    pub(crate) fn apply_output(&mut self, output: &OutputOptions) {
        for pixel in &mut self.pixels {
            *pixel = {
                let c = *pixel * output.exposure;
                match output.tonemap {
                    Tonemap::None => c,
                    Tonemap::Reinhard => c.map(|v| v / (1.0 + v)),
                }
            };
        }
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }
}

/// Buffer de uma grandeza escalar por pixel: profundidade, contagem de raios, nós de BVH testados.
#[derive(Debug, Clone, PartialEq)]
pub struct ScalarBuffer {
    width: u32,
    height: u32,
    values: Vec<f32>,
}

impl ScalarBuffer {
    pub fn filled(width: u32, height: u32, value: f32) -> Self {
        let n = width as usize * height as usize;
        ScalarBuffer {
            width,
            height,
            values: vec![value; n],
        }
    }

    pub fn new(width: u32, height: u32) -> Self {
        ScalarBuffer::filled(width, height, 0.0)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn values(&self) -> &[f32] {
        &self.values
    }

    pub fn get(&self, x: u32, y: u32) -> Option<f32> {
        self.index(x, y).map(|i| self.values[i])
    }

    pub fn set(&mut self, x: u32, y: u32, value: f32) {
        if let Some(i) = self.index(x, y) {
            self.values[i] = value;
        }
    }

    pub fn finite_range(&self) -> Option<(f32, f32)> {
        let mut range: Option<(f32, f32)> = None;

        for &v in &self.values {
            if v.is_finite() {
                range = Some(match range {
                    None => (v, v),
                    Some((lo, hi)) => (lo.min(v), hi.max(v)),
                });
            }
        }

        range
    }

    /// Visualiza como escala de cinza, normalizando linearmente o intervalo
    /// `[min, max]` para `[0, 1]`. Cuidado ao comparar duas imagens, porque um intervalo
    /// diferente vai gerar intensidades diferentes pro mesmo valor bruto de pixel.
    pub fn to_color(&self, range: Option<(f32, f32)>) -> ColorBuffer {
        let (lo, hi) = range.or_else(|| self.finite_range()).unwrap_or((0.0, 1.0));

        let span = hi - lo;

        let pixels = self
            .values
            .iter()
            .map(|&v| {
                if v.is_finite() {
                    let t = if span == 0.0 { 0.0 } else { (v - lo) / span };
                    Rgb::gray(t)
                } else {
                    Rgb::BLACK
                }
            })
            .collect();

        ColorBuffer {
            width: self.width,
            height: self.height,
            pixels,
        }
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }
}

/// Um buffer emprestado do `RenderResult`, no formato que o passe pede.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferRef<'a> {
    Color(&'a ColorBuffer),
    Scalar(&'a ScalarBuffer),
}

impl<'a> BufferRef<'a> {
    pub fn kind(self) -> BufferKind {
        match self {
            BufferRef::Color(_) => BufferKind::Color,
            BufferRef::Scalar(_) => BufferKind::Scalar,
        }
    }

    pub fn as_color(self) -> Option<&'a ColorBuffer> {
        match self {
            BufferRef::Color(b) => Some(b),
            BufferRef::Scalar(_) => None,
        }
    }

    pub fn as_scalar(self) -> Option<&'a ScalarBuffer> {
        match self {
            BufferRef::Scalar(b) => Some(b),
            BufferRef::Color(_) => None,
        }
    }

    /// Versão colorida do buffer, pronta para escrever em arquivo: buffer de
    /// cor sai como está, escalar é normalizado para cinza.
    pub fn to_color(self) -> ColorBuffer {
        match self {
            BufferRef::Color(b) => b.clone(),
            BufferRef::Scalar(b) => b.to_color(None),
        }
    }

    pub fn dimensions(self) -> (u32, u32) {
        match self {
            BufferRef::Color(b) => (b.width(), b.height()),
            BufferRef::Scalar(b) => (b.width(), b.height()),
        }
    }
}

impl<'a> From<&'a ColorBuffer> for BufferRef<'a> {
    fn from(b: &'a ColorBuffer) -> BufferRef<'a> {
        BufferRef::Color(b)
    }
}

impl<'a> From<&'a ScalarBuffer> for BufferRef<'a> {
    fn from(b: &'a ScalarBuffer) -> BufferRef<'a> {
        BufferRef::Scalar(b)
    }
}

/// Números coletados durante o render. Cada renderer preenche o que faz sentido
/// para ele e deixa o resto zerado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderStats {
    /// Testes de interseção/cobertura raio-primitivo ou pixel-triângulo.
    pub primitive_tests: u64,
    /// Triângulos descartados pelo backface culling.
    pub backfaces_culled: u64,
    /// Polígonos recortados contra o frustum.
    pub polygons_clipped: u64,
    /// Raios lançados no total (primários + sombra + secundários).
    pub rays: u64,
    /// Caixas de nó de BVH testadas contra um raio.
    pub bvh_nodes_visited: u64,
    /// Nós no caminho mais longo da TLAS, da raiz até a folha mais funda.
    pub tlas_depth: u32,
    /// O mesmo na mais funda das BLAS. Separado da TLAS de propósito: a soma
    /// dos dois é uma cota do caminho mais longo da árvore inteira, não o
    /// caminho — ver `teoria/bvh.md#as-duas-profundidades`.
    pub blas_depth: u32,
    /// Tempo de construção das BVHs, que acontece dentro do `render()`.
    pub bvh_build_time: Duration,
}

/// O que um render devolve: um campo por `RenderPass`, `None` no que este modo
/// não sabe produzir.
#[derive(Debug, Clone, Default)]
pub struct RenderResult {
    pub color: Option<ColorBuffer>,
    pub depth: Option<ScalarBuffer>,
    pub normal: Option<ColorBuffer>,
    pub uv: Option<ColorBuffer>,
    pub ray_count: Option<ScalarBuffer>,
    pub bvh_heatmap: Option<ScalarBuffer>,
    pub stats: RenderStats,
}

impl RenderResult {
    pub fn new() -> Self {
        RenderResult::default()
    }

    /// Acesso pelo passe, para código que compara os dois modos sem saber de
    /// antemão qual buffer vai olhar.
    pub fn get(&self, pass: RenderPass) -> Option<BufferRef<'_>> {
        match pass {
            RenderPass::Color => self.color.as_ref().map(BufferRef::from),
            RenderPass::Normal => self.normal.as_ref().map(BufferRef::from),
            RenderPass::Uv => self.uv.as_ref().map(BufferRef::from),
            RenderPass::Depth => self.depth.as_ref().map(BufferRef::from),
            RenderPass::RayCount => self.ray_count.as_ref().map(BufferRef::from),
            RenderPass::BvhHeatmap => self.bvh_heatmap.as_ref().map(BufferRef::from),
        }
    }
}

/// A interface comum, que `Rasterizer` e `RayTracer` implementam. Permite
/// escrever código genérico sobre `R: Renderer` pra comparar o resultado de dois
/// modos sobre a mesma cena.
pub trait Renderer {
    fn name(&self) -> &'static str;

    /// Renderiza `scene` com `params`, devolvendo todos os passes que este modo
    /// sabe produzir.
    ///
    /// Função pura: mesma entrada, mesma saída, sem estado guardado entre
    /// chamadas.
    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult;
}
