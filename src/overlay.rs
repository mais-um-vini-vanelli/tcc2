use crate::color::Rgb;
use crate::math::{Vec2, Vec3};
use crate::render::ColorBuffer;
use crate::scene::CameraInstance;

pub fn draw_box(
    camera_instance: &CameraInstance,
    corners: [Vec3; 8],
    color: Rgb,
    buffer: &mut ColorBuffer,
) {
    const EDGES: [(usize, usize); 12] = [
        // Ao longo de x
        (0, 1),
        (2, 3),
        (4, 5),
        (6, 7),
        // Ao longo de y
        (0, 2),
        (1, 3),
        (4, 6),
        (5, 7),
        // Ao longo de z
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    for (a, b) in EDGES {
        draw_line(camera_instance, corners[a], corners[b], color, buffer);
    }
}

pub fn draw_line(
    camera_instance: &CameraInstance,
    a: Vec3,
    b: Vec3,
    color: Rgb,
    buffer: &mut ColorBuffer,
) {
    let (width, height) = (buffer.width(), buffer.height());

    let (a, b) = (
        camera_instance.to_camera_space(a),
        camera_instance.to_camera_space(b),
    );

    let Some((a, b)) = clip_near(camera_instance, a, b) else {
        return;
    };

    let (Some(a), Some(b)) = (
        camera_instance.local.to_screen_space(a, width, height),
        camera_instance.local.to_screen_space(b, width, height),
    ) else {
        return;
    };

    let Some((a, b)) = clip_screen(a.xy(), b.xy(), width, height) else {
        return;
    };

    let d = b - a;
    let steps = d.x.abs().max(d.y.abs()).ceil().max(1.0);

    for i in 0..=(steps as u32) {
        let p = a + d * (i as f32 / steps);

        buffer.set(p.x as u32, p.y as u32, color);
    }
}

/// Encontra o trecho do segmento com `z >= near` em espaço de câmera.
fn clip_near(camera_instance: &CameraInstance, a: Vec3, b: Vec3) -> Option<(Vec3, Vec3)> {
    let (near, _far) = camera_instance.local.clip_planes();

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

/// Encontra o trecho do segmento que cai dentro da imagem, em pixels. É a mesma ideia de `Aabb::hit` aplicada em slabs 2D.
fn clip_screen(a: Vec2, b: Vec2, width: u32, height: u32) -> Option<(Vec2, Vec2)> {
    let d = b - a;

    let (mut t_entry, mut t_exit) = (0.0f32, 1.0f32);

    for (origin, direction, limit) in [
        (a.x, d.x, width as f32 - 1.0),
        (a.y, d.y, height as f32 - 1.0),
    ] {
        if direction == 0.0 {
            if origin < 0.0 || origin > limit {
                // Raio é paralelo a esta slab; a origem define se está dentro ou fora.
                return None;
            }
        } else {
            // origem + t·direcao = 0      → t = -origem / direcao
            // origem + t·direcao = limite → t = (limite - origem) / direcao
            let (t0, t1) = (-origin / direction, (limit - origin) / direction);

            t_entry = t_entry.max(t0.min(t1));
            t_exit = t_exit.min(t0.max(t1));
        }
    }

    (t_entry <= t_exit).then(|| (a + d * t_entry, a + d * t_exit))
}
