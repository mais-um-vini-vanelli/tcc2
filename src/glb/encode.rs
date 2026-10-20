use std::borrow::Cow;
use std::collections::BTreeMap;

use gltf::json::{self, validation::Checked::Valid, validation::USize64};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use serde_json::{Map, Value, json};

use crate::color::Rgb;
use crate::geometry::{Geometry, Mesh};
use crate::math::{Aabb, Vec3};
use crate::parameters::Parameters;
use crate::scene::{self, Camera, Light, Material, Object, ObjectKind, Primitive, Scene};
use crate::texture::{Texture, TextureWrap};
use crate::transform::Transform;

use super::{EXTENSION, Error, LUMENS_PER_WATT, VERSION};

pub fn encode(scene: &Scene, parameters: Option<&Parameters>) -> Result<Vec<u8>, Error> {
    let mut json_gltf = json::Root::default();

    let mut bin = Vec::new();
    let mut materials = Vec::new();

    json_gltf.asset.generator = Some(format!("tcc2 {}", env!("CARGO_PKG_VERSION")));

    json_gltf.scene = Some(json_gltf.push(json::Scene {
        extensions: None,
        extras: Default::default(),
        name: None,
        nodes: Vec::new(),
    }));

    for wrap in [
        json::texture::WrappingMode::Repeat,
        json::texture::WrappingMode::ClampToEdge,
    ] {
        json_gltf.push(json::texture::Sampler {
            mag_filter: Some(Valid(json::texture::MagFilter::Linear)),
            min_filter: Some(Valid(json::texture::MinFilter::LinearMipmapLinear)),
            wrap_s: Valid(wrap),
            wrap_t: Valid(wrap),
            name: None,
            extensions: None,
            extras: Default::default(),
        });
    }

    for object in &scene.objects {
        let index = encode_object(object, &mut json_gltf, &mut bin, &mut materials)?;
        json_gltf.scenes[0].nodes.push(index);
    }

    let mut extras = Map::new();
    extras.insert("version".into(), json!(VERSION));

    if let Some(parameters) = parameters {
        extras.insert("parameters".into(), super::parameters::encode(parameters));
    }

    json_gltf
        .extensions
        .get_or_insert_with(Default::default)
        .others
        .insert(EXTENSION.into(), Value::Object(extras));

    json_gltf.extensions_used = [
        "KHR_lights_punctual",
        "KHR_materials_emissive_strength",
        "KHR_materials_ior",
        "KHR_materials_transmission",
        EXTENSION,
    ]
    .map(|ext| ext.into())
    .to_vec();

    json_gltf.push(json::Buffer {
        byte_length: USize64::from(bin.len()),
        extensions: None,
        extras: Default::default(),
        name: None,
        uri: None,
    });

    let json = json::serialize::to_string(&json_gltf)?;

    gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: 0,
        },
        json: Cow::Owned(json.into_bytes()),
        bin: Some(Cow::Owned(bin)),
    }
    .to_vec()
    .map_err(Error::from)
}

fn encode_camera(camera: Camera, json_gltf: &mut json::Root) -> json::Node {
    let camera_gltf = match camera {
        Camera::Perspective {
            fov,
            aspect,
            near,
            far,
        } => json::Camera {
            type_: Valid(json::camera::Type::Perspective),
            perspective: Some(json::camera::Perspective {
                aspect_ratio: Some(aspect),
                yfov: fov.to_radians(),
                znear: near,
                zfar: Some(far),
                extensions: None,
                extras: Default::default(),
            }),
            orthographic: None,
            name: None,
            extensions: None,
            extras: Default::default(),
        },

        Camera::Orthographic {
            width,
            height,
            near,
            far,
        } => json::Camera {
            type_: Valid(json::camera::Type::Orthographic),
            orthographic: Some(json::camera::Orthographic {
                xmag: width * 0.5,
                ymag: height * 0.5,
                znear: near,
                zfar: far,
                extensions: None,
                extras: Default::default(),
            }),
            perspective: None,
            name: None,
            extensions: None,
            extras: Default::default(),
        },
    };

    let camera_gltf = json_gltf.push(camera_gltf);

    json::Node {
        camera: Some(camera_gltf),
        ..Default::default()
    }
}

