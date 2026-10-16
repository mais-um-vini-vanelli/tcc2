use godot::classes::ArrayMesh;
use godot::classes::mesh::{ArrayType, PrimitiveType};
use godot::prelude::*;

use tcc2::prelude::{Geometry, Mesh as Malha, Vec3};

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

/// A face `i` da tesselação vira o triângulo `i` da superfície, pra que o índice
/// que o editor escolhe seja o que o `on_triangle` recebe. Os atributos saem de
/// `Malha::face`, o mesmo que o rasterizador chama, então normal e tangente
/// faltando são preenchidas aqui igual lá.
///
/// A face da frente no Godot é o contrário da nossa, então cada face sai como
/// v0, v2, v1 — a troca fica dentro da face, ou a numeração escorrega.
fn surface(malha: &Malha) -> Gd<ArrayMesh> {
    let mut vertices = PackedVector3Array::new();
    let mut normals = PackedVector3Array::new();
    let mut tangents = PackedFloat32Array::new();
    let mut uvs = PackedVector2Array::new();

    for i in 0..malha.face_count() {
        let Some(face) = malha.face(i) else {
            continue;
        };

        for v in [face[0], face[2], face[1]] {
            vertices.push(Vector3::new(v.position.x, v.position.y, v.position.z));
            normals.push(Vector3::new(v.normal.x, v.normal.y, v.normal.z));
            tangents.push(v.tangent.vector.x);
            tangents.push(v.tangent.vector.y);
            tangents.push(v.tangent.vector.z);
            tangents.push(v.tangent.handedness);
            uvs.push(Vector2::new(v.uv.x, v.uv.y));
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
