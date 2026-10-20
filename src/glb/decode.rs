use gltf::image::Format;
use gltf::texture::WrappingMode;
use serde_json::Value;

use crate::color::Rgb;
use crate::geometry::{Geometry, Mesh};
use crate::math::{Tangent, Vec2, Vec3};
use crate::parameters::Parameters;
use crate::scene::{Camera, Channel, Light, Material, Object, ObjectKind, Primitive, Scene};
use crate::texture::{Texture, TextureWrap};
use crate::transform::Transform;

use super::{EXTENSION, Error, LUMENS_PER_WATT, non_null_field};

pub fn decode(bytes: &[u8]) -> Result<(Scene, Option<Parameters>), Error> {
    let (document, buffers, images) = gltf::import_slice(bytes)?;

    let extras = document
        .extension_value(EXTENSION)
        .ok_or("o arquivo não é tcc2")?;

    let gltf_scene = document
        .default_scene()
        .ok_or("o arquivo não tem cena padrão")?;

    let mut objects = Vec::new();
    let mut materials = Vec::new();

    for node in gltf_scene.nodes() {
        objects.push(decode_object(&node, &buffers, &images, &mut materials)?);
    }

    let scene = Scene { objects };

    let parameters = non_null_field(extras, "parameters").map(super::parameters::decode);

    Ok((scene, parameters))
}

fn decode_camera(camera: &gltf::Camera) -> Camera {
    match camera.projection() {
        gltf::camera::Projection::Perspective(p) => Camera::Perspective {
            fov: p.yfov().to_degrees(),
            aspect: p.aspect_ratio().unwrap_or(0.1),
            near: p.znear(),
            far: p.zfar().unwrap_or(1000.0),
        },

        gltf::camera::Projection::Orthographic(o) => Camera::Orthographic {
            width: 2.0 * o.xmag(),
            height: 2.0 * o.ymag(),
            near: o.znear(),
            far: o.zfar(),
        },
    }
}

fn decode_light(light: &gltf::khr_lights_punctual::Light) -> Light {
    let color = Rgb::new(light.color()[0], light.color()[1], light.color()[2]);
    let intensity = light.intensity() / LUMENS_PER_WATT;

    match light.kind() {
        gltf::khr_lights_punctual::Kind::Point => Light::Point { color, intensity },

        gltf::khr_lights_punctual::Kind::Directional => Light::Directional { color, intensity },

        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => Light::Spot {
            inner_angle: inner_cone_angle.to_degrees(),
            outer_angle: outer_cone_angle.to_degrees(),
            color,
            intensity,
        },
    }
}

fn decode_object(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    materials: &mut Vec<(Option<usize>, Material)>,
) -> Result<Object, Error> {
    let extras = node.extension_value(EXTENSION);

    let content = if let Some(projection) = node.camera() {
        Some(ObjectKind::Camera(decode_camera(&projection)))
    } else if let Some(light) = node.light() {
        Some(ObjectKind::Light(decode_light(&light)))
    } else if let Some(environment) = extras.and_then(|e| non_null_field(e, "environment")) {
        Some(ObjectKind::Light(Light::Environment {
            color: non_null_field(environment, "color")
                .and_then(|c| {
                    Some(Rgb::new(
                        c.get(0)?.as_f64()? as f32,
                        c.get(1)?.as_f64()? as f32,
                        c.get(2)?.as_f64()? as f32,
                    ))
                })
                .unwrap_or(Rgb::WHITE),
            intensity: non_null_field(environment, "intensity")
                .and_then(|i| i.as_f64().map(|i| i as f32))
                .unwrap_or(1.0),
        }))
    } else if let Some(mesh) = node.mesh() {
        let primitive = mesh.primitives().next().ok_or("mesh sem primitive")?;

        Some(ObjectKind::Primitive(Primitive {
            geometry: decode_geometry(&mesh, &primitive, buffers)?,
            material: decode_material(&primitive.material(), images, materials)?,
        }))
    } else {
        None
    };

    let children = node
        .children()
        .map(|child| decode_object(&child, buffers, images, materials))
        .collect::<Result<Vec<_>, Error>>()?;

    let (translation, rotation, scale) = node.transform().decomposed();

    let transform = Transform {
        translation: Vec3::new(translation[0], translation[1], translation[2]),
        rotation: euler(rotation),
        scale: Vec3::new(scale[0], scale[1], scale[2]),
    };

    Ok(Object {
        transform,
        kind: content,
        children,
    })
}

