//! `*.tcc2.glb`: a `Scene` gravada como glTF 2.0 binário.
//!
//! O arquivo é um `.glb` válido — qualquer visualizador abre e mostra a cena.
//! Os campos específicos da lib vão em extensions `TCC2_extras`:
//! `Light::Ambient`, `Object.casts_shadows` e qual geometria analítica uma malha
//! tesselada representa.
//!
//! O resto vai em campo nativo:
//!
//! | Tipo da lib | Onde vai |
//! |---|---|
//! | [`Camera`] | `cameras[i]`, só a projeção: onde ela está é o transform do nó, como no glTF (olha pro −Z local, +Y pra cima) |
//! | [`Light::Point`], [`Directional`](Light::Directional), [`Spot`](Light::Spot) | nó + `KHR_lights_punctual`, com a intensidade **crua** (sem converter para candela/lux); o eixo é o −Z do nó |
//! | [`Transform`] | `translation`/`rotation`/`scale` do nó, com o Euler XYZ virando quatérnio |
//! | [`Geometry::Mesh`], [`Triangle`](Geometry::Triangle) | `POSITION` e, só quando existem, `NORMAL`/`TANGENT`/`TEXCOORD_0`, mais `indices` |
//! | [`Material`] | `pbrMetallicRoughness`, `normalTexture`, `emissiveFactor` e as extensões `KHR_materials_transmission`, `_ior` e `_emissive_strength` |
//!
//! # Uso
//!
//! ```no_run
//! # use tcc2::prelude::*;
//! # use tcc2::glb;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let cena = Scene {
//! #     objects: vec![Object::camera(
//! #         Camera::Orthographic { width: 1.0, height: 1.0, near: 0.1, far: 10.0 },
//! #         Transform::looking_at(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0)),
//! #     )],
//! # };
//! std::fs::write("cena.tcc2.glb", glb::encode(&cena, None)?)?;
//!
//! let (cena, parametros) = glb::decode(&std::fs::read("cena.tcc2.glb")?)?;
//! # let _ = (cena, parametros);
//! # Ok(())
//! # }
//! ```

mod decode;
mod encode;
mod parameters;

pub use decode::decode;
pub use encode::encode;

pub const EXTENSION: &str = "TCC2_extras";
pub const VERSION: u64 = 1;

pub type Error = Box<dyn std::error::Error>;

fn non_null_field<'a>(object: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    object.get(field).filter(|val| !val.is_null())
}
