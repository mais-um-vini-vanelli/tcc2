use std::error::Error;
use std::fs::{self, File};
use std::io::BufWriter;

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use tcc2::prelude::*;

pub fn write_png(
    path: &str,
    buffer: &ColorBuffer,
    pixel: impl Fn(Rgb) -> [u8; 3],
) -> Result<(), Box<dyn Error>> {
    let bytes: Vec<u8> = buffer.pixels().iter().flat_map(|p| pixel(*p)).collect();

    fs::create_dir_all("./examples/renders")?;

    PngEncoder::new(BufWriter::new(File::create(path)?)).write_image(
        &bytes,
        buffer.width(),
        buffer.height(),
        ExtendedColorType::Rgb8,
    )?;

    Ok(())
}
