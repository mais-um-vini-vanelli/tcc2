mod decode;
mod encode;
mod parameters;

pub use decode::decode;
pub use encode::encode;

pub const EXTENSION: &str = "TCC2_extras";
pub const VERSION: u64 = 1;

pub const LUMENS_PER_WATT: f32 = 683.0;

pub type Error = Box<dyn std::error::Error>;

fn non_null_field<'a>(object: &'a serde_json::Value, field: &str) -> Option<&'a serde_json::Value> {
    object.get(field).filter(|val| !val.is_null())
}
