use rayon::prelude::*;
use glam::{Vec3, Vec3A, UVec3, Vec3Swizzles};

use std::collections::VecDeque;

use crate::gpu::buffers::GpuStorageBvhNode;

// End of imports
// Per axis bins*
const BINS: usize = 12;

pub trait BvhPrimitive {
    fn get_aabb(&self) -> AABB;
    fn get_centroid(&self) -> Vec3;
}

pub enum BvhNode {
    Leaf(usize),
    Branch(BvhBranch)
}

#[derive(Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

struct BvhBranch {
    left: Box<BvhNode>,
    right: Box<BvhNode>,
    aabb: AABB,
}

#[derive(Clone, Copy)]
struct RecurseCtx<'a> {
    aabbs: &'a [AABB],
    centroids: &'a [Vec3],
}

#[derive(Clone, Copy)]
struct BvhBin {
    bounds: AABB,
    tri_count: usize,
}

impl BvhBin {
    fn add(&mut self, tri_bounds: &AABB) {
        self.bounds.grow(tri_bounds);
        self.tri_count += 1;
    }
}

impl Default for BvhBin {
    fn default() -> Self {
        BvhBin {
            bounds: AABB::new_max_inv(),
            tri_count: 0,
        }
    }
}

impl BvhNode {
    pub fn build<T: BvhPrimitive>(primitives: Vec<T>) -> Self {
        let mut idxs: Vec<usize> = (0..primitives.len()).collect();

        let (aabbs, centroids): (Vec<AABB>, Vec<Vec3>) = primitives
            .into_iter()
            .map(|prim| (prim.get_aabb(), prim.get_centroid()))
            .collect();

        let recurse_ctx = RecurseCtx {
            aabbs: &aabbs,
            centroids: &centroids,
        };

        let mut parent_centroid_box = AABB::new_max_inv();

        for tri_idx in idxs.iter() {
            parent_centroid_box.grow_from_point(recurse_ctx.centroids[*tri_idx]);
        }

        bvh_recurse(&mut idxs, parent_centroid_box, recurse_ctx)
    }

    // Our goal is to take each branch at a time and spit out a GpuStorageBvhNode
    // We use a deque to add branches to the back and pop them off the front as we go
    // We give each node a tag to its parent, the tag is composed to 2 usize-s
    // The first usize is the idx of its parent, and the second is the idx inside that parent
    // When we process our next node, we update the previous to reflect the childs idx
    pub fn flatten_to_blas(self) -> Vec<GpuStorageBvhNode>{
        let mut storage_vec: Vec<GpuStorageBvhNode> = Vec::new();

        // Our [usize; 2] is the tag mentioned
        let mut child_deque: VecDeque<(Box<BvhBranch>, [usize; 2])> = VecDeque::new();

        match self {
            // This is the edge case of a model being a single triangle
            // We give it an infinite bounding box with its triangle as the only thing in it
            BvhNode::Leaf(idx) => {
                let min = [f32::NEG_INFINITY, 0.0, 0.0, 0.0];
                let max = [f32::INFINITY, 1.0, 1.0, 1.0];

                storage_vec.push(GpuStorageBvhNode {
                    min_x: min, min_y: min, min_z: min,
                    max_x: max, max_y: max, max_z: max,
                    indices: [(idx + 1) as i32 * -1, 0, 0, 0],
                    _pad: [0, 0, 0, 0],
                });
            },
            BvhNode::Branch(branch) => {



            }
        }

        while let Some(node) = child_deque.pop_front() {

        }

        storage_vec
    }

    fn to_gpu_node(nodes: [Option<&BvhNode>; 4]) -> GpuStorageBvhNode {
        let storage = GpuStorageBvhNode::default();
        let mut node_num = 0;

        for _ in 0..4 {
            let Some(node) = nodes[node_num] else { continue; };



            node_num += 1;
        }

        storage
    }
}

impl AABB {
    fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn grow(&mut self, other_box: &AABB) {
        self.min = self.min.min(other_box.min);
        self.max = self.max.max(other_box.max);
    }

    pub fn grow_from_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    fn shrink(&mut self, other_box: &AABB) {
        self.min = self.min.max(other_box.min);
        self.max = self.max.min(other_box.max);
    }

    fn surface_area(&self) -> f32 {
        // We make use of some SIMD functions to calculate the surface area faster
        let d = self.max - self.min;
        let surfaces = d * d.yzx();

        2.0 * surfaces.element_sum()
    }

    pub fn new_max_inv() -> Self {
        AABB {
            min: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            max: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    fn iter_grow(&mut self, box_itt: impl Iterator<Item=AABB>) {
        box_itt.for_each(|aabb| self.grow(&aabb));
    }

    fn new_inf() -> Self {
        AABB {
            min: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

fn bvh_recurse(idxs: &mut [usize], parent_centroid_bounds: AABB, ctx: RecurseCtx) -> BvhNode {

    if idxs.len() == 1 {
        return BvhNode::Leaf(idxs[0]);
    }



    todo!()

}