fn decode_geometry(
    mesh: &gltf::Mesh,
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
) -> Result<Geometry, Error> {
    let tag = mesh
        .extension_value(EXTENSION)
        .and_then(|extras| non_null_field(extras, "geometry"));

    match tag {
        None => Ok(Geometry::Mesh(decode_mesh(primitive, buffers)?)),

        Some(Value::String(name)) if name == "triangle" => {
            let mesh: &Mesh = &decode_mesh(primitive, buffers)?;

            fn three<T: Copy>(values: &[T], trio: [usize; 3]) -> Option<[T; 3]> {
                Some([
                    *values.get(trio[0])?,
                    *values.get(trio[1])?,
                    *values.get(trio[2])?,
                ])
            }

            let trio = mesh
                .indices
                .get(..3)
                .and_then(|i| <[u32; 3]>::try_from(i).ok())
                .map(|i| i.map(|k| k as usize))
                .ok_or("triangle com menos de três índices")?;

            Ok(Geometry::Triangle {
                points: three(&mesh.vertices, trio)
                    .ok_or("triangle com índice fora da lista de vértices")?,
            })
        }

        Some(v) => {
            if non_null_field(v, "sphere").is_some() {
                let sphere = &v["sphere"];

                match non_null_field(sphere, "radius").and_then(|r| r.as_f64().map(|r| r as f32)) {
                    Some(radius) => Ok(Geometry::Sphere { radius }),
                    _ => Err("sphere sem radius".into()),
                }
            } else if non_null_field(v, "cube").is_some() {
                let cube = &v["cube"];

                match non_null_field(cube, "half_edge").and_then(|h| h.as_f64().map(|h| h as f32)) {
                    Some(half_edge) => Ok(Geometry::Cube { half_edge }),
                    _ => Err("cube sem half_edge".into()),
                }
            } else {
                Err("geometry desconhecida.".into())
            }
        }
    }
}

fn decode_mesh(primitive: &gltf::Primitive, buffers: &[gltf::buffer::Data]) -> Result<Mesh, Error> {
    let reader = primitive.reader(|b| Some(&buffers[b.index()][..]));

    let vertices: Vec<Vec3> = reader
        .read_positions()
        .ok_or("primitive without POSITION")?
        .map(|p| Vec3::new(p[0], p[1], p[2]))
        .collect();

    let normals = reader
        .read_normals()
        .map(|n| n.map(|n| Vec3::new(n[0], n[1], n[2])).collect());

    let tangents = reader.read_tangents().map(|tangents| {
        tangents
            .map(|t| Tangent::new(Vec3::new(t[0], t[1], t[2]), t[3]))
            .collect()
    });

    let uvs = reader
        .read_tex_coords(0)
        .map(|uvs| uvs.into_f32().map(|uv| Vec2::new(uv[0], uv[1])).collect());

    let indices = match reader.read_indices() {
        Some(indices) => indices.into_u32().collect(),
        None => (0..vertices.len() as u32).collect(),
    };

    Ok(Mesh {
        vertices,
        indices,
        normals,
        tangents,
        uvs,
    })
}

