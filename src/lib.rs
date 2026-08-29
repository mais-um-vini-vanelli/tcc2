//! Software renderer didático: uma API única que alterna entre **rasterização**
//! e **ray tracing** sobre a mesma descrição da cena.
//!
//! `render()` é função pura, sem estado escondido. A mesma cena vale pros dois modos.

pub mod color;
pub mod math;
pub mod output;
pub mod parameters;
pub mod rasterizer;
pub mod raytracer;
pub mod render;
pub mod scene;

pub mod prelude {
    pub use crate::color::Rgb;
    pub use crate::math::{Ray, Vec2, Vec3};
    pub use crate::parameters::{
        AccelOptions, ClippingOptions, Filtering, OutputOptions, Parameters, RasterOptions,
        RayTracingOptions, TextureOptions,
    };
    pub use crate::rasterizer::Rasterizer;
    pub use crate::raytracer::RayTracer;
    pub use crate::render::{
        Buffer, BufferKind, ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer,
        ScalarBuffer,
    };
    pub use crate::scene::{Camera, Light, Material, Primitive, Scene, Texture};
}
