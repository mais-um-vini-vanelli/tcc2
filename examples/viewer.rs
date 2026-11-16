mod common;

use std::error::Error;
use std::fs;
use std::path::Path;

use minifb::{Key, Window, WindowOptions};
use tcc2::io::Reader;
use tcc2::prelude::*;

const DEFAULT_SCENE: &str = "examples/assets/Scene.tcc2";
const USAGE: &str = "usage: viewer [scene.tcc2] [--renderer rasterizer|raytracer]";

fn main() -> Result<(), Box<dyn Error>> {
    let mut path = None;
    let mut name = String::from("rasterizer");

    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--renderer" | "-r" => name = args.next().ok_or(USAGE)?,
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ => path = Some(arg),
        }
    }

    let path = path.unwrap_or(DEFAULT_SCENE.into());
    let bytes = fs::read(&path).map_err(|err| format!("{path}: {err}"))?;
    let (scene, params) = Reader::read(&bytes)?;

    let (result, extra) = match name.as_str() {
        "rasterizer" => {
            let result = Rasterizer.render(&scene, &params);
            let stats = &result.rasterization;

            println!("primitive_tests: {}", stats.primitive_tests);
            println!("backfaces_culled: {}", stats.backfaces_culled);
            println!("polygons_clipped: {}", stats.polygons_clipped);

            let extra = vec![
                ("depth", stats.depth.to_color()),
                ("normal", stats.normal.clone()),
                ("uv", stats.uv.clone()),
            ];

            (result, extra)
        }
        "raytracer" => {
            let result = RayTracer.render(&scene, &params);
            let stats = &result.ray_tracing;

            println!("rays: {}", stats.rays);
            println!("primitive_tests: {}", stats.primitive_tests);
            println!("bvh_nodes_visited: {}", stats.bvh_nodes_visited);
            println!("tlas_depth: {}", stats.tlas_depth);
            println!("blas_depth: {}", stats.blas_depth);
            println!("bvh_build_time: {:?}", stats.bvh_build_time);

            let extra = vec![
                ("ray_count", stats.ray_count.to_color()),
                ("bvh_heatmap", stats.bvh_heatmap.to_color()),
            ];

            (result, extra)
        }
        other => return Err(format!("unknown renderer: {other}; {USAGE}").into()),
    };

    let stem = Path::new(&path)
        .file_stem()
        .unwrap_or(path.as_ref())
        .to_string_lossy()
        .into_owned();

    let mut buffers = vec![("color", result.color.clone())];
    buffers.extend(extra);

    for (pass, buffer) in &buffers {
        let file = format!("./examples/renders/{stem}_{name}_{pass}.png");
        common::write_png(&file, buffer, Rgb::to_srgb)?;
        println!("{file}");
    }

    let pixels: Vec<u32> = result
        .color
        .pixels()
        .iter()
        .map(|color| {
            let [r, g, b] = color.to_srgb();
            u32::from_be_bytes([0, r, g, b])
        })
        .collect();

    let mut window = Window::new(
        &format!("{stem} - {name}"),
        params.output.width as usize,
        params.output.height as usize,
        WindowOptions::default(),
    )?;

    window.set_target_fps(30);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(
            &pixels,
            params.output.width as usize,
            params.output.height as usize,
        )?;
    }

    Ok(())
}
