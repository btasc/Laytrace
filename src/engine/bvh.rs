use rayon::prelude::*;
use glam::{Vec3, Vec3A, UVec3, Vec3Swizzles};

use std::collections::VecDeque;
use wgpu::DeviceDescriptor;
use crate::gpu::buffers::GpuStorageBvhNode;

// End of imports
// Per axis bins*
const BINS: usize = 12;

pub trait BvhPrimitive {
    fn get_aabb(&self) -> AABB;
    fn get_centroid(&self) -> Vec3;
}

pub enum BvhNode {
    Leaf(BvhLeaf),
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

struct BvhLeaf {
    aabb: AABB,
    idx: usize,
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

type DequeChild = (Box<BvhNode>, usize);

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

    // The idea behind this function is that it takes our binary tree that uses the format of 2
    // heap ptrs and some payload, and converts it to a bvh4 system, where each node contains
    // 4 indices (instead of heap ptrs), along with the corresponding payload
    // See docs for explanation of format (I might not of written them yet assuming this project is still early)

    // We do this by having a deque of all children to be processed
    // Instead of going through each node, constructing its GpuStorageBvhNode with all the filled in data
    // then returning that, we instead just add a default impl of it to the array plus a usize set to 0, and then give all
    // its children the index to that empty node

    // Now for each child, we can trace back to its parent, and fill in the information needed, then
    // inc the usize value to show the amount of times it has been inced
    pub fn flatten_to_blas(self) -> Vec<GpuStorageBvhNode> {
        let mut storage_vec: Vec<(GpuStorageBvhNode, usize)> = Vec::new();
        let mut child_deque: VecDeque<DequeChild> = VecDeque::new();

        // Start the cycle by adding a blank storage node and 0 representing there being 0 children slotted
        storage_vec.push((GpuStorageBvhNode::default(), 0));
        child_deque.extend(self.take_4(0).into_iter().flatten());

        while let Some((node, parent_idx)) = child_deque.pop_front() {
            // First we do the preprocessing, which is updating the parent with this nodes information
            // We scope this to drop our mutable ref to the storage vec after this is done, and it looks nice
            {
                // Since we are going to append this node's gpu node to the end, the index of the soon-to-be
                // node is just the len of the storage vec
                let idx: i32 = match &*node {
                    BvhNode::Leaf(leaf) => (leaf.idx as i32 + 1) * -1,
                    BvhNode::Branch(_) => storage_vec.len() as i32 + 1,
                };

                let (storage_node, slot) = &mut storage_vec[parent_idx];
                node.slot_to_gpu_node(storage_node, *slot, idx);

                // Inc the slot num to show that the next space is available for future children
                *slot += 1;
            }

            // Now we append our blank gpu storage node for future children to write to
            // 0 represents 0 children currently in the storage node
            // We only do this if we're on a branch
            if let BvhNode::Branch(_) = *node {
                storage_vec.push((GpuStorageBvhNode::default(), 0));

                // Now we redo our take cycle, adding this nodes children to the deque back
                // Our parent idx is the idx of our current node, as its about to be the parent to the 4 children
                // Again, the idx of the current node is the storage vec len, but since we appended it, we subtract 1
                child_deque.extend(node.take_4(storage_vec.len() - 1).into_iter().flatten());
            }
        }

        storage_vec.shrink_to_fit();

        // Finally, we remove our unnecessary slot num value and return our storage nodes
        storage_vec.into_iter()
            .map(|s| s.0)
            .collect()
    }

    fn take_4(self, parent_idx: usize) -> [Option<DequeChild>; 4] {
        let mut child_num: usize = 0;
        let mut ret_arr: [Option<DequeChild>; 4] = [const { None }; 4];

        let BvhNode::Branch(branch) = self else {
            return ret_arr;
        };

        let (left, right) = (branch.left, branch.right);

        Self::match_child(left, &mut ret_arr, &mut child_num, parent_idx);
        Self::match_child(right, &mut ret_arr, &mut child_num, parent_idx);

        ret_arr
    }

    // Helper function for take 4
    fn match_child(node: Box<BvhNode>, ret: &mut [Option<DequeChild>; 4], child_num: &mut usize, parent_idx: usize) {
        match *node {
            BvhNode::Leaf(_) => {
                ret[*child_num] = Some((node, parent_idx));
                *child_num += 1;
            },
            BvhNode::Branch(branch) => {
                ret[*child_num] = Some((branch.left, parent_idx));
                *child_num += 1;

                ret[*child_num] = Some((branch.right, parent_idx));
                *child_num += 1;
            }
        }
    }

    fn slot_to_gpu_node(&self, node: &mut GpuStorageBvhNode, slot: usize, idx: i32) {
        debug_assert!(slot <= 3, "Requested slot at slot_to_gpu_node in engine::bvh is greater than 3 (0-3), the max amount of children a bvh4 node can have.");

        let aabb = match self {
            BvhNode::Leaf(leaf) => leaf.aabb,
            BvhNode::Branch(branch) => branch.aabb,
        };

        node.indices[slot] = idx;

        node.min_x[slot] = aabb.min.x;
        node.min_y[slot] = aabb.min.y;
        node.min_z[slot] = aabb.min.z;
        node.max_x[slot] = aabb.max.x;
        node.max_y[slot] = aabb.max.y;
        node.max_z[slot] = aabb.max.z;
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


// Note if idxs is multiple (the child will be a bvh branch in this case),
// we then have parent bounds be equal to the bounds of the centroids of the box
// If idxs is a single usize, then we have a single triangle. In this case, we recurse
// with parent bounds being equal to
fn bvh_recurse(idxs: &mut [usize], parent_bounds: AABB, ctx: RecurseCtx) -> BvhNode {

    if idxs.len() == 1 {
        return BvhNode::Leaf(BvhLeaf {
            idx: idxs[0],
            aabb: parent_bounds,
        });
    }



    todo!()
}