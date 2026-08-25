//! Escrita da imagem final em PPM.
//!
//! Os dois formatos são implementados aqui, sem dependências externas. PPM
//! porque é fácil de escrever e de ler num editor de texto.

use std::fs;
use std::io;
use std::path::Path;

use crate::render::ColorBuffer;

pub fn encode_ppm(buffer: &ColorBuffer) -> Vec<u8> {
    let header = format!("P6\n{} {}\n255\n", buffer.width(), buffer.height());
    // Converter os pixels de (f32, f32, f32) pra (u8, u8, u8) e concatenar na imagem final
    let mut out = Vec::with_capacity(header.len() + buffer.pixels().len() * 3);
    out.extend_from_slice(header.as_bytes());
    for pixel in buffer.pixels() {
        out.extend_from_slice(&pixel.to_bytes());
    }
    out
}

/// Grava o buffer como PPM binário (P6).
pub fn write_ppm(path: impl AsRef<Path>, buffer: &ColorBuffer) -> io::Result<()> {
    fs::write(path, encode_ppm(buffer))
}
