use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use crate::color::Rgb;
use crate::geometry::{Geometry, Hit};
use crate::math::{Aabb, Ray, Vec3};
use crate::overlay;
use crate::parameters::{BvhSplit, TransformMethod};
use crate::render::{ColorBuffer, RayTracingResult};
use crate::scene::{CameraInstance, GeometryInstance};

const BOX_COST: f32 = 0.5;
const PRIMITIVE_COST: f32 = 1.0;
const BUCKETS: usize = 12;

pub struct AccelerationStructures {
    tlas: Bvh,
    blas: Vec<Option<Arc<Bvh>>>,
}

impl AccelerationStructures {
    pub fn build(
        geometry_instances: &[GeometryInstance],
        bvh_split: BvhSplit,
        transform_method: TransformMethod,
        stats: &RefCell<RayTracingResult>,
    ) -> AccelerationStructures {
        let start = Instant::now();

        let mut bvh_cache: HashMap<usize, Arc<Bvh>> = HashMap::new();

        let blas: Vec<Option<Arc<Bvh>>> = geometry_instances
            .iter()
            .map(|instance| {
                let Geometry::Mesh(mesh) = &instance.primitive.geometry else {
                    return None;
                };

                Some(Arc::clone(
                    bvh_cache
                        .entry(Arc::as_ptr(mesh) as usize)
                        .or_insert_with(|| {
                            Arc::new(Bvh::build(
                                (0..mesh.face_count())
                                    .filter_map(|i| {
                                        Some(BvhNodeItem {
                                            index: i,
                                            bounds: Aabb::around(mesh.face(i)?.map(|v| v.position)),
                                            cost: PRIMITIVE_COST,
                                        })
                                    })
                                    .collect(),
                                bvh_split,
                            ))
                        }),
                ))
            })
            .collect();

        let items = geometry_instances
            .iter()
            .zip(&blas)
            .enumerate()
            .map(|(i, (instance, blas))| BvhNodeItem {
                index: i,
                // Precisa recalcular a AABB com os pontos em espaço de mundo.
                bounds: Aabb::around(instance.primitive.geometry.bounds().corners().map(
                    |corner| {
                        instance
                            .transform_chain
                            .apply_point(corner, transform_method)
                    },
                )),
                cost: blas.as_deref().map_or(PRIMITIVE_COST, Bvh::expected_cost),
            })
            .collect();

        let acceleration_structures = AccelerationStructures {
            tlas: Bvh::build(items, bvh_split),
            blas,
        };

        let mut stats = stats.borrow_mut();
        stats.bvh_build_time = start.elapsed();
        (stats.tlas_depth, stats.blas_depth) = acceleration_structures.depth();

        acceleration_structures
    }

    /// A profundidade da TLAS e a da BLAS mais funda.
    pub fn depth(&self) -> (u32, u32) {
        let blas = self
            .blas
            .iter()
            .flatten()
            .map(|blas| blas.depth())
            .max()
            .unwrap_or(0);

        (self.tlas.depth(), blas)
    }

    /// Desenha as caixas da TLAS e das BLAS até `profundidade`, uma cor por nível.
    pub fn wireframe(
        &self,
        camera_instance: &CameraInstance,
        geometry_instances: &[GeometryInstance],
        transform_method: TransformMethod,
        max_depth: u32,
        buffer: &mut ColorBuffer,
    ) {
        const COLORS: [Rgb; 6] = [
            Rgb::new(1.0, 1.00, 1.0),
            Rgb::new(1.0, 0.85, 0.2),
            Rgb::new(0.3, 1.00, 0.4),
            Rgb::new(0.3, 0.80, 1.0),
            Rgb::new(0.8, 0.40, 1.0),
            Rgb::new(1.0, 0.40, 0.4),
        ];

        for (level, corners) in self.aabbs(geometry_instances, transform_method, max_depth) {
            overlay::draw_box(
                camera_instance,
                corners,
                COLORS[(level.max(1) - 1) as usize % COLORS.len()],
                buffer,
            );
        }
    }

