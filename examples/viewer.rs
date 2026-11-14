use std::error::Error;
use std::fs;
use std::path::Path;

use minifb::{Key, Window, WindowOptions};
use tcc2::io::Reader;
use tcc2::prelude::*;

const CENA: &str = "examples/assets/Scene.tcc2";

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args().nth(1).unwrap_or(CENA.into());
    let bytes = fs::read(&path).map_err(|erro| format!("{path}: {erro}"))?;
    let (scene, params) = Reader::read(&bytes)?;

    let result = Rasterizer.render(&scene, &params);

    let pixels: Vec<u32> = result
        .color
        .pixels()
        .iter()
        .map(|cor| {
            let [r, g, b] = cor.to_srgb();
            u32::from_be_bytes([0, r, g, b])
        })
        .collect();

    let title = Path::new(&path)
        .file_name()
        .unwrap_or(path.as_ref())
        .to_string_lossy()
        .into_owned();

    let mut window = Window::new(
        &title,
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
