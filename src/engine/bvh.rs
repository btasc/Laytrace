use rayon::prelude::*;

use glam::{
    Vec3, Vec3A,
    UVec3,
};
use crate::engine::bvh;
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

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}


struct BvhBranch {
    left: Box<BvhNode>,
    right: Box<BvhNode>,
    aabb: AABB,
}

struct RecurseCtx<'a> {
    aabbs: &'a [AABB],
    centroids: &'a [Vec3],
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

        bvh_recurse(&mut idxs, recurse_ctx);

        todo!()
    }

    pub fn flatten(self) -> Vec<GpuStorageBvhNode>{
        todo!()
    }
}

impl AABB {
    fn grow(&mut self, other_box: &AABB) {
        self.min = self.min.min(other_box.min);
        self.max = self.max.max(other_box.max);
    }

    fn shrink(&mut self, other_box: &AABB) {
        self.min = self.min.max(other_box.min);
        self.max = self.max.min(other_box.max);
    }
}

fn bvh_recurse(idxs: &mut [usize], ctx: RecurseCtx) {
    let best_cost: f64 = f64::MAX;

    // For each axis
    // X Y Z
    for axis in 0..3usize {

        let left_aabb = AABB {
            min: Vec3::default(),
            max: Vec3::default(),
        };

        let right_aabb = AABB {
            min: Vec3::default(),
            max: Vec3::default(),
        };

        // Sort our triangles by our current axis
        // This is where we take advantage of our &mut [usize] - we can just sort it without actually allocating a new vec
        idxs.sort_unstable_by(|a, b|
           ctx.centroids[*a][axis].total_cmp(&ctx.centroids[*b][axis])
        );

    }

}