    pub fn closest_hit(
        &self,
        geometry_instances: &[GeometryInstance],
        ray: Ray,
        in_shadow_ray: bool,
        t_min: f32,
        t_max: f32,
        transform_method: TransformMethod,
        stats: &RefCell<RayTracingResult>,
    ) -> Option<(usize, Hit)> {
        let intersect_geometry = |i: usize, t_max: f32| -> Option<(f32, Hit)> {
            let geometry_instance = &geometry_instances[i];

            if in_shadow_ray && geometry_instance.primitive.material.is_emissive() {
                return None;
            }

            // Leva o raio pro espaço local do objeto.
            let ray_local = geometry_instance
                .transform_chain
                .undo_ray(ray, transform_method);

            let hit = match &geometry_instance.primitive.geometry {
                Geometry::Mesh(mesh) => {
                    let intersect_face = |i, _| {
                        let hit = Geometry::intersect_triangle(ray_local, mesh.face(i).unwrap());
                        stats.borrow_mut().primitive_tests += 1;

                        hit.map(|hit| (hit.t, hit))
                    };

                    let blas = self.blas[i].as_ref().unwrap();
                    blas.traverse(ray_local, t_min, t_max, stats, intersect_face)
                        .map(|(_, hit)| hit)
                }

                _ => {
                    let hit = geometry_instance.primitive.geometry.intersect(ray_local);
                    stats.borrow_mut().primitive_tests += 1;
                    hit
                }
            };

            hit.map(|hit| (hit.t, hit))
        };

        self.tlas
            .traverse(ray, t_min, t_max, stats, intersect_geometry)
    }

    /// As AABBs dos nós da TLAS e da BLAS até `profundidade`.
    fn aabbs(
        &self,
        geometry_instances: &[GeometryInstance],
        transform_method: TransformMethod,
        max_depth: u32,
    ) -> Vec<(u32, [Vec3; 8])> {
        let tlas = self
            .tlas
            .aabbs(max_depth)
            .into_iter()
            .map(|(depth, aabb)| (depth, aabb.corners()));

        let blas = self
            .blas
            .iter()
            .zip(geometry_instances)
            .filter_map(|(blas, geometry_instance)| Some((blas.as_ref()?, geometry_instance)))
            .flat_map(|(blas, geometry_instance)| {
                blas.aabbs(max_depth).into_iter().map(|(depth, aabb)| {
                    (
                        depth,
                        aabb.corners().map(|corner| {
                            geometry_instance
                                .transform_chain
                                .apply_point(corner, transform_method)
                        }),
                    )
                })
            });

        tlas.chain(blas).collect()
    }
}

struct Bvh {
    /// Cada nó vem depois dos seus filhos, então a raiz é o último.
    nodes: Vec<BvhNode>,
}

enum BvhNode {
    Leaf { bounds: Aabb, items: Vec<usize> },
    Internal { bounds: Aabb, children: [usize; 2] },
}

impl BvhNode {
    fn bounds(&self) -> Aabb {
        match *self {
            BvhNode::Leaf { bounds, .. } | BvhNode::Internal { bounds, .. } => bounds,
        }
    }
}

/// O que o node da BVH guarda: um triângulo na BLAS ou um objeto na TLAS.
#[derive(Debug, Clone, Copy)]
struct BvhNodeItem {
    index: usize,
    bounds: Aabb,
    cost: f32,
}

#[derive(Clone, Copy)]
struct Bucket {
    bounds: Aabb,
    cost: f32,
    count: usize,
}

impl Bucket {
    const EMPTY: Bucket = Bucket {
        bounds: Aabb::EMPTY,
        cost: 0.0,
        count: 0,
    };

    fn merge(self, other: Bucket) -> Bucket {
        Bucket {
            bounds: self.bounds.union(other.bounds),
            cost: self.cost + other.cost,
            count: self.count + other.count,
        }
    }
}

impl Bvh {
    fn build(mut items: Vec<BvhNodeItem>, split: BvhSplit) -> Bvh {
        let mut nodes = Vec::new();

        if !items.is_empty() {
            Self::subdivide(&mut nodes, &mut items, split);
        }

        Bvh { nodes }
    }

