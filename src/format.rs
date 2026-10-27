use std::collections::HashMap;
use std::sync::Arc;

use serde::Deserialize;

use crate::color::Rgb;
use crate::geometry::{Geometry, Mesh};
use crate::math::{Tangent, Vec2, Vec3};
use crate::parameters::Parameters;
use crate::scene::{Camera, Channel, Light, Material, Object, ObjectKind, Primitive, Scene};
use crate::texture::Texture;
use crate::transform::Transform;

pub const MAGIC: u32 = 0x3243_4354;
pub const VERSION: u32 = 1;

pub type Error = Box<dyn std::error::Error>;

#[derive(Default, Deserialize)]
#[serde(default)]
struct Header<'a> {
    roots: Vec<NodeJson>,
    meshes: Vec<MeshJson>,
    images: Vec<SliceJson>,
    parameters: Parameters<'a>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct NodeJson {
    transform: TransformJson,
    camera: Option<CameraJson>,
    light: Option<LightJson>,
    geometry: Option<GeometryJson>,
    material: Option<MaterialJson>,
    children: Vec<NodeJson>,
}

#[derive(Deserialize)]
#[serde(default)]
struct TransformJson {
    t: [f32; 3],
    r: [f32; 3],
    s: [f32; 3],
}

impl Default for TransformJson {
    fn default() -> Self {
        TransformJson {
            t: [0.0; 3],
            r: [0.0; 3],
            s: [1.0; 3],
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CameraJson {
    Perspective {
        fov: Option<f32>,
        near: Option<f32>,
        far: Option<f32>,
    },
    Orthographic {
        height: Option<f32>,
        near: Option<f32>,
        far: Option<f32>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LightJson {
    Point {
        color: Option<[f32; 3]>,
        intensity: Option<f32>,
    },
    Directional {
        color: Option<[f32; 3]>,
        intensity: Option<f32>,
    },
    Environment {
        color: Option<[f32; 3]>,
        intensity: Option<f32>,
    },
    Spot {
        color: Option<[f32; 3]>,
        intensity: Option<f32>,
        inner_angle: Option<f32>,
        outer_angle: Option<f32>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum GeometryJson {
    Sphere { radius: Option<f32> },
    Cube { half_edge: Option<f32> },
    Triangle { points: [[f32; 3]; 3] },
    Mesh(usize),
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct MaterialJson {
    albedo: Option<ChannelJson<[f32; 3]>>,
    metallic: Option<ChannelJson<f32>>,
    roughness: Option<ChannelJson<f32>>,
    emissive: Option<ChannelJson<[f32; 3]>>,
    emissive_strength: Option<f32>,
    normal: Option<NormalJson>,
    transparency: Option<f32>,
    refractive_index: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChannelJson<T> {
    Constant(T),
    Map(usize),
}

#[derive(Deserialize)]
struct NormalJson {
    map: usize,
    scale: Option<f32>,
}

#[derive(Deserialize)]
struct MeshJson {
    vertices: SliceJson,
    indices: SliceJson,
    normals: Option<SliceJson>,
    tangents: Option<SliceJson>,
    uvs: Option<SliceJson>,
}

#[derive(Clone, Copy, Deserialize)]
struct SliceJson {
    offset: usize,
    length: usize,
}

pub struct Reader<'a> {
    meshes_json: &'a [MeshJson],
    images_json: &'a [SliceJson],
    blob: &'a [u8],
    meshes: HashMap<usize, Arc<Mesh>>,
    textures: HashMap<(usize, bool), Arc<Texture>>,
}

impl Reader<'_> {
    pub fn read(bytes: &[u8]) -> Result<(Scene, Parameters<'_>), Error> {
        if bytes.len() < 12 {
            return Err("arquivo menor que o cabeçalho".into());
        }

        let (magic, version, length) = (
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize,
        );

        let end = 12 + length;

        if magic != MAGIC {
            return Err("não é um arquivo .tcc2".into());
        } else if version != VERSION {
            return Err(format!("arquivo na versão {version}, esperado {VERSION}").into());
        } else if end > bytes.len() {
            return Err("cabeçalho corrompido".into());
        }

        let Header {
            roots,
            meshes: meshes_json,
            images: images_json,
            parameters,
        } = serde_json::from_slice(&bytes[12..end])?;

        let mut reader = Reader {
            meshes_json: &meshes_json,
            images_json: &images_json,
            blob: &bytes[end..],
            meshes: HashMap::new(),
            textures: HashMap::new(),
        };

        let mut objects = Vec::with_capacity(roots.len());

        for root in &roots {
            objects.push(reader.object(root)?);
        }

        Ok((Scene { objects }, parameters))
    }

    fn object(&mut self, node: &NodeJson) -> Result<Object, Error> {
        let (t, r, s) = (node.transform.t, node.transform.r, node.transform.s);

        let mut object = Object {
            transform: Transform {
                translation: Vec3::new(t[0], t[1], t[2]),
                rotation: Vec3::new(r[0], r[1], r[2]),
                scale: Vec3::new(s[0], s[1], s[2]),
            },
            kind: self.kind(node)?,
            children: Vec::new(),
        };

        for child in &node.children {
            object.children.push(self.object(child)?);
        }

        Ok(object)
    }

    fn kind(&mut self, node: &NodeJson) -> Result<Option<ObjectKind>, Error> {
        if let Some(camera) = &node.camera {
            return Ok(Some(ObjectKind::Camera(match camera {
                CameraJson::Perspective { fov, near, far } => Camera::Perspective {
                    fov: fov.unwrap_or(60.0),
                    near: near.unwrap_or(0.1),
                    far: far.unwrap_or(100.0),
                },
                CameraJson::Orthographic { height, near, far } => Camera::Orthographic {
                    height: height.unwrap_or(1.0),
                    near: near.unwrap_or(0.1),
                    far: far.unwrap_or(100.0),
                },
            })));
        }

        if let Some(light) = &node.light {
            let glow = |color: &Option<[f32; 3]>, intensity: &Option<f32>| {
                let channels = color.unwrap_or([1.0; 3]);

                (
                    Rgb::new(channels[0], channels[1], channels[2]),
                    intensity.unwrap_or(1.0),
                )
            };

            return Ok(Some(ObjectKind::Light(match light {
                LightJson::Point { color, intensity } => {
                    let (color, intensity) = glow(color, intensity);

                    Light::Point { color, intensity }
                }
                LightJson::Directional { color, intensity } => {
                    let (color, intensity) = glow(color, intensity);

                    Light::Directional { color, intensity }
                }
                LightJson::Environment { color, intensity } => {
                    let (color, intensity) = glow(color, intensity);

                    Light::Environment { color, intensity }
                }
                LightJson::Spot {
                    color,
                    intensity,
                    inner_angle,
                    outer_angle,
                } => {
                    let (color, intensity) = glow(color, intensity);

                    Light::Spot {
                        inner_angle: inner_angle.unwrap_or(20.0),
                        outer_angle: outer_angle.unwrap_or(30.0),
                        color,
                        intensity,
                    }
                }
            })));
        }

        if let Some(geometry) = &node.geometry {
            let geometry = self.geometry(geometry)?;

            return Ok(Some(ObjectKind::Primitive(Primitive {
                geometry,
                material: self.material(node.material.as_ref())?,
            })));
        }

        Ok(None)
    }

    fn geometry(&mut self, json: &GeometryJson) -> Result<Geometry, Error> {
        Ok(match json {
            GeometryJson::Sphere { radius } => Geometry::sphere(radius.unwrap_or(0.5)),
            GeometryJson::Cube { half_edge } => Geometry::cube(half_edge.unwrap_or(0.5)),
            GeometryJson::Triangle { points } => {
                Geometry::triangle(points.map(|point| Vec3::new(point[0], point[1], point[2])))
            }
            GeometryJson::Mesh(index) => Geometry::Mesh(self.mesh(*index)?),
        })
    }

    fn mesh(&mut self, index: usize) -> Result<Arc<Mesh>, Error> {
        if let Some(cached) = self.meshes.get(&index) {
            return Ok(cached.clone());
        }

        let (meshes_json, blob) = (self.meshes_json, self.blob);
        let json = meshes_json.get(index).ok_or("malha fora da tabela")?;
        let float = |bytes: &[u8]| f32::from_le_bytes(bytes.try_into().unwrap());

        let mut mesh = Mesh::new(
            Self::slice(blob, json.vertices)?
                .chunks_exact(12)
                .map(|v| Vec3::new(float(&v[0..4]), float(&v[4..8]), float(&v[8..12])))
                .collect(),
            Self::slice(blob, json.indices)?
                .chunks_exact(4)
                .map(|i| u32::from_le_bytes(i.try_into().unwrap()))
                .collect(),
        );

        if let Some(normals) = json.normals {
            mesh.normals = Some(
                Self::slice(blob, normals)?
                    .chunks_exact(12)
                    .map(|n| Vec3::new(float(&n[0..4]), float(&n[4..8]), float(&n[8..12])))
                    .collect(),
            );
        }

        if let Some(tangents) = json.tangents {
            mesh.tangents = Some(
                Self::slice(blob, tangents)?
                    .chunks_exact(16)
                    .map(|t| {
                        Tangent::new(
                            Vec3::new(float(&t[0..4]), float(&t[4..8]), float(&t[8..12])),
                            float(&t[12..16]),
                        )
                    })
                    .collect(),
            );
        }

        if let Some(uvs) = json.uvs {
            mesh.uvs = Some(
                Self::slice(blob, uvs)?
                    .chunks_exact(8)
                    .map(|uv| Vec2::new(float(&uv[0..4]), float(&uv[4..8])))
                    .collect(),
            );
        }

        let mesh = Arc::new(mesh);
        self.meshes.insert(index, mesh.clone());

        Ok(mesh)
    }

    fn material(&mut self, json: Option<&MaterialJson>) -> Result<Material, Error> {
        let default = Material::default();

        let Some(json) = json else {
            return Ok(default);
        };

        let normal = match &json.normal {
            Some(normal) => Some((
                self.texture(normal.map, false)?,
                normal.scale.unwrap_or(default.normal_scale),
            )),
            None => None,
        };

        Ok(Material {
            albedo: self.rgb_channel(json.albedo.as_ref(), Rgb::gray(0.8))?,
            metallic: self.scalar_channel(json.metallic.as_ref(), 0.0)?,
            roughness: self.scalar_channel(json.roughness.as_ref(), 0.5)?,
            emissive: self.rgb_channel(json.emissive.as_ref(), Rgb::BLACK)?,
            emissive_strength: json.emissive_strength.unwrap_or(default.emissive_strength),
            normal: normal.as_ref().map(|(texture, _)| texture.clone()),
            normal_scale: normal.map_or(default.normal_scale, |(_, scale)| scale),
            transparency: json.transparency.unwrap_or(default.transparency),
            refractive_index: json.refractive_index.unwrap_or(default.refractive_index),
        })
    }

    fn rgb_channel(
        &mut self,
        json: Option<&ChannelJson<[f32; 3]>>,
        fallback: Rgb,
    ) -> Result<Channel<Rgb>, Error> {
        Ok(match json {
            Some(ChannelJson::Map(index)) => Channel::Map(self.texture(*index, true)?),
            Some(ChannelJson::Constant(channels)) => {
                Channel::Constant(Rgb::new(channels[0], channels[1], channels[2]))
            }
            None => Channel::Constant(fallback),
        })
    }

    fn scalar_channel(
        &mut self,
        json: Option<&ChannelJson<f32>>,
        fallback: f32,
    ) -> Result<Channel<f32>, Error> {
        Ok(match json {
            Some(ChannelJson::Map(index)) => Channel::Map(self.texture(*index, false)?),
            Some(ChannelJson::Constant(constant)) => Channel::Constant(*constant),
            None => Channel::Constant(fallback),
        })
    }

    fn texture(&mut self, index: usize, srgb: bool) -> Result<Arc<Texture>, Error> {
        if let Some(cached) = self.textures.get(&(index, srgb)) {
            return Ok(cached.clone());
        }

        let (images_json, blob) = (self.images_json, self.blob);
        let slice = *images_json.get(index).ok_or("imagem fora da tabela")?;

        let decoded = image::load_from_memory_with_format(
            Self::slice(blob, slice)?,
            image::ImageFormat::Png,
        )?
        .to_rgb8();

        let (width, height) = decoded.dimensions();

        let texels = decoded
            .pixels()
            .map(|texel| {
                if srgb {
                    Rgb::from_srgb(texel.0)
                } else {
                    Rgb::from_linear(texel.0)
                }
            })
            .collect();

        let texture = Arc::new(Texture::new(width, height, texels)?);
        self.textures.insert((index, srgb), texture.clone());

        Ok(texture)
    }

    fn slice(blob: &[u8], slice: SliceJson) -> Result<&[u8], Error> {
        let end = slice
            .offset
            .checked_add(slice.length)
            .ok_or("fatia inválida")?;

        blob.get(slice.offset..end)
            .ok_or("fatia fora do blob".into())
    }
}
