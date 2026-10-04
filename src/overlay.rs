use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::render::ColorBuffer;
use crate::scene::Camera;

/// Desenha as arestas da caixa a partir dos oito cantos em espaço de mundo.
pub fn caixa(destino: &mut ColorBuffer, camera: &Camera, cantos: [Vec3; 8], cor: Rgb) {
    const ARESTAS: [(usize, usize); 12] = [
        // ao longo de x
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        // ao longo de y
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        // ao longo de z
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for (a, b) in ARESTAS {
        linha(destino, camera, cantos[a], cantos[b], cor);
    }
}

/// Desenha o segmento de `a` a `b`, os dois em espaço de mundo, sem teste de profundidade.
pub fn linha(destino: &mut ColorBuffer, camera: &Camera, a: Vec3, b: Vec3, cor: Rgb) {
    let (largura, altura) = (destino.width(), destino.height());

    let (a, b) = (camera.to_camera_space(a), camera.to_camera_space(b));

    let Some((a, b)) = recorta_no_near(camera, a, b) else {
        return;
    };

    let (Some(a), Some(b)) = (
        camera.to_screen_space(a, largura, altura),
        camera.to_screen_space(b, largura, altura),
    ) else {
        return;
    };

    let Some((a, b)) = recorta_na_tela(a.xy(), b.xy(), largura, altura) else {
        return;
    };

    let d = b - a;
    let passos = d.x.abs().max(d.y.abs()).ceil().max(1.0);

    for i in 0..=(passos as u32) {
        let p = a + d * (i as f32 / passos);

        destino.set(p.x as u32, p.y as u32, cor);
    }
}

/// O trecho do segmento com `z >= near`, em espaço de câmera, ou `None` se ele
/// está todo antes do plano.
fn recorta_no_near(camera: &Camera, a: Vec3, b: Vec3) -> Option<(Vec3, Vec3)> {
    let (near, _far) = camera.clip_planes();

    match (a.z >= near, b.z >= near) {
        (true, true) => Some((a, b)),
        (false, false) => None,

        (a_dentro, _) => {
            let t = (near - a.z) / (b.z - a.z);
            let cruzamento = a + (b - a) * t;

            Some(if a_dentro {
                (a, cruzamento)
            } else {
                (cruzamento, b)
            })
        }
    }
}

/// O trecho do segmento que cai dentro da imagem, em pixels. É a mesma ideia de `Aabb::hit`, aplicada em slabs 2D.
fn recorta_na_tela(a: Vec2, b: Vec2, largura: u32, altura: u32) -> Option<(Vec2, Vec2)> {
    let d = b - a;

    let (mut t_entrada, mut t_saida) = (0.0f32, 1.0f32);

    for (origem, direcao, limite) in [(a.x, d.x, largura as f32), (a.y, d.y, altura as f32)] {
        if direcao == 0.0 {
            if origem < 0.0 || origem > limite {
                // Paralelo a esta slab, a origem define se está dentro ou fora.
                return None;
            }
        } else {
            // origem + t·direcao = 0      → t = -origem / direcao
            // origem + t·direcao = limite → t = (limite - origem) / direcao
            let (t0, t1) = (-origem / direcao, (limite - origem) / direcao);

            t_entrada = t_entrada.max(t0.min(t1));
            t_saida = t_saida.min(t0.max(t1));
        }
    }

    (t_entrada <= t_saida).then(|| (a + d * t_entrada, a + d * t_saida))
}
