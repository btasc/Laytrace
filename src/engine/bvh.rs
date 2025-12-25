use std::path::PathBuf;

use rayon::prelude::*;

use crate::gpu::buffers::{
    GpuStorageBlasLeafNode as BlasLeaf,
    GpuStorageBlasTreeNode as BlasTree,
    GpuBuffers,
};

// This function is meant to be run on a separate thread
pub fn build_write_bvh(model_file: Option<PathBuf>, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) {
    
}