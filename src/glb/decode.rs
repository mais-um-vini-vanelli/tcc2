use gltf::image::Format;
use serde_json::Value;

use crate::color::Rgb;
use crate::math::{Tangent, Vec2, Vec3};
use crate::parameters::Parameters;
use crate::scene::{Camera, Light, Material, Mesh, Object, Primitive, Scene, Value as Canal};
use crate::texture::Texture;
use crate::transform::Transform;

use super::{EXTENSION, Error, non_null_field};

pub fn decode(bytes: &[u8]) -> Result<(Scene, Option<Parameters>), Error> {
    let (documento, buffers, imagens) = gltf::import_slice(bytes)?;

    let extras = documento
        .extension_value(EXTENSION)
        .ok_or("o arquivo não é tcc2")?;

    let cena_gltf = documento
        .default_scene()
        .ok_or("o arquivo não tem cena padrão")?;

    let mut camera = None;
    let mut lights = Vec::new();
    let mut objects = Vec::new();
    let mut materiais = Vec::new();

    for no in cena_gltf.nodes() {
        if let Some(projecao) = no.camera() {
            camera = Some(decode_camera(&projecao, &no));
        } else if let Some(luz) = no.light() {
            lights.push(decode_light(&luz, &no));
        } else if let Some(malha) = no.mesh() {
            objects.push(decode_object(
                &no,
                &malha,
                &buffers,
                &imagens,
                &mut materiais,
            )?);
        }
    }

    let camera = camera.ok_or("a cena não tem câmera")?;

    // As ambientes voltam no fim da lista.
    if let Some(ambientes) = non_null_field(extras, "ambient").and_then(Value::as_array) {
        for ambiente in ambientes {
            lights.push(Light::Ambient {
                color: non_null_field(ambiente, "color")
                    .and_then(|c| {
                        Some(Rgb::new(
                            c.get(0)?.as_f64()? as f32,
                            c.get(1)?.as_f64()? as f32,
                            c.get(2)?.as_f64()? as f32,
                        ))
                    })
                    .unwrap_or(Rgb::WHITE),
                intensity: non_null_field(ambiente, "intensity")
                    .and_then(|i| i.as_f64().map(|i| i as f32))
                    .unwrap_or(1.0),
            });
        }
    }

    let parameters = non_null_field(extras, "parameters").map(super::parameters::decode);

    Ok((
        Scene {
            camera,
            lights,
            objects,
        },
        parameters,
    ))
}

fn decode_camera(projecao: &gltf::Camera, no: &gltf::Node) -> Camera {
    let matrix = no.transform().matrix();

    let position = column(&matrix, 3);
    let up = direction(&matrix, Vec3::new(0.0, 1.0, 0.0));
    let look_at = position + direction(&matrix, Vec3::new(0.0, 0.0, -1.0));

    match projecao.projection() {
        gltf::camera::Projection::Perspective(p) => Camera::Perspective {
            fov: p.yfov().to_degrees(),
            aspect: p.aspect_ratio().unwrap_or(0.1),
            near: p.znear(),
            far: p.zfar().unwrap_or(1000.0),
            position,
            look_at,
            up,
        },

        gltf::camera::Projection::Orthographic(o) => Camera::Orthographic {
            width: 2.0 * o.xmag(),
            height: 2.0 * o.ymag(),
            near: o.znear(),
            far: o.zfar(),
            position,
            look_at,
            up,
        },
    }
}

fn decode_light(luz: &gltf::khr_lights_punctual::Light, no: &gltf::Node) -> Light {
    let matrix = no.transform().matrix();

    let position = column(&matrix, 3);
    let direction = direction(&matrix, Vec3::new(0.0, 0.0, -1.0));

    let color = Rgb::new(luz.color()[0], luz.color()[1], luz.color()[2]);
    let intensity = luz.intensity();

    match luz.kind() {
        gltf::khr_lights_punctual::Kind::Point => Light::Point {
            position,
            color,
            intensity,
        },

        gltf::khr_lights_punctual::Kind::Directional => Light::Directional {
            direction,
            color,
            intensity,
        },

        gltf::khr_lights_punctual::Kind::Spot {
            inner_cone_angle,
            outer_cone_angle,
        } => Light::Spot {
            position,
            direction,
            inner_angle: inner_cone_angle.to_degrees(),
            outer_angle: outer_cone_angle.to_degrees(),
            color,
            intensity,
        },
    }
}

