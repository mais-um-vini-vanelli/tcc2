//! `Parameters` ↔ JSON, para o preset opcional em `TCC2_extras.parameters`.
//!
//! Nada aqui tem a ver com glTF: os parâmetros não descrevem a cena, descrevem
//! como renderizá-la, e o glTF não tem onde guardar isso. Por isso vão num
//! objeto solto.

use serde_json::{Value, json};

use crate::color::Rgb;
use crate::parameters::*;

use super::non_null_field;

pub(super) fn encode(p: &Parameters) -> Value {
    json!({
        "transform": {
            "method": match p.transform.method {
                TransformMethod::Stepwise => "stepwise",
            },
        },
        "rasterization": {
            "msaa": p.rasterization.msaa,
            "backface_cull": p.rasterization.backface_cull,
            "wireframe": p.rasterization.wireframe,
            "perspective_correct": p.rasterization.perspective_correct,
            "depth_test": p.rasterization.depth_test,
        },
        "shading": {
            "frequency": match p.shading.frequency {
                ShadingFrequency::Flat => "flat",
                ShadingFrequency::Gouraud => "gouraud",
                ShadingFrequency::Phong => "phong",
            },
            "reflectance": match p.shading.reflectance {
                ReflectanceModel::BlinnPhong => "blinn_phong",
            },
            "filtering": match p.shading.filtering {
                Filtering::Nearest => "nearest",
                Filtering::Bilinear => "bilinear",
                Filtering::MipmapBilinear => "mipmap_bilinear",
                Filtering::Trilinear => "trilinear",
            },
        },
        "ray_tracing": {
            "max_depth": p.ray_tracing.max_depth,
            "shadows": p.ray_tracing.shadows,
            "reflections": p.ray_tracing.reflections,
            "refractions": p.ray_tracing.refractions,
            "acceleration": {
                "bvh": p.ray_tracing.acceleration.bvh,
                "split": match p.ray_tracing.acceleration.split {
                    BvhSplit::Median => "median",
                    BvhSplit::Sah => "sah",
                },
                "wireframe": p.ray_tracing.acceleration.wireframe,
            },
        },
        "output": {
            "width": p.output.width,
            "height": p.output.height,
            "background": [
                p.output.background.r,
                p.output.background.g,
                p.output.background.b,
            ],
            "exposure": p.output.exposure,
            "tonemap": match p.output.tonemap {
                Tonemap::None => "none",
                Tonemap::Reinhard => "reinhard",
            },
        },
    })
}

pub(super) fn decode(v: &Value) -> Parameters {
    static VAZIO: Value = Value::Null;

    let transform = non_null_field(v, "transform").unwrap_or(&VAZIO);
    let rasterization = non_null_field(v, "rasterization").unwrap_or(&VAZIO);
    let shading = non_null_field(v, "shading").unwrap_or(&VAZIO);
    let ray_tracing = non_null_field(v, "ray_tracing").unwrap_or(&VAZIO);
    let aceleracao = non_null_field(ray_tracing, "acceleration").unwrap_or(&VAZIO);
    let output = non_null_field(v, "output").unwrap_or(&VAZIO);

    let raster = RasterizationOptions::default();
    let tracado = RayTracingOptions::default();
    let ac = AccelerationOptions::default();
    let saida = OutputOptions::default();

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

    fn to_rgb(v: &Value) -> Option<Rgb> {
        Some(Rgb::new(
            v.get(0)?.as_f64()? as f32,
            v.get(1)?.as_f64()? as f32,
            v.get(2)?.as_f64()? as f32,
        ))
    }

    Parameters {
        transform: TransformOptions {
            method: match to_str(transform, "method") {
                Some("stepwise") => TransformMethod::Stepwise,
                _ => TransformMethod::default(),
            },
        },
        rasterization: RasterizationOptions {
            msaa: non_null_field(rasterization, "msaa")
                .and_then(Value::as_u64)
                .map(|n| n as u8),
            backface_cull: to_bool(rasterization, "backface_cull", raster.backface_cull),
            wireframe: to_bool(rasterization, "wireframe", raster.wireframe),
            perspective_correct: to_bool(
                rasterization,
                "perspective_correct",
                raster.perspective_correct,
            ),
            depth_test: to_bool(rasterization, "depth_test", raster.depth_test),
            on_fragment: raster.on_fragment,
            on_triangle: raster.on_triangle,
        },
        shading: ShadingOptions {
            frequency: match to_str(shading, "frequency") {
                Some("flat") => ShadingFrequency::Flat,
                Some("gouraud") => ShadingFrequency::Gouraud,
                Some("phong") => ShadingFrequency::Phong,
                _ => ShadingFrequency::default(),
            },
            reflectance: match to_str(shading, "reflectance") {
                Some("blinn_phong") => ReflectanceModel::BlinnPhong,
                _ => ReflectanceModel::default(),
            },
            filtering: match to_str(shading, "filtering") {
                Some("nearest") => Filtering::Nearest,
                Some("bilinear") => Filtering::Bilinear,
                Some("mipmap_bilinear") => Filtering::MipmapBilinear,
                Some("trilinear") => Filtering::Trilinear,
                _ => Filtering::default(),
            },
        },
        ray_tracing: RayTracingOptions {
            max_depth: to_u32(ray_tracing, "max_depth", tracado.max_depth),
            shadows: to_bool(ray_tracing, "shadows", tracado.shadows),
            reflections: to_bool(ray_tracing, "reflections", tracado.reflections),
            refractions: to_bool(ray_tracing, "refractions", tracado.refractions),
            acceleration: AccelerationOptions {
                bvh: to_bool(aceleracao, "bvh", ac.bvh),
                split: match to_str(aceleracao, "split") {
                    Some("median") => BvhSplit::Median,
                    Some("sah") => BvhSplit::Sah,
                    _ => ac.split,
                },
                wireframe: non_null_field(aceleracao, "wireframe")
                    .and_then(Value::as_u64)
                    .map(|n| n as u32),
            },
            on_ray: tracado.on_ray,
        },
        output: OutputOptions {
            width: to_u32(output, "width", saida.width),
            height: to_u32(output, "height", saida.height),
            background: non_null_field(output, "background")
                .and_then(to_rgb)
                .unwrap_or(saida.background),
            exposure: non_null_field(output, "exposure")
                .and_then(|e| e.as_f64().map(|e| e as f32))
                .unwrap_or(saida.exposure),
            tonemap: match to_str(output, "tonemap") {
                Some("none") => Tonemap::None,
                Some("reinhard") => Tonemap::Reinhard,
                _ => saida.tonemap,
            },
        },
    }
}
