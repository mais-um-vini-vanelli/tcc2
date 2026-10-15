//! Software renderer didático: uma API única que alterna entre **rasterização**
//! e **ray tracing** sobre a mesma descrição da cena.
//!
//! `render()` é função pura, sem estado escondido. A mesma cena vale pros dois modos.

mod bvh;
mod overlay;

#[cfg(feature = "glb")]
pub mod glb;

pub mod color;
pub mod math;
pub mod parameters;
pub mod rasterizer;
pub mod raytracer;
pub mod render;
pub mod scene;
pub mod shading;
pub mod texture;
pub mod transform;

pub mod prelude {
    pub use crate::color::Rgb;
    pub use crate::math::{Aabb, Baricentrics, Plane, Ray, Tangent, Vec2, Vec3, Vertex};
    pub use crate::parameters::{
        AccelerationOptions, BvhSplit, Filtering, OutputOptions, Parameters, RasterizationOptions,
        RayTracingOptions, ReflectanceModel, ShadingFrequency, ShadingOptions, Tonemap,
        TransformMethod, TransformOptions,
    };
    pub use crate::rasterizer::Rasterizer;
    pub use crate::raytracer::RayTracer;
    pub use crate::render::{
        ColorBuffer, RasterizationResult, RayTracingResult, RenderResult, Renderer, ScalarBuffer,
    };
    pub use crate::scene::{
        Camera, CameraInstance, Content, Geometry, GeometryInstance, Light, LightInstance,
        Material, MaterialSample, Mesh, Object, Scene, Value, eixos_da_uv,
    };
    pub use crate::shading::{Surface, shade};
    pub use crate::texture::{Footprint, Texture, TextureError};
    pub use crate::transform::{Chain, Transform};
}
