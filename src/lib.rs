//! Software renderer didático: uma API única que alterna entre **rasterização**
//! e **ray tracing** sobre a mesma descrição da cena.
//!
//! `render()` é função pura, sem estado escondido. A mesma cena vale pros dois modos.

pub mod color;
pub mod math;
pub mod parameters;
pub mod rasterizer;
pub mod raytracer;
pub mod render;
pub mod scene;
pub mod shading;
pub mod transform;

pub mod prelude {
    pub use crate::color::Rgb;
    pub use crate::math::{Baricentrics, Plane, Ray, Vec2, Vec3, Vertex};
    pub use crate::parameters::{
        AccelerationOptions, ClippingOptions, Filtering, OutputOptions, Parameters,
        RasterizationOptions, RayTracingOptions, ReflectanceModel, ShadingFrequency,
        ShadingOptions, TextureOptions,
    };
    pub use crate::rasterizer::Rasterizer;
    pub use crate::raytracer::RayTracer;
    pub use crate::render::{
        BufferKind, BufferRef, ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer,
        ScalarBuffer,
    };
    pub use crate::scene::{
        Camera, Light, Material, MaterialSample, Mesh, Object, Primitive, Scene, Texture, Value,
    };
    pub use crate::shading::{Surface, shade};
    pub use crate::transform::Transform;
}
