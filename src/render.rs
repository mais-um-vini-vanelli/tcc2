//! A interface comum aos dois pipelines.

use std::cell::Cell;
use std::time::Duration;

use crate::color::Rgb;
use crate::parameters::{OutputOptions, Parameters, Tonemap};
use crate::scene::Scene;

/// Uma imagem: `width * height` elementos.
#[derive(Debug, Clone, PartialEq)]
pub struct Buffer<T> {
    width: u32,
    height: u32,
    pixels: Vec<T>,
}

pub type ColorBuffer = Buffer<Rgb>;
pub type ScalarBuffer = Buffer<f32>;

impl<T: Copy> Buffer<T> {
    pub fn filled(width: u32, height: u32, value: T) -> Self {
        let size = width as usize * height as usize;
        Buffer {
            width,
            height,
            pixels: vec![value; size],
        }
    }

    pub fn from_pixels(width: u32, height: u32, pixels: Vec<T>) -> Option<Self> {
        if pixels.len() == width as usize * height as usize {
            Some(Buffer {
                width,
                height,
                pixels,
            })
        } else {
            None
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[T] {
        &self.pixels
    }

    pub fn get(&self, x: u32, y: u32) -> Option<T> {
        self.index(x, y).map(|i| self.pixels[i])
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        match self.index(x, y) {
            Some(i) => self.pixels[i] = value,
            None => panic!("indices out of bounds"),
        }
    }

    fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }
}

impl<T: Copy + Default> Buffer<T> {
    pub fn new(width: u32, height: u32) -> Self {
        Buffer::filled(width, height, T::default())
    }
}

impl ScalarBuffer {
    /// Converte para grayscale, normalizando linearmente o intervalo dos
    /// valores finitos para `[0, 1]`. Cuidado ao comparar duas imagens, porque
    /// um intervalo diferente vai gerar intensidades diferentes pro mesmo valor
    /// bruto de pixel.
    pub fn to_color(&self) -> ColorBuffer {
        let mut intervalo: Option<(f32, f32)> = None;

        for &val in &self.pixels {
            if val.is_finite() {
                intervalo = Some(match intervalo {
                    None => (val, val),
                    Some((lo, hi)) => (lo.min(val), hi.max(val)),
                });
            }
        }

        let (lo, hi) = intervalo.unwrap_or((0.0, 1.0));
        let span = hi - lo;

        let pixels = self
            .pixels
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterizationResult {
    /// Profundidade do fragmento mais próximo em cada pixel.
    pub depth: ScalarBuffer,
    /// Normal de sombreamento, remapeada de `[-1, 1]` para `[0, 1]`.
    pub normal: ColorBuffer,
    /// Coordenada de textura interpolada, no canal rg.
    pub uv: ColorBuffer,
    /// Testes de cobertura pixel-triângulo.
    pub primitive_tests: u64,
    /// Triângulos descartados pelo backface culling.
    pub backfaces_culled: u64,
    /// Polígonos recortados contra o frustum.
    pub polygons_clipped: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RayTracingResult {
    /// Raios lançados por pixel, contando sombra e recursão.
    pub ray_count: ScalarBuffer,
    /// Nós de BVH testados por pixel, por todos os raios dele.
    pub bvh_heatmap: ScalarBuffer,
    /// Testes de interseção entre raios e primitivos.
    pub primitive_tests: u64,
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

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Contagem {
    pub primitive_tests: u64,
    pub rays: u64,
    pub bvh_nodes_visited: u64,
    pub tlas_depth: u32,
    pub blas_depth: u32,
    pub bvh_build_time: Duration,
}

/// Contadores do render.
///
/// `Cell` porque o callback de visibilidade que `shade` recebe é `Fn`, não
/// `FnMut`, e mesmo assim precisa contabilizar o raio de sombra que dispara.
/// `Contagem` é `Copy`, então a célula sai barata.
#[derive(Debug, Default)]
pub(crate) struct Contador(Cell<Contagem>);

impl Contador {
    pub(crate) fn raio(&self) {
        self.atualiza(|s| s.rays += 1);
    }

    pub(crate) fn teste(&self) {
        self.atualiza(|s| s.primitive_tests += 1);
    }

    pub(crate) fn no(&self) {
        self.atualiza(|s| s.bvh_nodes_visited += 1);
    }

    pub(crate) fn stats(&self) -> Contagem {
        self.0.get()
    }

    pub(crate) fn atualiza(&self, f: impl FnOnce(&mut Contagem)) {
        let mut stats = self.0.get();
        f(&mut stats);
        self.0.set(stats);
    }
}

#[derive(Debug, Clone)]
pub struct RenderResult {
    pub color: ColorBuffer,
    pub rasterization: RasterizationResult,
    pub ray_tracing: RayTracingResult,
}

impl RenderResult {
    pub fn new(width: u32, height: u32, environment: Rgb) -> Self {
        RenderResult {
            color: ColorBuffer::filled(width, height, environment),
            rasterization: RasterizationResult {
                depth: ScalarBuffer::filled(width, height, f32::INFINITY),
                normal: ColorBuffer::new(width, height),
                uv: ColorBuffer::new(width, height),
                primitive_tests: 0,
                backfaces_culled: 0,
                polygons_clipped: 0,
            },
            ray_tracing: RayTracingResult {
                ray_count: ScalarBuffer::new(width, height),
                bvh_heatmap: ScalarBuffer::new(width, height),
                primitive_tests: 0,
                rays: 0,
                bvh_nodes_visited: 0,
                tlas_depth: 0,
                blas_depth: 0,
                bvh_build_time: Duration::ZERO,
            },
        }
    }

    /// Aplica a exposure e tonemap.
    pub fn apply_output(&mut self, output: &OutputOptions) {
        let exposure = 2.0f32.powf(output.exposure);

        for pixel in &mut self.color.pixels {
            *pixel = {
                let c = *pixel * exposure;
                match output.tonemap {
                    Tonemap::None => c,
                    Tonemap::Reinhard => c.map(|v| v / (1.0 + v)),
                }
            };
        }
    }
}

/// A interface comum, que `Rasterizer` e `RayTracer` implementam, pra poder
/// escrever código genérico sobre `R: Renderer` e comparar o resultado dos dois
/// modos sobre a mesma cena.
pub trait Renderer {
    /// Renderiza `scene` com `params` e retorna a imagem final e as informações
    /// de debug.
    fn render(&self, scene: &Scene, params: &Parameters) -> RenderResult;
}
