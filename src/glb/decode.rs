use std::collections::HashMap;
use std::sync::Arc;

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

#[derive(Default)]
struct Cache {
    meshes: HashMap<(usize, usize), Arc<Mesh>>,
    images: HashMap<ImageKey, Arc<Texture>>,
}

#[derive(Hash, PartialEq, Eq)]
struct ImageKey {
    index: usize,
    decoding: ImageKeyDecoding,
    texture_wrap: TextureWrap,
}

#[derive(Hash, PartialEq, Eq)]
enum ImageKeyDecoding {
    Srgb { factor: [u32; 3] },
    Normal { scale: u32 },
    Gray { channel: usize, factor: u32 },
}

pub fn decode(bytes: &[u8]) -> Result<(Scene, Option<Parameters>), Error> {
    let (document, buffers, images) = gltf::import_slice(bytes)?;

    let extras = document
        .extension_value(EXTENSION)
        .ok_or("o arquivo não é tcc2")?;

    let gltf_scene = document
        .default_scene()
        .ok_or("o arquivo não tem cena padrão")?;

    let mut objects = Vec::new();
    let mut cache = Cache::default();

    for node in gltf_scene.nodes() {
        objects.push(decode_object(&node, &buffers, &images, &mut cache)?);
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
    cache: &mut Cache,
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
            geometry: decode_geometry(&mesh, &primitive, buffers, cache)?,
            material: decode_material(&primitive.material(), images, cache)?,
        }))
    } else {
        None
    };

    let children = node
        .children()
        .map(|child| decode_object(&child, buffers, images, cache))
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
    cache: &mut Cache,
) -> Result<Geometry, Error> {
    let tag = mesh
        .extension_value(EXTENSION)
        .and_then(|extras| non_null_field(extras, "geometry"));

    match tag {
        None => {
            let mesh_key = (mesh.index(), primitive.index());

            if let Some(cached) = cache.meshes.get(&mesh_key) {
                return Ok(Geometry::Mesh(Arc::clone(cached)));
            }

            let decoded = Arc::new(decode_mesh(primitive, buffers)?);
            cache.meshes.insert(mesh_key, Arc::clone(&decoded));

            Ok(Geometry::Mesh(decoded))
        }

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
    cache: &mut Cache,
) -> Result<Material, Error> {
    let pbr = material.pbr_metallic_roughness();

    let [r, g, b, _] = pbr.base_color_factor();
    let base_color_factor = Rgb::new(r, g, b);

    let albedo = match pbr.base_color_texture() {
        Some(info) => Channel::Map(decode_image(
            &info.texture(),
            ImageKeyDecoding::Srgb {
                factor: [
                    base_color_factor.r.to_bits(),
                    base_color_factor.g.to_bits(),
                    base_color_factor.b.to_bits(),
                ],
            },
            images,
            |texels| {
                texels
                    .into_iter()
                    .map(|t| Rgb::from_srgb(t) * base_color_factor)
                    .collect()
            },
            cache,
        )?),
        None => Channel::Constant(base_color_factor),
    };

    let (metallic, roughness) = match pbr.metallic_roughness_texture() {
        None => (
            Channel::Constant(pbr.metallic_factor()),
            Channel::Constant(pbr.roughness_factor()),
        ),
        // Roughness no verde, metallic no azul, como manda o glTF.
        Some(info) => (
            decode_grayscale_image(&info.texture(), 2, pbr.metallic_factor(), images, cache)?,
            decode_grayscale_image(&info.texture(), 1, pbr.roughness_factor(), images, cache)?,
        ),
    };

    let [r, g, b] = material.emissive_factor();
    let emissive_factor = Rgb::new(r, g, b) * material.emissive_strength().unwrap_or(1.0);

    let emissive = match material.emissive_texture() {
        Some(info) => Channel::Map(decode_image(
            &info.texture(),
            ImageKeyDecoding::Srgb {
                factor: [
                    emissive_factor.r.to_bits(),
                    emissive_factor.g.to_bits(),
                    emissive_factor.b.to_bits(),
                ],
            },
            images,
            |texels| {
                texels
                    .into_iter()
                    .map(|t| Rgb::from_srgb(t) * emissive_factor)
                    .collect()
            },
            cache,
        )?),
        None => Channel::Constant(emissive_factor),
    };

    let normal = match material.normal_texture() {
        Some(info) => {
            let normal_scale = info.scale();

            // O normal map é dado, não cor: sem a curva sRGB, senão os vetores saem
            // todos torcidos.
            Some(decode_image(
                &info.texture(),
                ImageKeyDecoding::Normal {
                    scale: normal_scale.to_bits(),
                },
                images,
                |texels| {
                    texels
                        .into_iter()
                        .map(|t| {
                            Rgb::new(
                                ((Rgb::from_linear(t).r * 2.0 - 1.0) * normal_scale + 1.0) * 0.5,
                                ((Rgb::from_linear(t).g * 2.0 - 1.0) * normal_scale + 1.0) * 0.5,
                                Rgb::from_linear(t).b,
                            )
                        })
                        .collect()
                },
                cache,
            )?)
        }
        None => None,
    };

    Ok(Material {
        albedo,
        metallic,
        roughness,
        emissive,
        normal,
        transparency: material
            .transmission()
            .map_or(0.0, |t| t.transmission_factor()),
        refractive_index: material.ior().unwrap_or(1.5),
    })
}

fn decode_image(
    texture: &gltf::texture::Texture,
    decoding: ImageKeyDecoding,
    images: &[gltf::image::Data],
    texels: impl FnOnce(Vec<[u8; 3]>) -> Vec<Rgb>,
    cache: &mut Cache,
) -> Result<Arc<Texture>, Error> {
    let image_key = ImageKey {
        index: texture.source().index(),
        decoding,
        texture_wrap: match texture.sampler().wrap_s() {
            WrappingMode::ClampToEdge => TextureWrap::Clamp,
            WrappingMode::MirroredRepeat | WrappingMode::Repeat => TextureWrap::Repeat,
        },
    };

    if let Some(decoded) = cache.images.get(&image_key) {
        return Ok(Arc::clone(decoded));
    }

    let (width, height, bytes) = {
        let data = &images[image_key.index];

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
        (data.width, data.height, texels)
    };

    let decoded = Arc::new(decode_texture(width, height, texels(bytes), texture)?);

    cache.images.insert(image_key, Arc::clone(&decoded));

    Ok(decoded)
}

fn decode_grayscale_image(
    texture: &gltf::texture::Texture,
    channel: usize,
    factor: f32,
    images: &[gltf::image::Data],
    cache: &mut Cache,
) -> Result<Channel<f32>, Error> {
    let flat = images[texture.source().index()]
        .pixels
        .chunks_exact(3)
        .all(|t| t[channel] == u8::MAX);

    if flat {
        return Ok(Channel::Constant(factor));
    }

    Ok(Channel::Map(decode_image(
        texture,
        ImageKeyDecoding::Gray {
            channel,
            factor: factor.to_bits(),
        },
        images,
        |texels| {
            texels
                .iter()
                .map(|t| Rgb::gray(t[channel] as f32 / 255.0 * factor))
                .collect()
        },
        cache,
    )?))
}

fn decode_texture(
    width: u32,
    height: u32,
    texels: Vec<Rgb>,
    texture: &gltf::texture::Texture,
) -> Result<Texture, Error> {
    let mut decoded = Texture::new(width, height, texels)?;

    decoded.wrap = match texture.sampler().wrap_s() {
        WrappingMode::ClampToEdge => TextureWrap::Clamp,
        WrappingMode::MirroredRepeat | WrappingMode::Repeat => TextureWrap::Repeat,
    };

    Ok(decoded)
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
