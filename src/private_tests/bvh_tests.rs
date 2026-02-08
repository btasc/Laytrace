use std::path::PathBuf;
use glam::Vec3;

use crate::core::error::EngineError;

use crate::engine::{bvh::*, blas::*, mesh_file_parsers::*};
use crate::gpu::buffers::*;

const ROOT: &'static str = "./src/private_tests/test_files/";

#[test]
fn single_tri() -> Result<(), EngineError> {
    let path = PathBuf::from(ROOT).join("single_meshes/single_tri.tri");
    let raw_tris = parse_tri_file(&path)?;

    assert_eq!(raw_tris.len(), 1);
    assert_eq!(raw_tris[0][0], Vec3::from_array([0.0, 0.0, 0.0]));
    assert_eq!(raw_tris[0][1], Vec3::from_array([1.0, 1.0, 1.0]));
    assert_eq!(raw_tris[0][2], Vec3::from_array([3.0, 3.0, 3.0]));

    let leaf = BvhLeaf {
        aabb: raw_tris[0].get_aabb(),
        idx: 0,
    };

    let gpu_node = GpuStorageBvhNode {
        min_x: [leaf.aabb.min.x, 0f32, 0f32, 0f32],
        min_y: [leaf.aabb.min.y, 0f32, 0f32, 0f32],
        min_z: [leaf.aabb.min.z, 0f32, 0f32, 0f32],
        max_x: [leaf.aabb.max.x, 0f32, 0f32, 0f32],
        max_y: [leaf.aabb.max.y, 0f32, 0f32, 0f32],
        max_z: [leaf.aabb.max.z, 0f32, 0f32, 0f32],
        indices: [-1, 0, 0,0 ],
        _pad: [0, 0, 0, 0],
    };

    let node = BvhNode::build(raw_tris);
    assert_eq!(node, BvhNode::Leaf(leaf));

    let flattened = node.flatten_to_blas();
    assert_eq!(gpu_node, flattened[0]);
    assert_eq!(1, flattened.len());

    Ok(())
}