use serde_json::{Value, json};

use crate::parameters::*;

use super::non_null_field;

pub(super) fn encode(parameters: &Parameters) -> Value {
    json!({
        "transform": {
            "method": match parameters.transform.method {
                TransformMethod::Stepwise => "stepwise",
            },
        },
        "rasterization": {
            "backface_cull": parameters.rasterization.backface_cull,
            "perspective_correct": parameters.rasterization.perspective_correct,
            "depth_test": parameters.rasterization.depth_test,
        },
        "shading": {
            "reflectance_model": match parameters.shading.reflectance_model {
                ReflectanceModel::BlinnPhong => "blinn_phong",
            },
            "texture_filter": match parameters.shading.texture_filter {
                TextureFilter::Nearest => "nearest",
                TextureFilter::Bilinear => "bilinear",
                TextureFilter::MipmapBilinear => "mipmap_bilinear",
                TextureFilter::Trilinear => "trilinear",
            },
        },
        "ray_tracing": {
            "max_depth": parameters.ray_tracing.max_depth,
            "shadows": parameters.ray_tracing.shadows,
            "reflections": parameters.ray_tracing.reflections,
            "refractions": parameters.ray_tracing.refractions,
            "acceleration": {
                "enabled": parameters.ray_tracing.acceleration.enabled,
                "bvh_split": match parameters.ray_tracing.acceleration.bvh_split {
                    BvhSplit::Median => "median",
                    BvhSplit::Sah => "sah",
                },
                "wireframe": parameters.ray_tracing.acceleration.wireframe,
            },
        },
        "output": {
            "width": parameters.output.width,
            "height": parameters.output.height,
            "exposure": parameters.output.exposure,
            "tonemap": match parameters.output.tonemap {
                Tonemap::None => "none",
                Tonemap::Reinhard => "reinhard",
            },
        },
    })
}

pub(super) fn decode(json: &Value) -> Parameters {
    let transform = non_null_field(json, "transform").unwrap_or(&Value::Null);
    let rasterization = non_null_field(json, "rasterization").unwrap_or(&Value::Null);
    let shading = non_null_field(json, "shading").unwrap_or(&Value::Null);
    let ray_tracing = non_null_field(json, "ray_tracing").unwrap_or(&Value::Null);
    let aceleracao = non_null_field(ray_tracing, "acceleration").unwrap_or(&Value::Null);
    let output = non_null_field(json, "output").unwrap_or(&Value::Null);

    let rasterization_options = RasterizationOptions::default();
    let ray_tracing_options = RayTracingOptions::default();
    let acceleration_options = AccelerationOptions::default();
    let output_options = OutputOptions::default();

    fn to_str<'a>(objeto: &'a Value, nome: &str) -> Option<&'a str> {
        non_null_field(objeto, nome).and_then(Value::as_str)
    }

    fn to_bool(objeto: &Value, nome: &str, padrao: bool) -> bool {
        non_null_field(objeto, nome)
            .and_then(Value::as_bool)
            .unwrap_or(padrao)
    }

    fn to_u32(objeto: &Value, nome: &str, padrao: u32) -> u32 {
        non_null_field(objeto, nome)
            .and_then(Value::as_u64)
            .map_or(padrao, |n| n as u32)
    }

    Parameters {
        transform: TransformOptions {
            method: match to_str(transform, "method") {
                Some("stepwise") => TransformMethod::Stepwise,
                _ => TransformMethod::default(),
            },
        },
        rasterization: RasterizationOptions {
            backface_cull: to_bool(
                rasterization,
                "backface_cull",
                rasterization_options.backface_cull,
            ),
            perspective_correct: to_bool(
                rasterization,
                "perspective_correct",
                rasterization_options.perspective_correct,
            ),
            depth_test: to_bool(
                rasterization,
                "depth_test",
                rasterization_options.depth_test,
            ),
            on_fragment: rasterization_options.on_fragment,
            on_triangle: rasterization_options.on_triangle,
        },
        shading: ShadingOptions {
            reflectance_model: match to_str(shading, "reflectance_model") {
                Some("blinn_phong") => ReflectanceModel::BlinnPhong,
                _ => ReflectanceModel::default(),
            },
            texture_filter: match to_str(shading, "texture_filter") {
                Some("nearest") => TextureFilter::Nearest,
                Some("bilinear") => TextureFilter::Bilinear,
                Some("mipmap_bilinear") => TextureFilter::MipmapBilinear,
                Some("trilinear") => TextureFilter::Trilinear,
                _ => TextureFilter::default(),
            },
        },
        ray_tracing: RayTracingOptions {
            max_depth: to_u32(ray_tracing, "max_depth", ray_tracing_options.max_depth),
            shadows: to_bool(ray_tracing, "shadows", ray_tracing_options.shadows),
            reflections: to_bool(ray_tracing, "reflections", ray_tracing_options.reflections),
            refractions: to_bool(ray_tracing, "refractions", ray_tracing_options.refractions),
            acceleration: AccelerationOptions {
                enabled: to_bool(aceleracao, "enabled", acceleration_options.enabled),
                bvh_split: match to_str(aceleracao, "bvh_split") {
                    Some("median") => BvhSplit::Median,
                    Some("sah") => BvhSplit::Sah,
                    _ => acceleration_options.bvh_split,
                },
                wireframe: non_null_field(aceleracao, "wireframe")
                    .and_then(Value::as_u64)
                    .map(|n| n as u32),
            },
            on_ray: ray_tracing_options.on_ray,
        },
        output: OutputOptions {
            width: to_u32(output, "width", output_options.width),
            height: to_u32(output, "height", output_options.height),
            exposure: non_null_field(output, "exposure")
                .and_then(|e| e.as_f64().map(|e| e as f32))
                .unwrap_or(output_options.exposure),
            tonemap: match to_str(output, "tonemap") {
                Some("none") => Tonemap::None,
                Some("reinhard") => Tonemap::Reinhard,
                _ => output_options.tonemap,
            },
        },
    }
}