fn decode_material(
    material: &gltf::Material,
    images: &[gltf::image::Data],
    materials: &mut Vec<(Option<usize>, Material)>,
) -> Result<Material, Error> {
    if let Some((_, cached)) = materials.iter().find(|(i, _)| *i == material.index()) {
        return Ok(cached.clone());
    }

    let pbr = material.pbr_metallic_roughness();

    let albedo = match pbr.base_color_texture() {
        Some(info) => {
            let (width, height, texels) = pop_image(info.texture().source().index(), images)?;
            let texels = texels.into_iter().map(Rgb::from_srgb).collect();

            Channel::Map(decode_texture(width, height, texels, info.texture())?)
        }
        None => {
            let [r, g, b, _] = pbr.base_color_factor();
            Channel::Constant(Rgb::new(r, g, b))
        }
    };

    let (metallic, roughness) = match pbr.metallic_roughness_texture() {
        None => (
            Channel::Constant(pbr.metallic_factor()),
            Channel::Constant(pbr.roughness_factor()),
        ),
        Some(info) => {
            let (width, height, texels) = pop_image(info.texture().source().index(), images)?;

            // Roughness no verde, metallic no azul, como manda o glTF.
            let channel = |i: usize, factor: f32| -> Result<Channel<f32>, Error> {
                if texels.iter().all(|t| t[i] == u8::MAX) {
                    return Ok(Channel::Constant(factor));
                }

                let grays = texels
                    .iter()
                    .map(|t| Rgb::gray(t[i] as f32 / 255.0 * factor))
                    .collect();

                Ok(Channel::Map(decode_texture(
                    width,
                    height,
                    grays,
                    info.texture(),
                )?))
            };

            (
                channel(2, pbr.metallic_factor())?,
                channel(1, pbr.roughness_factor())?,
            )
        }
    };

    let [r, g, b] = material.emissive_factor();
    let factor = Rgb::new(r, g, b) * material.emissive_strength().unwrap_or(1.0);

    let emissive = match material.emissive_texture() {
        Some(info) => {
            let (width, height, texels) = pop_image(info.texture().source().index(), images)?;
            let texels = texels
                .into_iter()
                .map(|t| Rgb::from_srgb(t) * factor)
                .collect();

            Channel::Map(decode_texture(width, height, texels, info.texture())?)
        }
        None => Channel::Constant(factor),
    };

    let normal = match material.normal_texture() {
        Some(info) => {
            let (width, height, texels) = pop_image(info.texture().source().index(), images)?;
            // O normal map é dado, não cor: sem a curva sRGB, senão os vetores
            // saem todos torcidos.
            let texels = texels.into_iter().map(Rgb::from_linear).collect();

            Some(decode_texture(width, height, texels, info.texture())?)
        }
        None => None,
    };

    let converted = Material {
        albedo,
        metallic,
        roughness,
        emissive,
        normal,
        transparency: material
            .transmission()
            .map_or(0.0, |t| t.transmission_factor()),
        refractive_index: material.ior().unwrap_or(1.5),
    };

    materials.push((material.index(), converted.clone()));

    Ok(converted)
}

fn decode_texture(
    width: u32,
    height: u32,
    texels: Vec<Rgb>,
    texture: gltf::texture::Texture,
) -> Result<Texture, Error> {
    let mut decoded = Texture::new(width, height, texels)?;

    decoded.wrap = match texture.sampler().wrap_s() {
        WrappingMode::ClampToEdge => TextureWrap::Clamp,
        WrappingMode::MirroredRepeat | WrappingMode::Repeat => TextureWrap::Repeat,
    };

    Ok(decoded)
}

fn pop_image(
    index: usize,
    images: &[gltf::image::Data],
) -> Result<(u32, u32, Vec<[u8; 3]>), Error> {
    let data = &images[index];

    match data.format {
        Format::R8G8B8 => (),
        _ => {
            return Err("image não está em RGB8".into());
        }
    }

    let texels = data
        .pixels
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    Ok((data.width, data.height, texels))
}

fn euler(quaternion: [f32; 4]) -> Vec3 {
    let [x, y, z, w] = quaternion;

    let m00 = 1.0 - 2.0 * (y * y + z * z);
    let m01 = 2.0 * (x * y - z * w);
    let m10 = 2.0 * (x * y + z * w);
    let m11 = 1.0 - 2.0 * (x * x + z * z);
    let m20 = 2.0 * (x * z - y * w);
    let m21 = 2.0 * (y * z + x * w);
    let m22 = 1.0 - 2.0 * (x * x + y * y);

    let beta = (-m20).clamp(-1.0, 1.0).asin();

    if m20.abs() > 0.9999 {
        return Vec3::new((-m20 * m01).atan2(m11).to_degrees(), beta.to_degrees(), 0.0);
    }

    Vec3::new(
        m21.atan2(m22).to_degrees(),
        beta.to_degrees(),
        m10.atan2(m00).to_degrees(),
    )
}