    fn subdivide(nodes: &mut Vec<BvhNode>, items: &mut [BvhNodeItem], split: BvhSplit) -> usize {
        let bounds = items
            .iter()
            .fold(Aabb::EMPTY, |bounds, item| bounds.union(item.bounds));

        let split_index = match split {
            BvhSplit::Median => Self::median_split(items),
            BvhSplit::Sah => Self::sah_split(items, bounds),
        };

        let node = match split_index {
            Some(split_index) => {
                let (left, right) = items.split_at_mut(split_index);

                BvhNode::Internal {
                    bounds,
                    children: [
                        Self::subdivide(nodes, left, split),
                        Self::subdivide(nodes, right, split),
                    ],
                }
            }

            None => BvhNode::Leaf {
                bounds,
                items: items.iter().map(|item| item.index).collect(),
            },
        };

        nodes.push(node);
        nodes.len() - 1
    }

    fn median_split(items: &mut [BvhNodeItem]) -> Option<usize> {
        const ITEMS_PER_LEAF: usize = 2;

        if items.len() <= ITEMS_PER_LEAF {
            return None;
        }

        // Escolher o eixo em que os centros estão mais espalhados (o lado da AABB dos centros é maior).
        let longest_axis = {
            let centroids = Aabb::around(items.iter().map(|item| item.bounds.center()));
            let extent = centroids.max - centroids.min;

            if extent.x >= extent.y && extent.x >= extent.z {
                0
            } else if extent.y >= extent.z {
                1
            } else {
                2
            }
        };

        let split_index = items.len() / 2;

        items.select_nth_unstable_by(split_index, |a, b| {
            a.bounds.center()[longest_axis].total_cmp(&b.bounds.center()[longest_axis])
        });

        Some(split_index)
    }

    fn sah_split(items: &mut [BvhNodeItem], bounds: Aabb) -> Option<usize> {
        if items.len() <= 1 {
            return None;
        }

        const MAX_ITEMS_PER_LEAF: usize = 8;

        let centroids = Aabb::around(items.iter().map(|item| item.bounds.center()));

        let bucket_index = |axis: usize, item: &BvhNodeItem| -> usize {
            let (min, max) = (centroids.min[axis], centroids.max[axis]);
            let i = (item.bounds.center()[axis] - min) / (max - min);

            ((i * BUCKETS as f32) as usize).min(BUCKETS - 1)
        };

        let mut best_split: Option<(f32, usize, usize)> = None;

        for axis in 0..3 {
            // Todos os centros no mesmo plano: nada a separar neste eixo.
            if centroids.max[axis] <= centroids.min[axis] {
                continue;
            }

            let buckets = {
                let mut result = [Bucket::EMPTY; BUCKETS];

                for item in items.iter() {
                    let i = bucket_index(axis, item);

                    result[i] = result[i].merge(Bucket {
                        bounds: item.bounds,
                        cost: item.cost,
                        count: 1,
                    });
                }

                result
            };

            let buckets_after = {
                let mut result = [Bucket::EMPTY; BUCKETS + 1];

                for i in (0..BUCKETS).rev() {
                    result[i] = buckets[i].merge(result[i + 1]);
                }

                result
            };

            let mut buckets_until = Bucket::EMPTY;

            for i in 1..BUCKETS {
                buckets_until = buckets_until.merge(buckets[i - 1]);

                if buckets_until.count == 0 || buckets_after[i].count == 0 {
                    // Se um lado corte está vazio.
                    continue;
                }

                let cost = buckets_until.bounds.surface_area() * buckets_until.cost
                    + buckets_after[i].bounds.surface_area() * buckets_after[i].cost;

                if best_split.is_none_or(|(lowest, ..)| cost < lowest) {
                    best_split = Some((cost, axis, i));
                }
            }
        }

        if best_split.is_none() {
            return if items.len() > MAX_ITEMS_PER_LEAF {
                Some(items.len() / 2)
            } else {
                None
            };
        }

        let (cost, axis, i) = best_split.unwrap();

        // leaf_cost  = soma dos custos dos itens
        // split_cost = 2·BOX_COST + (area_filho_esq / area_pai)·custo_esq + (area_filho_dir / area_pai)·custo_dir
        //            = 2·BOX_COST + (area_filho_esq·custo_filho_esq + area_filho_dir·custo_filho_dir) / area_pai
        //            = 2·BOX_COST + cost / area_pai
        //
        // Pra evitar dividir por 0, multiplicamos os dois lados por area_pai:
        //  split_cost · area_pai = 2·BOX_COST·area_pai + cost
        //  leaf_cost  · area_pai = soma(custos) · area_pai
        let split_cost = 2.0 * BOX_COST * bounds.surface_area() + cost;
        let leaf_cost = items.iter().map(|item| item.cost).sum::<f32>() * bounds.surface_area();

        if split_cost >= leaf_cost && items.len() <= MAX_ITEMS_PER_LEAF {
            return None;
        }

        items.sort_unstable_by_key(|it| bucket_index(axis, it) >= i);
        let split_index = items.partition_point(|it| bucket_index(axis, it) < i);

        Some(split_index)
    }

