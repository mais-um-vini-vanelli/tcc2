mod common;

use std::error::Error;
use std::sync::Arc;

use tcc2::prelude::*;

const BRICK: &str = "examples/assets/textures/BrickWall/BrickWall";
const BARK: &str = "examples/assets/textures/TreeWood/TreeWood";
const WET: f32 = 0.45;

fn main() -> Result<(), Box<dyn Error>> {
    let brick = Material {
        albedo: Channel::Map(Arc::new(Texture::open(
            format!("{BRICK}_Albedo.jpg"),
            Encoding::Srgb,
        )?)),
        roughness: Channel::Map(Arc::new(
            Texture::open(format!("{BRICK}_Roughness.jpg"), Encoding::Linear)?
                .map_texels(|c| c * WET),
        )),
        normal: Some(Arc::new(Texture::open(
            format!("{BRICK}_Normal.png"),
            Encoding::Linear,
        )?)),
        ..Material::default()
    };

    let bark = Material {
        albedo: Channel::Map(Arc::new(Texture::open(
            format!("{BARK}_Albedo.jpg"),
            Encoding::Srgb,
        )?)),
        roughness: Channel::Map(Arc::new(
            Texture::open(format!("{BARK}_Roughness.png"), Encoding::Linear)?
                .map_texels(|c| c * WET),
        )),
        normal: Some(Arc::new(Texture::open(
            format!("{BARK}_Normal.jpg"),
            Encoding::Linear,
        )?)),
        ..Material::default()
    };

    let mut params = Parameters::with_resolution(900, 675);
    params.output.tonemap = Tonemap::Reinhard;

    for (name, material) in [("tijolo", brick), ("casca", bark)] {
        for (suffix, map) in [
            ("com_normal", material.normal.clone()),
            ("sem_normal", None),
        ] {
            let material = Material {
                normal: map,
                ..material.clone()
            };

            let image = RayTracer.render(&scene(material), &params).color;

            let file = format!("./examples/renders/texturas_{name}_{suffix}.png");
            common::write_png(&file, &image, Rgb::to_srgb)?;

            println!("wrote {file}");
        }
    }

    Ok(())
}

fn scene(material: Material) -> Scene {
    let corners = [
        Vec3::new(-6.0, 0.0, -2.2),
        Vec3::new(20.0, 0.0, -2.2),
        Vec3::new(20.0, 4.2, -2.2),
        Vec3::new(-6.0, 4.2, -2.2),
    ];

    // A uv acompanha o tamanho em mundo: a textura se repete a cada 2,2.
    let u = (corners[1] - corners[0]).length() / 2.2;
    let v = (corners[3] - corners[0]).length() / 2.2;

    let normal = (corners[1] - corners[0])
        .cross(corners[3] - corners[0])
        .normalized();

    let point_light = Vec3::new(3.5, 1.7, 0.4);

    // A intensidade é compensada pela distância até o centro da parede, porque a
    // pontual atenua com d².
    let d = (point_light - Vec3::new(0.0, 2.1, -2.2)).length();

    Scene {
        objects: vec![
            Object::camera(
                Camera::Perspective {
                    fov: 48.0,
                    near: 0.1,
                    far: 100.0,
                },
                // Perto da ponta esquerda da parede, olhando ao longo dela.
                Transform::looking_at(Vec3::new(-2.6, 1.8, -1.2), Vec3::new(0.5, 1.5, -3.5)),
            ),
            Object::light(
                Light::Point {
                    color: Rgb::new(1.0, 0.95, 0.88),
                    intensity: 3.0 * d * d,
                },
                Transform {
                    translation: point_light,
                    ..Transform::IDENTITY
                },
            ),
            // Direcional e fraca, só pra ponta direita não fechar em preto.
            Object::light(
                Light::Directional {
                    color: Rgb::new(0.78, 0.82, 0.95),
                    intensity: 0.5,
                },
                Transform::looking_at(Vec3::ZERO, Vec3::new(-0.7, -0.6, -0.18)),
            ),
            Object::light(
                Light::Environment {
                    color: Rgb::new(0.5, 0.6, 0.8),
                    intensity: 0.08,
                },
                Transform::IDENTITY,
            ),
            Object::primitive(
                Primitive {
                    geometry: Geometry::Mesh(Arc::new(Mesh {
                        vertices: corners.to_vec(),
                        indices: vec![0, 1, 2, 0, 2, 3],
                        normals: Some(vec![normal; 4]),
                        tangents: None,
                        uvs: Some(vec![
                            Vec2::new(0.0, v),
                            Vec2::new(u, v),
                            Vec2::new(u, 0.0),
                            Vec2::new(0.0, 0.0),
                        ]),
                    })),
                    material,
                },
                Transform::IDENTITY,
            ),
        ],
    }
}
