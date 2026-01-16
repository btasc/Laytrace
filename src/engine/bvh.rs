use rayon::prelude::*;

use glam::{Vec3, Vec3A, UVec3, Vec3Swizzles};

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

    pub fn flatten(self) -> Vec<GpuStorageBvhNode>{
        let storage_vec: Vec<GpuStorageBvhNode> = Vec::new();

        storage_vec
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

    let mut best_cost: f32 = f32::MAX;

    // 0 -> x, 1 -> y, 2 -> z, it's the index of the Vec3
    let mut best_axis: usize = 0;
    let mut best_split: f32 = f32::NAN;

    // For each axis
    // X Y Z
    for axis in 0..3usize {
        let axis_extent: f32 = (parent_centroid_bounds.max[axis] - parent_centroid_bounds.min[axis]) as f32;
        let mut bins: [BvhBin; BINS] = [BvhBin::default(); BINS];

        for idx in idxs.iter() {
            // Distance of tri centroid from side, divided by axis_extent for the ratio of tri to box for our bin
            let tri_extent: f32 = (ctx.centroids[*idx][axis] - parent_centroid_bounds.min[axis]) as f32;
            // BINS causes an off by one error, so we -1 at the end
            let bin_idx: usize = ( (tri_extent / axis_extent) * (BINS as f32) ).floor() as usize - 1;

            bins[bin_idx].add(&ctx.aabbs[*idx]);
        }

        let mut left_surface_areas: [f32; BINS] = [0.0; BINS];
        let mut left_tri_count: [usize; BINS] = [0; BINS];

        let mut iter_bounds = AABB::new_max_inv();
        let mut iter_count = 0;

        for i in 0..BINS {
            iter_bounds.grow(&bins[i].bounds);
            iter_count += &bins[i].tri_count;

            left_surface_areas[i] = iter_bounds.surface_area();
            left_tri_count[i] = iter_count;
        }

        iter_bounds = AABB::new_max_inv();
        iter_count = 0;

        let mut right_surface_areas: [f32; BINS] = [0.0; BINS];
        let mut right_tri_count: [usize; BINS] = [0; BINS];

        for i in (0..BINS).rev() {
            iter_bounds.grow(&bins[i].bounds);
            iter_count += &bins[i].tri_count;

            right_surface_areas[i] = iter_bounds.surface_area();
            right_tri_count[i] = iter_count;
        }

        for i in 0..BINS {
            let split_cost = left_surface_areas[i] * left_tri_count[i] as f32 + right_surface_areas[i] * right_tri_count[i] as f32;

            if split_cost < best_cost {
                best_cost = split_cost;

                best_split = (i as f32 * axis_extent) / (BINS as f32 * axis_extent);
                best_axis = axis;
            }
        }
    }

    let mut mid: usize = 0;

    // This is the Lomuto Partition Algorithm
    // It just splits our array down the middle for our .split_at_mut
    for i in 0..idxs.len() {

        if ctx.centroids[idxs[i]][best_axis] <= best_split {
            idxs.swap(i, mid);
            mid += 1;
        }
    }

    let (left_split, right_split): (&mut [usize], &mut [usize]) = idxs.split_at_mut(mid);
    let (mut left_centroid_bounds, mut right_centroid_bounds) = (AABB::new_max_inv(), AABB::new_max_inv());

    for tri_idx in left_split.iter() {
        left_centroid_bounds.grow_from_point(ctx.centroids[*tri_idx]);
    }

    for tri_idx in right_split.iter() {
        right_centroid_bounds.grow_from_point(ctx.centroids[*tri_idx]);
    }

    let (left_res, right_res): (BvhNode, BvhNode) =
        rayon::join(|| bvh_recurse(left_split, left_centroid_bounds, ctx), || bvh_recurse(right_split, right_centroid_bounds, ctx));

    BvhNode::Branch(BvhBranch {
        left: Box::new(left_res),
        right: Box::new(right_res),
        aabb: Default::default(),
    })
}