    fn expected_cost(&self) -> f32 {
        let root = self.root().expect("the BVH shouldn't be empty");
        let root_surface_area = self.nodes[root].bounds().surface_area();

        let hit_chance = |bounds: Aabb| {
            if root_surface_area > 0.0 {
                bounds.surface_area() / root_surface_area
            } else {
                1.0
            }
        };

        let nodes: f32 = self
            .nodes
            .iter()
            .map(|node| match node {
                BvhNode::Leaf { bounds, items } => {
                    hit_chance(*bounds) * items.len() as f32 * PRIMITIVE_COST
                }
                BvhNode::Internal { bounds, .. } => hit_chance(*bounds) * 2.0 * BOX_COST,
            })
            .sum();

        BOX_COST + nodes
    }
}

impl Bvh {
    fn root(&self) -> Option<usize> {
        self.nodes.len().checked_sub(1)
    }

    pub fn depth(&self) -> u32 {
        fn descend(bvh: &Bvh, i: usize) -> u32 {
            match &bvh.nodes[i] {
                BvhNode::Leaf { .. } => 1,
                &BvhNode::Internal {
                    children: [a, b], ..
                } => 1 + descend(bvh, a).max(descend(bvh, b)),
            }
        }

        match self.root() {
            Some(root) => descend(self, root),
            None => 0,
        }
    }

    /// As AABBs dos nodes até `depth`.
    fn aabbs(&self, depth: u32) -> Vec<(u32, Aabb)> {
        let Some(root) = self.root() else {
            return Vec::new();
        };

        let mut aabbs = Vec::new();
        let mut queue = VecDeque::from([(1, root)]);

        while let Some((level, i)) = queue.pop_front() {
            if level > depth {
                break;
            }

            aabbs.push((level, self.nodes[i].bounds()));

            if let &BvhNode::Internal { children, .. } = &self.nodes[i] {
                queue.extend(children.map(|child| (level + 1, child)));
            }
        }

        aabbs
    }

    /// Encontra o item mais próximo entre `t_min` e `t_max`, com o que `intersect`
    /// retornou para ele.
    pub fn traverse(
        &self,
        ray: Ray,
        t_min: f32,
        t_max: f32,
        stats: &RefCell<RayTracingResult>,
        mut intersect: impl FnMut(usize, f32) -> Option<(f32, Hit)>,
    ) -> Option<(usize, Hit)> {
        let node = self.root()?;
        let t_node = self.nodes[node].bounds().intersect(ray, t_min, t_max)?;

        stats.borrow_mut().bvh_nodes_visited += 1;

        let mut closest = None;
        let mut t_max = t_max;

        // Os nodes a visitar, com o `t` em que o raio entra em cada um.
        let mut stack = vec![(node, t_node)];

        while let Some((node, t_node)) = stack.pop() {
            if t_node >= t_max {
                continue;
            }

            match &self.nodes[node] {
                BvhNode::Leaf { items, .. } => {
                    for &item in items {
                        let Some((t, hit)) = intersect(item, t_max) else {
                            continue;
                        };

                        if t >= t_min && t < t_max {
                            t_max = t;
                            closest = Some((item, hit));
                        }
                    }
                }

                BvhNode::Internal { children, .. } => {
                    let [a, b] = children.map(|child| {
                        stats.borrow_mut().bvh_nodes_visited += 1;

                        let t = self.nodes[child].bounds().intersect(ray, t_min, t_max)?;
                        Some((child, t))
                    });

                    match (a, b) {
                        (Some(a), Some(b)) => {
                            let (near, far) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                            stack.push(far);
                            stack.push(near);
                        }
                        (Some(child), None) | (None, Some(child)) => stack.push(child),
                        (None, None) => {}
                    }
                }
            }
        }

        closest
    }
}
