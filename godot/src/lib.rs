use godot::classes::ArrayMesh;
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::prelude::*;

use tcc2::prelude::{Geometry, Mesh as Tcc2Mesh, Rgb, Vec3};

struct Tcc2Godot;

#[gdextension]
unsafe impl ExtensionLibrary for Tcc2Godot {}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Tessellate;

#[godot_api]
impl Tcc2Tessellate {
    #[func]
    fn sphere(radius: f32) -> Gd<ArrayMesh> {
        surface(&Geometry::sphere(radius).tessellate())
    }

    #[func]
    fn cube(half_edge: f32) -> Gd<ArrayMesh> {
        surface(&Geometry::cube(half_edge).tessellate())
    }

    #[func]
    fn triangle(a: Vector3, b: Vector3, c: Vector3) -> Gd<ArrayMesh> {
        let points = [a, b, c].map(|v| Vec3::new(v.x, v.y, v.z));
        surface(&Geometry::triangle(points).tessellate())
    }
}

fn surface(mesh: &Tcc2Mesh) -> Gd<ArrayMesh> {
    let mut vertices = PackedVector3Array::new();
    let mut normals = PackedVector3Array::new();
    let mut tangents = PackedFloat32Array::new();
    let mut uvs = PackedVector2Array::new();

    for i in 0..mesh.face_count() {
        let Some(face) = mesh.face(i) else {
            continue;
        };

        for vertex in [face[0], face[2], face[1]] {
            vertices.push(Vector3::new(
                vertex.position.x,
                vertex.position.y,
                vertex.position.z,
            ));
            normals.push(Vector3::new(
                vertex.normal.x,
                vertex.normal.y,
                vertex.normal.z,
            ));
            tangents.push(vertex.tangent.direction.x);
            tangents.push(vertex.tangent.direction.y);
            tangents.push(vertex.tangent.direction.z);
            tangents.push(vertex.tangent.handedness);
            uvs.push(Vector2::new(vertex.uv.x, vertex.uv.y));
        }
    }

    let mut arrays = VarArray::new();
    arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
    arrays.set(ArrayType::VERTEX.ord() as usize, &vertices.to_variant());
    arrays.set(ArrayType::NORMAL.ord() as usize, &normals.to_variant());
    arrays.set(ArrayType::TANGENT.ord() as usize, &tangents.to_variant());
    arrays.set(ArrayType::TEX_UV.ord() as usize, &uvs.to_variant());

    let mut mesh = ArrayMesh::new_gd();
    mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrays);
    mesh
}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct Tcc2Bake;

#[godot_api]
impl Tcc2Bake {
    #[func]
    fn apply_rgb_factor(data: PackedByteArray, factor: Color) -> PackedByteArray {
        let source = data.as_slice();
        let factor = Rgb::new(factor.r, factor.g, factor.b);

        let mut result = Vec::with_capacity(source.len());

        for texel in source.chunks_exact(4) {
            let color = Rgb::from_srgb([texel[0], texel[1], texel[2]]) * factor;

            result.extend_from_slice(&color.to_srgb());
            result.push(texel[3]);
        }

        PackedByteArray::from(result.as_slice())
    }

    #[func]
    fn apply_scalar_factor(data: PackedByteArray, channel: i64, factor: f32) -> PackedByteArray {
        let source = data.as_slice();
        let mut result = Vec::with_capacity(source.len() / 4);

        for texel in source.chunks_exact(4) {
            let level = match channel {
                0..=3 => texel[channel as usize] as u32,
                _ => (texel[0] as u32 + texel[1] as u32 + texel[2] as u32) / 3,
            };

            result.push(((level as f32 / 255.0 * factor).clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }

        PackedByteArray::from(result.as_slice())
    }
}
