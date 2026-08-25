//! Software renderer didático: uma API única que alterna entre **rasterização**
//! e **ray tracing** sobre a mesma descrição de cena.
//!
//! `render()` é função pura, sem estado escondido. A mesma cena vale pros dois modos.
//!
//! # Exemplo
//!
//! Montar uma cena, escolher parâmetros e escrever um buffer em disco. O
//! `render()` ainda não existe (M0.5), então aqui o buffer é preenchido à mão:
//!
//! ```no_run
//! use tcc2::prelude::*;
//!
//! let scene = Scene::new(Camera::Perspective {
//!     fov: 60.0,
//!     aspect: 4.0 / 3.0,
//!     near: 0.1,
//!     far: 100.0,
//!     position: Vec3::new(0.0, 0.0, 5.0),
//!     look_at: Vec3::ZERO,
//!     up: Vec3::new(0.0, 1.0, 0.0),
//! })
//! .with_light(Light::Point {
//!     position: Vec3::new(2.0, 3.0, 4.0),
//!     color: Rgb::WHITE,
//!     intensity: 1.0,
//! })
//! .with_object(Primitive::sphere(Vec3::ZERO, 1.0, Material::diffuse(Rgb::new(0.8, 0.2, 0.2))));
//!
//! let params = Parameters::with_resolution(320, 240);
//! assert_eq!(scene.objects.len(), 1);
//!
//! let imagem = ColorBuffer::filled(
//!     params.output.width,
//!     params.output.height,
//!     params.output.background,
//! );
//! tcc2::output::write_png("saida.png", &imagem).unwrap();
//! ```

pub mod color;
pub mod math;
pub mod output;
pub mod parameters;
pub mod render;
pub mod scene;

/// Importação em bloco do que se usa para montar e renderizar uma cena.
pub mod prelude {
    pub use crate::color::Rgb;
    pub use crate::math::{Vec2, Vec3};
    pub use crate::parameters::{
        AccelOptions, ClippingOptions, Filtering, OutputOptions, Parameters, RasterOptions,
        RayTracingOptions, TextureOptions,
    };
    pub use crate::render::{
        Buffer, BufferKind, ColorBuffer, RenderPass, RenderResult, RenderStats, Renderer,
        ScalarBuffer,
    };
    pub use crate::scene::{Camera, Light, Material, Primitive, Scene, Texture};
}