fn decode_object(
    no: &gltf::Node,
    malha: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
    imagens: &[gltf::image::Data],
    materiais: &mut Vec<(Option<usize>, Material)>,
) -> Result<Object, Error> {
    let primitive = malha.primitives().next().ok_or("malha sem primitive")?;

    let material = decode_material(&primitive.material(), imagens, materiais)?;
    let geometry = decode_geometry(malha, &primitive, buffers)?;

    let (translation, rotation, scale) = no.transform().decomposed();

    let casts_shadows = no
        .extension_value(EXTENSION)
        .and_then(|extras| non_null_field(extras, "casts_shadows"))
        .and_then(Value::as_bool)
        .unwrap_or(true);

    Ok(Object {
        geometry,
        material,
        transform: Transform {
            translation: Vec3::new(translation[0], translation[1], translation[2]),
            rotation: euler(rotation),
            scale: Vec3::new(scale[0], scale[1], scale[2]),
        },
        casts_shadows,
    })
}

fn decode_geometry(
    malha: &gltf::Mesh,
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
) -> Result<Primitive, Error> {
    let marca = malha
        .extension_value(EXTENSION)
        .and_then(|extras| non_null_field(extras, "primitive"));

    match marca {
        None => Ok(Primitive::Mesh(decode_mesh(primitive, buffers)?)),

        Some(Value::String(nome)) if nome == "triangle" => {
            let malha: &Mesh = &decode_mesh(primitive, buffers)?;

            fn tres<T: Copy>(valores: &[T], trio: [usize; 3]) -> Option<[T; 3]> {
                Some([
                    *valores.get(trio[0])?,
                    *valores.get(trio[1])?,
                    *valores.get(trio[2])?,
                ])
            }

            let trio = malha
                .indices
                .get(..3)
                .and_then(|i| <[u32; 3]>::try_from(i).ok())
                .map(|i| i.map(|k| k as usize))
                .ok_or("triangle com menos de três índices")?;

            Ok(Primitive::Triangle {
                points: tres(&malha.vertices, trio)
                    .ok_or("triangle com índice fora da lista de vértices")?,
                normals: malha.normals.as_deref().and_then(|n| tres(n, trio)),
                tangents: malha.tangents.as_deref().and_then(|t| tres(t, trio)),
                uvs: malha.uvs.as_deref().and_then(|u| tres(u, trio)),
            })
        }

        Some(v) => {
            let to_vec3 = |val: &Value| -> Option<Vec3> {
                Some(Vec3::new(
                    val.get(0)?.as_f64()? as f32,
                    val.get(1)?.as_f64()? as f32,
                    val.get(2)?.as_f64()? as f32,
                ))
            };

            if non_null_field(v, "sphere").is_some() {
                let esfera = &v["sphere"];

                match (
                    non_null_field(esfera, "center").and_then(to_vec3),
                    non_null_field(esfera, "radius").and_then(|r| r.as_f64().map(|r| r as f32)),
                ) {
                    (Some(center), Some(radius)) => Ok(Primitive::Sphere { center, radius }),
                    _ => Err("sphere sem center ou radius".into()),
                }
            } else if non_null_field(v, "cube").is_some() {
                let cubo = &v["cube"];

                match (
                    non_null_field(cubo, "center").and_then(to_vec3),
                    non_null_field(cubo, "half_edge").and_then(|h| h.as_f64().map(|h| h as f32)),
                ) {
                    (Some(center), Some(half_edge)) => Ok(Primitive::Cube { center, half_edge }),
                    _ => Err("cube sem center ou half_edge".into()),
                }
            } else {
                Err("primitive desconhecido.".into())
            }
        }
    }
}

