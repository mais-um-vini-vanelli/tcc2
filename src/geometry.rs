use std::sync::Arc;

use crate::math::{Aabb, Baricentrics, Ray, Tangent, Vec2, Vec3, Vertex};

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    /// Índices em trios.
    pub indices: Vec<u32>,
    /// Normais por vértice. Se `None`, uma normal é gerada a partir da geometria.
    pub normals: Option<Vec<Vec3>>,
    /// Tangentes por vértice. Se `None`, uma tangente é gerada a partir da geometria.
    pub tangents: Option<Vec<Tangent>>,
    /// Coordenadas de textura por vértice.
    pub uvs: Option<Vec<Vec2>>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, indices: Vec<u32>) -> Self {
        Mesh {
            vertices,
            indices,
            normals: None,
            tangents: None,
            uvs: None,
        }
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn face(&self, index: usize) -> Option<[Vertex; 3]> {
        let indices = [
            *self.indices.get(index * 3)? as usize,
            *self.indices.get(index * 3 + 1)? as usize,
            *self.indices.get(index * 3 + 2)? as usize,
        ];

        fn attributes<T: Copy>(valores: &[T], indices: [usize; 3]) -> Option<[T; 3]> {
            Some([
                *valores.get(indices[0])?,
                *valores.get(indices[1])?,
                *valores.get(indices[2])?,
            ])
        }

        let vertices = attributes(&self.vertices, indices)?;

        let normals = self
            .normals
            .as_deref()
            .and_then(|n| attributes(n, indices))
            .unwrap_or_else(|| {
                let geometric_normal = (vertices[1] - vertices[0])
                    .cross(vertices[2] - vertices[0])
                    .normalized();

                [geometric_normal; 3]
            });

        let uvs = self
            .uvs
            .as_deref()
            .and_then(|u| attributes(u, indices))
            .unwrap_or([Vec2::ZERO; 3]);

        let tangentes = self
            .tangents
            .as_deref()
            .and_then(|t| attributes(t, indices))
            .unwrap_or_else(|| {
                let edge1 = vertices[1] - vertices[0];
                let d1 = uvs[1] - uvs[0];

                let edge2 = vertices[2] - vertices[0];
                let d2 = uvs[2] - uvs[0];

                let det = d1.x * d2.y - d2.x * d1.y;
                if det.abs() < 1e-12 {
                    // du e dv são paralelos; o triângulo de uvs é degenerado.
                    return normals.map(Tangent::any_perpendicular);
                }

                let (du, dv) = (
                    (edge1 * d2.y - edge2 * d1.y) / det,
                    (edge2 * d1.x - edge1 * d2.x) / det,
                );

                [0, 1, 2].map(|k| Tangent::from_uv_axes(normals[k], du, dv))
            });

        Some([0, 1, 2].map(|k| Vertex {
            position: vertices[k],
            normal: normals[k],
            tangent: tangentes[k],
            uv: uvs[k],
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit {
    pub t: f32,
    pub shading_normal: Vec3,
    pub geometric_normal: Vec3,
    pub tangent: Tangent,
    pub uv: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Triangle { points: [Vec3; 3] },
    Cube { half_edge: f32 },
    Sphere { radius: f32 },
    Mesh(Arc<Mesh>),
}

impl Geometry {
    pub fn triangle(points: [Vec3; 3]) -> Self {
        Geometry::Triangle { points }
    }

    pub fn mesh(vertices: Vec<Vec3>, indices: Vec<u32>) -> Self {
        Geometry::Mesh(Arc::new(Mesh::new(vertices, indices)))
    }

    pub fn sphere(radius: f32) -> Self {
        Geometry::Sphere { radius }
    }

    pub fn cube(half_edge: f32) -> Self {
        Geometry::Cube { half_edge }
    }

    pub fn tessellate(&self) -> Arc<Mesh> {
        match self {
            Geometry::Triangle { points } => Arc::new(Mesh {
                vertices: points.to_vec(),
                indices: vec![0, 1, 2],
                normals: None,
                tangents: None,
                uvs: Some(
                    [
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(0.0, 1.0),
                    ]
                    .to_vec(),
                ),
            }),

            Geometry::Cube { half_edge } => Arc::new(Self::tessellate_cube(Vec3::ZERO, *half_edge)),

            Geometry::Sphere { radius } => Arc::new(Self::tessellate_sphere(Vec3::ZERO, *radius)),

            Geometry::Mesh(mesh) => Arc::clone(mesh),
        }
    }

    fn tessellate_cube(center: Vec3, half_edge: f32) -> Mesh {
        let half_edge = half_edge.abs();

        let mut vertices = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut tangents = Vec::with_capacity(24);
        let mut uvs = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        // Para cada face: a normal, a direção pra direita e a direção pra baixo de quem
        // olha a face de fora.
        #[rustfmt::skip]
        let faces = [
            (Vec3::new( 1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0, -1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0,  1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0,  1.0)),
            (Vec3::new( 0.0,  0.0,  1.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new(-1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0,  1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0, -1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0, -1.0)),
            (Vec3::new( 0.0,  0.0, -1.0), Vec3::new(-1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        ];

        for (normal, right, down) in faces {
            let first = vertices.len() as u32;

            // A uv da face anda pra `direita` quando o u cresce e pra `baixo`
            // quando o v cresce.
            let tangent = Tangent::from_uv_axes(normal, right, down);

            for (corner, uv) in [
                // Os cantos, vistos de fora.
                (normal - right - down, Vec2::new(0.0, 0.0)),
                (normal + right - down, Vec2::new(1.0, 0.0)),
                (normal + right + down, Vec2::new(1.0, 1.0)),
                (normal - right + down, Vec2::new(0.0, 1.0)),
            ]
            .into_iter()
            {
                vertices.push(center + corner * half_edge);
                normals.push(normal);
                tangents.push(tangent);
                uvs.push(uv);
            }

            // As faces da frente são as anti-horárias visto de fora: 0→3→2 e 0→2→1.
            indices.extend([0, 3, 2, 0, 2, 1].map(|k| first + k));
        }

        Mesh {
            vertices,
            indices,
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
        }
    }

    fn tessellate_sphere(center: Vec3, radius: f32) -> Mesh {
        let parallels = 16;
        let meridians = parallels * 2;
        let per_line = meridians + 1;

        let vertex_count = (parallels + 1) as usize * per_line as usize;

        let mut vertices = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut tangents = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);

        for i in 0..=parallels {
            let u = i as f32 / parallels as f32;

            let (sin_theta, cos_theta) = (u * std::f32::consts::PI).sin_cos();

            for j in 0..=meridians {
                let v = j as f32 / meridians as f32;

                let (sin_phi, cos_phi) = (v * std::f32::consts::TAU).sin_cos();

                // Na esfera unitária centrada na origem, a posição do ponto e a
                // normal são o mesmo vetor.
                let r = sin_theta;
                let normal = Vec3::new(r * cos_phi, cos_theta, r * sin_phi);

                vertices.push(center + normal * radius);
                normals.push(normal);
                tangents.push(Self::sphere_tangent(normal));
                uvs.push(Vec2::new(v, u));
            }
        }

        let mut indices = Vec::with_capacity((parallels * meridians * 6) as usize);

        for i in 0..parallels {
            for j in 0..meridians {
                // Os 4 pontos do quadrado atual (paralelo P x meridiano M).
                let a = i * per_line + j;
                let b = a + 1;
                let c = a + per_line;
                let d = c + 1;

                // Nos quadrados dos polos um dos triângulos é degenerado (a = b, ou c = d) e é descartado.
                if i != 0 {
                    indices.extend_from_slice(&[a, b, c]);
                }

                if i != parallels - 1 {
                    indices.extend_from_slice(&[b, d, c]);
                }
            }
        }

        Mesh {
            vertices,
            indices,
            normals: Some(normals),
            tangents: Some(tangents),
            uvs: Some(uvs),
        }
    }

    /// Calcula a onde o raio intersecta com a geometria, em espaço local.
    pub fn intersect(&self, ray: Ray) -> Option<Hit> {
        match self {
            Geometry::Triangle { points } => {
                let (t, [_, u, v]) = Self::moller_trumbore(ray, *points)?;

                let edge1 = points[1] - points[0];
                let edge2 = points[2] - points[0];
                let normal = edge1.cross(edge2).normalized();

                Some(Hit {
                    t,
                    shading_normal: normal,
                    geometric_normal: normal,
                    tangent: Tangent::from_uv_axes(normal, edge1, edge2),
                    uv: Vec2::new(u, v),
                })
            }

            Geometry::Cube { half_edge } => Self::intersect_cube(ray, Vec3::ZERO, *half_edge),

            Geometry::Sphere { radius } => {
                let center = Vec3::ZERO;
                let t = Self::intersect_sphere(ray, center, *radius)?;

                let normal = (ray.at(t) - center).normalized();

                Some(Hit {
                    t,
                    shading_normal: normal,
                    geometric_normal: normal,
                    tangent: Self::sphere_tangent(normal),
                    uv: {
                        let theta = normal.y.acos() / std::f32::consts::PI;
                        let phi = normal.z.atan2(normal.x) / std::f32::consts::TAU;

                        // atan2 devolve -pi..pi, que vira -0.5..0.5 aqui.
                        let phi = if phi >= 0.0 { phi } else { 1.0 + phi };

                        Vec2::new(phi, theta)
                    },
                })
            }

            Geometry::Mesh(malha) => (0..malha.face_count())
                .filter_map(|i| Self::intersect_triangle(ray, malha.face(i)?))
                .reduce(|a, b| if b.t < a.t { b } else { a }),
        }
    }

    fn moller_trumbore(ray: Ray, vertices: [Vec3; 3]) -> Option<(f32, [f32; 3])> {
        let edge1 = vertices[1] - vertices[0];
        let edge2 = vertices[2] - vertices[0];

        let perpendicular = ray.direction.cross(edge2);
        let det = edge1.dot(perpendicular);

        if det.abs() < 1e-8 {
            return None;
        }

        let one_over_det = 1.0 / det;
        let to_vertex = ray.origin - vertices[0];

        let u = to_vertex.dot(perpendicular) * one_over_det;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = to_vertex.cross(edge1);

        let v = ray.direction.dot(q) * one_over_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge2.dot(q) * one_over_det;
        if t <= 0.0 {
            return None;
        }

        Some((t, [1.0 - u - v, u, v]))
    }

    pub fn intersect_triangle(ray: Ray, vertices: [Vertex; 3]) -> Option<Hit> {
        let (t, edges) = Self::moller_trumbore(ray, vertices.map(|v| v.position))?;

        let baricentrics = Baricentrics::new(edges);

        let shading_normal = baricentrics
            .interpolate_vec3(vertices.map(|v| v.normal))
            .normalized();

        let geometric_normal = {
            let n = (vertices[1].position - vertices[0].position)
                .cross(vertices[2].position - vertices[0].position)
                .normalized();

            if n.dot(shading_normal) < 0.0 { -n } else { n }
        };

        Some(Hit {
            t,
            shading_normal,
            geometric_normal,
            tangent: baricentrics.interpolate_tangent(vertices.map(|v| v.tangent)),
            uv: baricentrics.interpolate_vec2(vertices.map(|v| v.uv)),
        })
    }

    fn intersect_cube(r: Ray, center: Vec3, half_edge: f32) -> Option<Hit> {
        let half_edge = half_edge.abs();

        if half_edge == 0.0 {
            return None;
        }

        // O raio com o centro do cubo na origem: as seis faces são os planos
        // x = ±half_edge, y = ±half_edge e z = ±half_edge.
        let origin = r.origin - center;
        let direction = r.direction;

        let mut closest: Option<Hit> = None;

        // Para cada face: a normal, a direção pra direita e a direção pra baixo de quem
        // olha a face de fora.
        #[rustfmt::skip]
        let faces = [
            (Vec3::new( 1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0, -1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0,  1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0,  1.0)),
            (Vec3::new( 0.0,  0.0,  1.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new(-1.0,  0.0,  0.0), Vec3::new( 0.0, 0.0,  1.0), Vec3::new(0.0, -1.0,  0.0)),
            (Vec3::new( 0.0, -1.0,  0.0), Vec3::new( 1.0, 0.0,  0.0), Vec3::new(0.0,  0.0, -1.0)),
            (Vec3::new( 0.0,  0.0, -1.0), Vec3::new(-1.0, 0.0,  0.0), Vec3::new(0.0, -1.0,  0.0)),
        ];

        for (normal, up, down) in faces {
            let t = (half_edge - origin.dot(normal)) / direction.dot(normal);
            let point = origin + direction * t;

            // `(a, b)` são as coordenadas do ponto no plano da face, com 1 = half_edge. Se
            // a coordenada é < -1 ou > 1, o ponto está fora do quadrado da face.
            let (a, b) = (point.dot(up) / half_edge, point.dot(down) / half_edge);

            if t > 0.0 && a.abs() <= 1.0 && b.abs() <= 1.0 {
                let hit = Hit {
                    t,
                    shading_normal: normal,
                    geometric_normal: normal,
                    tangent: Tangent::from_uv_axes(normal, up, down),
                    uv: Vec2::new((a + 1.0) * 0.5, (b + 1.0) * 0.5),
                };

                closest = Some(closest.map_or(hit, |p| if hit.t < p.t { hit } else { p }));
            }
        }

        closest
    }

    fn intersect_sphere(ray: Ray, center: Vec3, radius: f32) -> Option<f32> {
        let oc = ray.origin - center;

        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - radius * radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let root = discriminant.sqrt();
        let t0 = (-b - root) / (2.0 * a);
        let t1 = (-b + root) / (2.0 * a);

        [t0, t1].into_iter().find(|&t| t > 0.0)
    }

    fn sphere_tangent(normal: Vec3) -> Tangent {
        let du = Vec3::new(-normal.z, 0.0, normal.x);

        if du.length() < 1e-6 {
            // Nos polos o paralelo tem raio zero e a direção do u não existe.
            return Tangent::any_perpendicular(normal);
        }

        Tangent::new(du.normalized(), 1.0)
    }

    pub fn bounds(&self) -> Aabb {
        match self {
            Geometry::Triangle { points } => Aabb::around(*points),

            Geometry::Cube { half_edge } => {
                let center = Vec3::ZERO;
                let half_edge = Vec3::ONE * half_edge.abs();
                Aabb {
                    min: center - half_edge,
                    max: center + half_edge,
                }
            }

            Geometry::Sphere { radius } => {
                let center = Vec3::ZERO;
                let radius = Vec3::ONE * radius.abs();
                Aabb {
                    min: center - radius,
                    max: center + radius,
                }
            }

            Geometry::Mesh(mesh) => Aabb::around(mesh.vertices.iter().copied()),
        }
    }
}
