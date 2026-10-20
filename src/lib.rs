mod bvh;
mod overlay;

#[cfg(feature = "glb")]
pub mod glb;

pub mod color;
pub mod geometry;
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
    pub use crate::geometry::{Geometry, Mesh};
    pub use crate::math::{Aabb, Baricentrics, Plane, Ray, Tangent, Vec2, Vec3, Vertex};
    pub use crate::parameters::{
        AccelerationOptions, BvhSplit, OutputOptions, Parameters, RasterizationOptions,
        RayTracingOptions, ReflectanceModel, ShadingOptions, TextureFilter, Tonemap,
        TransformMethod, TransformOptions,
    };
    pub use crate::rasterizer::Rasterizer;
    pub use crate::raytracer::RayTracer;
    pub use crate::render::{
        ColorBuffer, RasterizationResult, RayTracingResult, RenderResult, Renderer, ScalarBuffer,
    };
    pub use crate::scene::{
        Camera, CameraInstance, Channel, GeometryInstance, Light, LightInstance, Material,
        MaterialSample, Object, ObjectKind, Primitive, Scene,
    };
    pub use crate::shading::{Surface, shade};
    pub use crate::texture::{Footprint, Mipmaps, Texture, TextureError, TextureWrap};
    pub use crate::transform::{Transform, TransformChain};
}
