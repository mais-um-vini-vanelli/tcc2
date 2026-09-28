use std::borrow::Cow;
use std::collections::BTreeMap;

use gltf::json::{self, validation::Checked::Valid, validation::USize64};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use serde_json::{Map, Value, json};

use crate::color::Rgb;
use crate::math::{Aabb, Vec3};
use crate::parameters::{Parameters, TransformMethod};
use crate::scene::{self, Camera, Content, Light, Material, Mesh, Object, Primitive, Scene};
use crate::texture::Texture;
use crate::transform::{Chain, Transform};

use super::{EXTENSION, Error, VERSION};

pub fn encode(scene: &Scene, parameters: Option<&Parameters>) -> Result<Vec<u8>, Error> {
    let mut json_gltf = json::Root::default();

    let mut bin = Vec::new();
    let mut materiais = Vec::new();

    json_gltf.asset.generator = Some(format!("tcc2 {}", env!("CARGO_PKG_VERSION")));

    json_gltf.scene = Some(json_gltf.push(json::Scene {
        extensions: None,
        extras: Default::default(),
        name: None,
        nodes: Vec::new(),
    }));

    json_gltf.push(json::texture::Sampler {
        mag_filter: Some(Valid(json::texture::MagFilter::Linear)),
        min_filter: Some(Valid(json::texture::MinFilter::LinearMipmapLinear)),
        wrap_s: Valid(json::texture::WrappingMode::Repeat),
        wrap_t: Valid(json::texture::WrappingMode::Repeat),
        name: None,
        extensions: None,
        extras: Default::default(),
    });

    for objeto in &scene.objects {
        let indice = encode_object(objeto, &mut json_gltf, &mut bin, &mut materiais)?;
        json_gltf.scenes[0].nodes.push(indice);
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
            ..
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
            ..
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

    let (direita, cima, frente) = camera.basis();

    json::Node {
        camera: Some(camera_gltf),
        // A câmera do glTF olha para o −z local, com +y para cima.
        matrix: Some(matrix(direita, cima, -frente, camera.position())),
        ..Default::default()
    }
}

fn encode_light(luz: Light, json_gltf: &mut json::Root) -> json::Node {
    use json::extensions::scene::khr_lights_punctual as khr;

    if let Light::Ambient { color, intensity } = luz {
        // A ambiente não tem equivalente no glTF: ela vai para uma extension.
        return json::Node {
            extensions: Some(json::extensions::scene::Node {
                others: Map::from_iter([(
                    EXTENSION.into(),
                    json!({ "ambient": { "color": [color.r, color.g, color.b], "intensity": intensity } }),
                )]),
                ..Default::default()
            }),
            ..Default::default()
        };
    }

    let punctual = |tipo, spot| khr::Light {
        type_: Valid(tipo),
        color: [luz.color().r, luz.color().g, luz.color().b],
        intensity: luz.intensity(),
        range: None,
        spot,
        extensions: None,
        extras: Default::default(),
        name: None,
    };

    let (light, posicao, direcao) = match luz {
        Light::Point { position, .. } => (
            punctual(khr::Type::Point, None),
            position,
            Vec3::new(0.0, 0.0, -1.0),
        ),

        Light::Directional { direction, .. } => (
            punctual(khr::Type::Directional, None),
            Vec3::ZERO,
            direction,
        ),

        Light::Spot {
            position,
            direction,
            inner_angle,
            outer_angle,
            ..
        } => (
            punctual(
                khr::Type::Spot,
                Some(khr::Spot {
                    inner_cone_angle: inner_angle.to_radians(),
                    outer_cone_angle: outer_angle.to_radians(),
                }),
            ),
            position,
            direction,
        ),

        Light::Ambient { .. } => unreachable!(),
    };

    let light_gltf = json_gltf.push(light);

    let (direita, cima, frente) = {
        // Direcional e spot apontam para o −z local.
        let z = -direcao.normalized();

        let referencia = if z.y.abs() < 0.99 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        let x = referencia.cross(z).normalized();
        (x, z.cross(x), z)
    };

    json::Node {
        matrix: Some(matrix(direita, cima, frente, posicao)),
        extensions: Some(json::extensions::scene::Node {
            khr_lights_punctual: Some(khr::KhrLightsPunctual { light: light_gltf }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn encode_object(
    objeto: &Object,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
    materiais: &mut Vec<(Material, json::Index<json::Material>)>,
) -> Result<json::Index<json::Node>, Error> {
    let filhos = objeto
        .children
        .iter()
        .map(|filho| encode_object(filho, json_gltf, bin, materiais))
        .collect::<Result<Vec<_>, Error>>()?;

    let filhos = (!filhos.is_empty()).then_some(filhos);

    let transform = Chain::from_root_first(&[objeto.transform]);

    let mut no = match &objeto.content {
        None => json::Node::default(),

        Some(Content::Camera(camera)) => encode_camera(
            camera.transformed(&transform, TransformMethod::Stepwise),
            json_gltf,
        ),

        Some(Content::Light(luz @ Light::Ambient { .. })) => encode_light(*luz, json_gltf),

        Some(Content::Light(luz)) => encode_light(
            luz.transformed(&transform, TransformMethod::Stepwise),
            json_gltf,
        ),

        Some(Content::Geometry {
            geometry,
            material,
            casts_shadows,
        }) => {
            let material_gltf = encode_material(material, json_gltf, bin, materiais)?;

            let tesselada = geometry.tessellate();

            if tesselada.vertices.is_empty() || tesselada.indices.is_empty() {
                return Err("objeto sem nenhum triângulo".into());
            }

            let malha_gltf = encode_mesh(&tesselada, material_gltf, json_gltf, bin);

            // Esfera e cubo vão com a malha tesselada **e** com os campos exatos: o
            // decoder daqui lê os campos e ignora a malha, e quem não conhece a
            // extensão fica com a malha.
            let primitivo = match geometry {
                Primitive::Mesh(_) => None,
                Primitive::Triangle { .. } => Some(json!("triangle")),
                Primitive::Sphere { center, radius } => Some(json!({
                    "sphere": { "center": json!([center.x, center.y, center.z]), "radius": radius }
                })),
                Primitive::Cube { center, half_edge } => Some(json!({
                    "cube": { "center": json!([center.x, center.y, center.z]), "half_edge": half_edge }
                })),
            };

            json_gltf.meshes[malha_gltf.value()].extensions =
                primitivo.map(|primitivo| json::extensions::mesh::Mesh {
                    others: Map::from_iter([(EXTENSION.into(), json!({ "primitive": primitivo }))]),
                });

            json::Node {
                mesh: Some(malha_gltf),
                extensions: (!casts_shadows).then(|| json::extensions::scene::Node {
                    others: Map::from_iter([(EXTENSION.into(), json!({ "casts_shadows": false }))]),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }
    };

    if no.matrix.is_none() {
        let Transform {
            translation,
            rotation,
            scale,
        } = objeto.transform;

        no.translation = Some([translation.x, translation.y, translation.z]);
        no.rotation = Some(json::scene::UnitQuaternion(quaternion(rotation)));
        no.scale = Some([scale.x, scale.y, scale.z]);
    }

    no.children = filhos;

    Ok(json_gltf.push(no))
}

fn encode_mesh(
    malha: &Mesh,
    material_gltf: json::Index<json::Material>,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> json::Index<json::Mesh> {
    use json::accessor::{ComponentType, Type};

    let mut atributos = BTreeMap::new();

    let posicoes: Vec<[f32; 3]> = malha.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
    let posicoes = push_acessor(
        flatten(&posicoes),
        posicoes.len(),
        Type::Vec3,
        ComponentType::F32,
        json_gltf,
        bin,
    );

    let Aabb { min, max } = Aabb::around(malha.vertices.iter().copied());
    json_gltf.accessors[posicoes.value()].min = Some(json!([min.x, min.y, min.z]));
    json_gltf.accessors[posicoes.value()].max = Some(json!([max.x, max.y, max.z]));

    atributos.insert(Valid(json::mesh::Semantic::Positions), posicoes);

    if let Some(normais) = &malha.normals {
        let dados: Vec<[f32; 3]> = normais.iter().map(|n| [n.x, n.y, n.z]).collect();
        let normais = push_acessor(
            flatten(&dados),
            dados.len(),
            Type::Vec3,
            ComponentType::F32,
            json_gltf,
            bin,
        );
        atributos.insert(Valid(json::mesh::Semantic::Normals), normais);
    }

    // O TANGENT do glTF é vec4: xyz é a tangente, w é o lado da bitangente.
    if let Some(tangentes) = &malha.tangents {
        let dados: Vec<[f32; 4]> = tangentes
            .iter()
            .map(|t| [t.vector.x, t.vector.y, t.vector.z, t.handedness])
            .collect();

        let tangentes = push_acessor(
            flatten(&dados),
            dados.len(),
            Type::Vec4,
            ComponentType::F32,
            json_gltf,
            bin,
        );

        atributos.insert(Valid(json::mesh::Semantic::Tangents), tangentes);
    }

    if let Some(uvs) = &malha.uvs {
        let dados: Vec<[f32; 2]> = uvs.iter().map(|uv| [uv.x, uv.y]).collect();

        let uvs = push_acessor(
            flatten(&dados),
            dados.len(),
            Type::Vec2,
            ComponentType::F32,
            json_gltf,
            bin,
        );

        atributos.insert(Valid(json::mesh::Semantic::TexCoords(0)), uvs);
    }

    let bytes: Vec<u8> = malha.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let indices = push_acessor(
        bytes,
        malha.indices.len(),
        Type::Scalar,
        ComponentType::U32,
        json_gltf,
        bin,
    );

    let primitive = json::mesh::Primitive {
        attributes: atributos,
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

fn push_acessor(
    bytes: Vec<u8>,
    count: usize,
    tipo: json::accessor::Type,
    componente: json::accessor::ComponentType,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> json::Index<json::accessor::Accessor> {
    let target = match componente {
        json::accessor::ComponentType::U32 => json::buffer::Target::ElementArrayBuffer,
        _ => json::buffer::Target::ArrayBuffer,
    };

    let view = push_buffer_view(&bytes, Some(target), json_gltf, bin);

    json_gltf.push(json::accessor::Accessor {
        buffer_view: Some(view),
        byte_offset: Some(USize64(0)),
        count: USize64::from(count),
        component_type: Valid(json::accessor::GenericComponentType(componente)),
        type_: Valid(tipo),
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
    materiais: &mut Vec<(Material, json::Index<json::Material>)>,
) -> Result<json::Index<json::Material>, Error> {
    if let Some((_, indice)) = materiais.iter().find(|(j, _)| j == material) {
        return Ok(*indice);
    }

    let (base_color_factor, base_color_texture) = match &material.albedo {
        scene::Value::Constant(cor) => ([cor.r, cor.g, cor.b, 1.0], None),
        scene::Value::Map(mapa) => (
            [1.0; 4],
            Some({
                push_image(
                    mapa.width(),
                    mapa.height(),
                    &mapa
                        .texels()
                        .iter()
                        .flat_map(|c| c.to_srgb())
                        .collect::<Vec<u8>>(),
                    json_gltf,
                    bin,
                )?
            }),
        ),
    };

    let (metallic_factor, roughness_factor, metallic_roughness_texture) =
        encode_metallic_roughness(material, json_gltf, bin)?;

    let normal_texture = match &material.normal {
        Some(mapa) => Some(json::material::NormalTexture {
            index: {
                push_image(
                    mapa.width(),
                    mapa.height(),
                    &mapa
                        .texels()
                        .iter()
                        .flat_map(|c| c.to_linear())
                        .collect::<Vec<u8>>(),
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

    let mut extensoes = json::extensions::material::Material::default();

    if material.transparency > 0.0 {
        extensoes.transmission = Some(json::extensions::material::Transmission {
            transmission_factor: json::extensions::material::TransmissionFactor(
                material.transparency,
            ),
            transmission_texture: None,
            extras: Default::default(),
        });
    }

    extensoes.ior = Some(json::extensions::material::Ior {
        ior: json::extensions::material::IndexOfRefraction(material.refractive_index),
        extras: Default::default(),
    });

    let (emissive_factor, emissive_texture, forca) =
        encode_emissive(&material.emissive, json_gltf, bin)?;

    if forca > 1.0 {
        extensoes.emissive_strength = Some(json::extensions::material::EmissiveStrength {
            emissive_strength: json::extensions::material::EmissiveStrengthFactor(forca),
        });
    }

    let indice = json_gltf.push(json::Material {
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
        extensions: Some(extensoes),
        extras: Default::default(),
    });

    materiais.push((material.clone(), indice));

    Ok(indice)
}

fn encode_metallic_roughness(
    material: &Material,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<(f32, f32, Option<json::texture::Info>), Error> {
    let (metallic, roughness) = (&material.metallic, &material.roughness);

    if let (scene::Value::Constant(m), scene::Value::Constant(r)) = (metallic, roughness) {
        return Ok((*m, *r, None));
    }

    let tamanho = match (metallic, roughness) {
        (scene::Value::Map(m), scene::Value::Map(r)) => {
            if (m.width(), m.height()) != (r.width(), r.height()) {
                return Err(format!(
                    "metallic é {}x{} e roughness é {}x{}: os dois viram uma textura só no                      glTF, então precisam do mesmo tamanho",
                    m.width(),
                    m.height(),
                    r.width(),
                    r.height()
                )
                .into());
            }
            (m.width(), m.height())
        }

        (scene::Value::Map(t), _) | (_, scene::Value::Map(t)) => (t.width(), t.height()),

        _ => unreachable!(),
    };

    let info = push_image(
        tamanho.0,
        tamanho.1,
        &(0..tamanho.0 as usize * tamanho.1 as usize)
            .flat_map(|i| {
                let g = match roughness {
                    scene::Value::Constant(_) => 1.0,
                    scene::Value::Map(mapa) => mapa.texels()[i].r,
                };

                let b = match metallic {
                    scene::Value::Constant(_) => 1.0,
                    scene::Value::Map(mapa) => mapa.texels()[i].r,
                };

                Rgb::new(0.0, g, b).to_linear()
            })
            .collect::<Vec<u8>>(),
        json_gltf,
        bin,
    )?;

    let metallic_factor = match metallic {
        scene::Value::Constant(v) => *v,
        scene::Value::Map(_) => 1.0,
    };

    let roughness_factor = match roughness {
        scene::Value::Constant(v) => *v,
        scene::Value::Map(_) => 1.0,
    };

    Ok((metallic_factor, roughness_factor, Some(info)))
}

fn encode_emissive(
    emissive: &scene::Value<Rgb>,
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<([f32; 3], Option<json::texture::Info>, f32), Error> {
    match emissive {
        scene::Value::Constant(cor) => {
            let forca = cor.r.max(cor.g).max(cor.b).max(1.0);
            Ok(([cor.r / forca, cor.g / forca, cor.b / forca], None, forca))
        }

        scene::Value::Map(mapa) => {
            let forca = mapa
                .texels()
                .iter()
                .map(|c| c.r.max(c.g).max(c.b))
                .fold(1.0f32, f32::max);

            let normalizada = Texture::new(
                mapa.width(),
                mapa.height(),
                mapa.texels().iter().map(|c| *c / forca).collect(),
            )?;

            Ok((
                [1.0; 3],
                Some({
                    push_image(
                        normalizada.width(),
                        normalizada.height(),
                        &normalizada
                            .texels()
                            .iter()
                            .flat_map(|c| c.to_srgb())
                            .collect::<Vec<u8>>(),
                        json_gltf,
                        bin,
                    )?
                }),
                forca,
            ))
        }
    }
}

fn push_image(
    largura: u32,
    altura: u32,
    bytes: &[u8],
    json_gltf: &mut json::Root,
    bin: &mut Vec<u8>,
) -> Result<json::texture::Info, Error> {
    let mut png = Vec::new();

    PngEncoder::new_with_quality(&mut png, CompressionType::Default, FilterType::Adaptive)
        .write_image(bytes, largura, altura, ExtendedColorType::Rgb8)?;

    let view = push_buffer_view(&png, None, json_gltf, bin);

    let imagem_gltf = json_gltf.push(json::Image {
        buffer_view: Some(view),
        mime_type: Some(json::image::MimeType("image/png".into())),
        uri: None,
        name: None,
        extensions: None,
        extras: Default::default(),
    });

    let textura_gltf = json_gltf.push(json::Texture {
        sampler: Some(json::Index::new(0)),
        source: imagem_gltf,
        name: None,
        extensions: None,
        extras: Default::default(),
    });

    Ok(json::texture::Info {
        index: textura_gltf,
        tex_coord: 0,
        extensions: None,
        extras: Default::default(),
    })
}

fn flatten<const N: usize>(dados: &[[f32; N]]) -> Vec<u8> {
    dados
        .iter()
        .flat_map(|linha| linha.iter().flat_map(|v| v.to_le_bytes()))
        .collect()
}

#[rustfmt::skip]
fn matrix(x: Vec3, y: Vec3, z: Vec3, posicao: Vec3) -> [f32; 16] {
    [
        x.x      , x.y      , x.z      , 0.0,
        y.x      , y.y      , y.z      , 0.0,
        z.x      , z.y      , z.z      , 0.0,
        posicao.x, posicao.y, posicao.z, 1.0,
    ]
}

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