fn decode_mesh(primitive: &gltf::Primitive, buffers: &[gltf::buffer::Data]) -> Result<Mesh, Error> {
    let leitor = primitive.reader(|b| Some(&buffers[b.index()][..]));

    let vertices: Vec<Vec3> = leitor
        .read_positions()
        .ok_or("primitive sem POSITION")?
        .map(|p| Vec3::new(p[0], p[1], p[2]))
        .collect();

    let normals = leitor
        .read_normals()
        .map(|n| n.map(|n| Vec3::new(n[0], n[1], n[2])).collect());

    let tangents = leitor.read_tangents().map(|tangentes| {
        tangentes
            .map(|t| Tangent::new(Vec3::new(t[0], t[1], t[2]), t[3]))
            .collect()
    });

    let uvs = leitor
        .read_tex_coords(0)
        .map(|uvs| uvs.into_f32().map(|uv| Vec2::new(uv[0], uv[1])).collect());

    let indices = match leitor.read_indices() {
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
    imagens: &[gltf::image::Data],
    materiais: &mut Vec<(Option<usize>, Material)>,
) -> Result<Material, Error> {
    if let Some((_, pronto)) = materiais.iter().find(|(i, _)| *i == material.index()) {
        return Ok(pronto.clone());
    }

    let pbr = material.pbr_metallic_roughness();

    let albedo = match pbr.base_color_texture() {
        Some(info) => {
            let (largura, altura, texels) = pop_image(info.texture().source().index(), imagens)?;
            let texels = texels.into_iter().map(Rgb::from_srgb).collect();

            Canal::Map(Texture::new(largura, altura, texels)?)
        }
        None => {
            let [r, g, b, _] = pbr.base_color_factor();
            Canal::Constant(Rgb::new(r, g, b))
        }
    };

    let (metallic, roughness) = match pbr.metallic_roughness_texture() {
        None => (
            Canal::Constant(pbr.metallic_factor()),
            Canal::Constant(pbr.roughness_factor()),
        ),
        Some(info) => {
            let (largura, altura, texels) = pop_image(info.texture().source().index(), imagens)?;

            // Roughness no verde, metallic no azul, como manda o glTF.
            let canal = |i: usize, fator: f32| -> Result<Canal<f32>, Error> {
                if texels.iter().all(|t| t[i] == u8::MAX) {
                    return Ok(Canal::Constant(fator));
                }

                let cinzas = texels
                    .iter()
                    .map(|t| Rgb::gray(t[i] as f32 / 255.0 * fator))
                    .collect();

                Ok(Canal::Map(Texture::new(largura, altura, cinzas)?))
            };

            (
                canal(2, pbr.metallic_factor())?,
                canal(1, pbr.roughness_factor())?,
            )
        }
    };

    let forca = material.emissive_strength().unwrap_or(1.0);
    let emissive = match material.emissive_texture() {
        Some(info) => {
            let (largura, altura, texels) = pop_image(info.texture().source().index(), imagens)?;
            let texels = texels
                .into_iter()
                .map(|t| Rgb::from_srgb(t) * forca)
                .collect();

            Canal::Map(Texture::new(largura, altura, texels)?)
        }
        None => {
            let [r, g, b] = material.emissive_factor();
            let fator = Rgb::new(r, g, b) * forca;

            Canal::Constant(fator)
        }
    };

    let normal = match material.normal_texture() {
        Some(info) => {
            let (largura, altura, texels) = pop_image(info.texture().source().index(), imagens)?;
            // O normal map é dado, não cor: sem a curva sRGB, senão os vetores
            // saem todos torcidos.
            let texels = texels.into_iter().map(Rgb::from_linear).collect();

            Some(Texture::new(largura, altura, texels)?)
        }
        None => None,
    };

    let convertido = Material {
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

    materiais.push((material.index(), convertido.clone()));

    Ok(convertido)
}

fn pop_image(
    indice: usize,
    imagens: &[gltf::image::Data],
) -> Result<(u32, u32, Vec<[u8; 3]>), Error> {
    let dados = &imagens[indice];

    match dados.format {
        Format::R8G8B8 => (),
        _ => {
            return Err("imagem não está em RGB8".into());
        }
    }

    let texels = dados
        .pixels
        .chunks_exact(3)
        .map(|t| [t[0], t[1], t[2]])
        .collect();

    Ok((dados.width, dados.height, texels))
}

fn column(mat: &[[f32; 4]; 4], col: usize) -> Vec3 {
    Vec3::new(mat[col][0], mat[col][1], mat[col][2])
}

fn direction(m: &[[f32; 4]; 4], v: Vec3) -> Vec3 {
    column(m, 0) * v.x + column(m, 1) * v.y + column(m, 2) * v.z
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