fn encode_light(light: Light, json_gltf: &mut json::Root) -> json::Node {
    use json::extensions::scene::khr_lights_punctual as khr;

    if let Light::Environment { color, intensity } = light {
        // O ambiente não tem equivalente no glTF: ele vai para uma extension.
        return json::Node {
            extensions: Some(json::extensions::scene::Node {
                others: Map::from_iter([(
                    EXTENSION.into(),
                    json!({ "environment": { "color": [color.r, color.g, color.b], "intensity": intensity } }),
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
    }

    let punctual = |kind, spot| khr::Light {
        type_: Valid(kind),
        color: [light.color().r, light.color().g, light.color().b],
        intensity: light.intensity() * LUMENS_PER_WATT,
        range: None,
        spot,
        extensions: None,
        extras: Default::default(),
        name: None,
    };

    let encoded = match light {
        Light::Point { .. } => punctual(khr::Type::Point, None),

        Light::Directional { .. } => punctual(khr::Type::Directional, None),

        Light::Spot {
            inner_angle,
            outer_angle,
            ..
        } => punctual(
            khr::Type::Spot,
            Some(khr::Spot {
                inner_cone_angle: inner_angle.to_radians(),
                outer_cone_angle: outer_angle.to_radians(),
            }),
        ),

        Light::Environment { .. } => unreachable!(),
    };

    let light_gltf = json_gltf.push(encoded);

    json::Node {
        extensions: Some(json::extensions::scene::Node {
            khr_lights_punctual: Some(khr::KhrLightsPunctual { light: light_gltf }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn encode_object(
    object: &Object,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
    materials: &mut Vec<(Material, json::Index<json::Material>)>,
) -> Result<json::Index<json::Node>, Error> {
    let children = object
        .children
        .iter()
        .map(|child| encode_object(child, json_gltf, bin, materials))
        .collect::<Result<Vec<_>, Error>>()?;

    let children = (!children.is_empty()).then_some(children);

    let mut node = match &object.kind {
        None => json::Node::default(),

        Some(ObjectKind::Camera(camera)) => encode_camera(*camera, json_gltf),

        Some(ObjectKind::Light(light)) => encode_light(*light, json_gltf),

        Some(ObjectKind::Primitive(Primitive { geometry, material })) => {
            let material_gltf = encode_material(material, json_gltf, bin, materials)?;

            let tessellated = geometry.tessellate();

            if tessellated.vertices.is_empty() || tessellated.indices.is_empty() {
                return Err("object sem nenhum triângulo".into());
            }

            let mesh_gltf = encode_mesh(&tessellated, material_gltf, json_gltf, bin);

            // Esfera e cubo vão com a malha tesselada **e** com os campos exatos: o
            // decoder daqui lê os campos e ignora a malha, e quem não conhece a
            // extensão fica com a malha.
            let geometry_tag = match geometry {
                Geometry::Mesh(_) => None,
                Geometry::Triangle { .. } => Some(json!("triangle")),
                Geometry::Sphere { radius } => Some(json!({
                    "sphere": { "radius": radius }
                })),
                Geometry::Cube { half_edge } => Some(json!({
                    "cube": { "half_edge": half_edge }
                })),
            };

            json_gltf.meshes[mesh_gltf.value()].extensions =
                geometry_tag.map(|tag| json::extensions::mesh::Mesh {
                    others: Map::from_iter([(EXTENSION.into(), json!({ "geometry": tag }))]),
                });

            json::Node {
                mesh: Some(mesh_gltf),
                ..Default::default()
            }
        }
    };

    let Transform {
        translation,
        rotation,
        scale,
    } = object.transform;

    node.translation = Some([translation.x, translation.y, translation.z]);
    node.rotation = Some(json::scene::UnitQuaternion(quaternion(rotation)));
    node.scale = Some([scale.x, scale.y, scale.z]);

    node.children = children;

    Ok(json_gltf.push(node))
}

fn encode_mesh(
    mesh: &Mesh,
    material_gltf: json::Index<json::Material>,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> json::Index<json::Mesh> {
    use json::accessor::{ComponentType, Type};

    let mut attributes = BTreeMap::new();

    let positions: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let positions = push_accessor(
        flatten(&positions),
        positions.len(),
        Type::Vec3,
        ComponentType::F32,
        json_gltf,
        bin,
    );

    let Aabb { min, max } = Aabb::around(mesh.vertices.iter().copied());
    json_gltf.accessors[positions.value()].min = Some(json!([min.x, min.y, min.z]));
    json_gltf.accessors[positions.value()].max = Some(json!([max.x, max.y, max.z]));

    attributes.insert(Valid(json::mesh::Semantic::Positions), positions);

    if let Some(normals) = &mesh.normals {
        let data: Vec<[f32; 3]> = normals.iter().map(|n| [n.x, n.y, n.z]).collect();
        let normals = push_accessor(
            flatten(&data),
            data.len(),
            Type::Vec3,
            ComponentType::F32,
            json_gltf,
            bin,
        );
        attributes.insert(Valid(json::mesh::Semantic::Normals), normals);
    }

    // O TANGENT do glTF é vec4: xyz é a tangente, w é o lado da bitangente.
    if let Some(tangents) = &mesh.tangents {
        let data: Vec<[f32; 4]> = tangents
            .iter()
            .map(|t| [t.direction.x, t.direction.y, t.direction.z, t.handedness])
            .collect();

        let tangents = push_accessor(
            flatten(&data),
            data.len(),
            Type::Vec4,
            ComponentType::F32,
            json_gltf,
            bin,
        );

        attributes.insert(Valid(json::mesh::Semantic::Tangents), tangents);
    }

    if let Some(uvs) = &mesh.uvs {
        let data: Vec<[f32; 2]> = uvs.iter().map(|uv| [uv.x, uv.y]).collect();

        let uvs = push_accessor(
            flatten(&data),
            data.len(),
            Type::Vec2,
            ComponentType::F32,
            json_gltf,
            bin,
        );

        attributes.insert(Valid(json::mesh::Semantic::TexCoords(0)), uvs);
    }

    let bytes: Vec<u8> = mesh.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let indices = push_accessor(
        bytes,
        mesh.indices.len(),
        Type::Scalar,
        ComponentType::U32,
        json_gltf,
        bin,
    );

    let primitive = json::mesh::Primitive {
        attributes,
        indices: Some(indices),
        material: Some(material_gltf),
        mode: Valid(json::mesh::Mode::Triangles),
        targets: None,
        extensions: None,
        extras: Default::default(),
    };

    json_gltf.push(json::Mesh {
        primitives: vec![primitive],
        weights: None,
        name: None,
        extensions: None,
        extras: Default::default(),
    })
}

fn push_accessor(
    bytes: Vec<u8>,
    count: usize,
    kind: json::accessor::Type,
    component: json::accessor::ComponentType,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> json::Index<json::accessor::Accessor> {
    let target = match component {
        json::accessor::ComponentType::U32 => json::buffer::Target::ElementArrayBuffer,
        _ => json::buffer::Target::ArrayBuffer,
    };

    let view = push_buffer_view(&bytes, Some(target), json_gltf, bin);

    json_gltf.push(json::accessor::Accessor {
        buffer_view: Some(view),
        byte_offset: Some(USize64(0)),
        count: USize64::from(count),
        component_type: Valid(json::accessor::GenericComponentType(component)),
        type_: Valid(kind),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
        extensions: None,
        extras: Default::default(),
    })
}

fn push_buffer_view(
    bytes: &[u8],
    target: Option<json::buffer::Target>,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> json::Index<json::buffer::View> {
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }

    let offset = bin.len();
    bin.extend_from_slice(bytes);

    json_gltf.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: USize64::from(bytes.len()),
        byte_offset: Some(USize64::from(offset)),
        byte_stride: None,
        target: target.map(Valid),
        name: None,
        extensions: None,
        extras: Default::default(),
    })
}

fn encode_material(
    material: &Material,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
    materials: &mut Vec<(Material, json::Index<json::Material>)>,
) -> Result<json::Index<json::Material>, Error> {
    if let Some((_, index)) = materials.iter().find(|(j, _)| j == material) {
        return Ok(*index);
    }

    let (base_color_factor, base_color_texture) = match &material.albedo {
        scene::Channel::Constant(color) => ([color.r, color.g, color.b, 1.0], None),
        scene::Channel::Map(map) => (
            [1.0; 4],
            Some({
                push_image(
                    map.width(),
                    map.height(),
                    &map.texels()
                        .iter()
                        .flat_map(|c| c.to_srgb())
                        .collect::<Vec<u8>>(),
                    map.wrap,
                    json_gltf,
                    bin,
                )?
            }),
        ),
    };

    let (metallic_factor, roughness_factor, metallic_roughness_texture) =
        encode_metallic_roughness(material, json_gltf, bin)?;

    let normal_texture = match &material.normal {
        Some(map) => Some(json::material::NormalTexture {
            index: {
                push_image(
                    map.width(),
                    map.height(),
                    &map.texels()
                        .iter()
                        .flat_map(|c| c.to_linear())
                        .collect::<Vec<u8>>(),
                    map.wrap,
                    json_gltf,
                    bin,
                )?
            }
            .index,
            scale: 1.0,
            tex_coord: 0,
            extensions: None,
            extras: Default::default(),
        }),
        None => None,
    };

    let mut extensions = json::extensions::material::Material::default();

    if material.transparency > 0.0 {
        extensions.transmission = Some(json::extensions::material::Transmission {
            transmission_factor: json::extensions::material::TransmissionFactor(
                material.transparency,
            ),
            transmission_texture: None,
            extras: Default::default(),
        });
    }

    extensions.ior = Some(json::extensions::material::Ior {
        ior: json::extensions::material::IndexOfRefraction(material.refractive_index),
        extras: Default::default(),
    });

    let (emissive_factor, emissive_texture, strength) =
        encode_emissive(&material.emissive, json_gltf, bin)?;

    if strength > 1.0 {
        extensions.emissive_strength = Some(json::extensions::material::EmissiveStrength {
            emissive_strength: json::extensions::material::EmissiveStrengthFactor(strength),
        });
    }

    let index = json_gltf.push(json::Material {
        pbr_metallic_roughness: json::material::PbrMetallicRoughness {
            base_color_factor: json::material::PbrBaseColorFactor(base_color_factor),
            base_color_texture,
            metallic_factor: json::material::StrengthFactor(metallic_factor),
            roughness_factor: json::material::StrengthFactor(roughness_factor),
            metallic_roughness_texture,
            extensions: None,
            extras: Default::default(),
        },
        normal_texture,
        emissive_factor: json::material::EmissiveFactor(emissive_factor),
        emissive_texture,
        occlusion_texture: None,
        alpha_mode: Valid(json::material::AlphaMode::Opaque),
        alpha_cutoff: None,
        // O ray tracer acerta a face de trás.
        double_sided: true,
        name: None,
        extensions: Some(extensions),
        extras: Default::default(),
    });

    materials.push((material.clone(), index));

    Ok(index)
}

fn encode_metallic_roughness(
    material: &Material,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<(f32, f32, Option<json::texture::Info>), Error> {
    let (metallic, roughness) = (&material.metallic, &material.roughness);

    if let (scene::Channel::Constant(m), scene::Channel::Constant(r)) = (metallic, roughness) {
        return Ok((*m, *r, None));
    }

    let (width, height, wrap) = match (metallic, roughness) {
        (scene::Channel::Map(m), scene::Channel::Map(r)) => {
            if (m.width(), m.height()) != (r.width(), r.height()) {
                return Err(format!(
                    "metallic é {}x{} e roughness é {}x{}: os dois viram uma textura só no \
                    glTF, então precisam do mesmo tamanho",
                    m.width(),
                    m.height(),
                    r.width(),
                    r.height()
                )
                .into());
            }
            (m.width(), m.height(), m.wrap)
        }

        (scene::Channel::Map(t), _) | (_, scene::Channel::Map(t)) => {
            (t.width(), t.height(), t.wrap)
        }

        _ => unreachable!(),
    };

    let info = push_image(
        width,
        height,
        &(0..width as usize * height as usize)
            .flat_map(|i| {
                let g = match roughness {
                    scene::Channel::Constant(_) => 1.0,
                    scene::Channel::Map(map) => map.texels()[i].r,
                };

                let b = match metallic {
                    scene::Channel::Constant(_) => 1.0,
                    scene::Channel::Map(map) => map.texels()[i].r,
                };

                Rgb::new(0.0, g, b).to_linear()
            })
            .collect::<Vec<u8>>(),
        wrap,
        json_gltf,
        bin,
    )?;

    let metallic_factor = match metallic {
        scene::Channel::Constant(v) => *v,
        scene::Channel::Map(_) => 1.0,
    };

    let roughness_factor = match roughness {
        scene::Channel::Constant(v) => *v,
        scene::Channel::Map(_) => 1.0,
    };

    Ok((metallic_factor, roughness_factor, Some(info)))
}

fn encode_emissive(
    emissive: &scene::Channel<Rgb>,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<([f32; 3], Option<json::texture::Info>, f32), Error> {
    match emissive {
        scene::Channel::Constant(color) => {
            let strength = color.r.max(color.g).max(color.b).max(1.0);
            Ok((
                [color.r / strength, color.g / strength, color.b / strength],
                None,
                strength,
            ))
        }

        scene::Channel::Map(map) => {
            let strength = map
                .texels()
                .iter()
                .map(|c| c.r.max(c.g).max(c.b))
                .fold(1.0f32, f32::max);

            let normalized = Texture::new(
                map.width(),
                map.height(),
                map.texels().iter().map(|c| *c / strength).collect(),
            )?;

            Ok((
                [1.0; 3],
                Some({
                    push_image(
                        normalized.width(),
                        normalized.height(),
                        &normalized
                            .texels()
                            .iter()
                            .flat_map(|c| c.to_srgb())
                            .collect::<Vec<u8>>(),
                        map.wrap,
                        json_gltf,
                        bin,
                    )?
                }),
                strength,
            ))
        }
    }
}

fn push_image(
    width: u32,
    height: u32,
    bytes: &[u8],
    wrap: TextureWrap,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<json::texture::Info, Error> {
    let mut png = Vec::new();

    PngEncoder::new_with_quality(&mut png, CompressionType::Default, FilterType::Adaptive)
        .write_image(bytes, width, height, ExtendedColorType::Rgb8)?;

    let view = push_buffer_view(&png, None, json_gltf, bin);

    let image_gltf = json_gltf.push(json::Image {
        buffer_view: Some(view),
        mime_type: Some(json::image::MimeType("image/png".into())),
        uri: None,
        name: None,
        extensions: None,
        extras: Default::default(),
    });

    let texture_gltf = json_gltf.push(json::Texture {
        sampler: Some(json::Index::new(match wrap {
            TextureWrap::Repeat => 0,
            TextureWrap::Clamp => 1,
        })),
        source: image_gltf,
        name: None,
        extensions: None,
        extras: Default::default(),
    });

    Ok(json::texture::Info {
        index: texture_gltf,
        tex_coord: 0,
        extensions: None,
        extras: Default::default(),
    })
}

fn flatten<const N: usize>(data: &[[f32; N]]) -> Vec<u8> {
    data.iter()
        .flat_map(|row| row.iter().flat_map(|v| v.to_le_bytes()))
        .collect()
}

#[rustfmt::skip]
fn quaternion(euler: Vec3) -> [f32; 4] {
    let (sx, cx) = (euler.x.to_radians() * 0.5).sin_cos();
    let (sy, cy) = (euler.y.to_radians() * 0.5).sin_cos();
    let (sz, cz) = (euler.z.to_radians() * 0.5).sin_cos();

    [
        sx * cy * cz - cx * sy * sz,
        cx * sy * cz + sx * cy * sz,
        cx * cy * sz - sx * sy * cz,
        cx * cy * cz + sx * sy * sz,
    ]
}
