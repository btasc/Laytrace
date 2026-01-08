use std::path::{ PathBuf, Path };
use std::io::ErrorKind;

use crate::gpu::buffers::{GpuBuffers, GpuStorageBvhNode};

use crate::core::config::{
    ModelConfig,
    ExplicitModelConfig,
    DirectoriesConfig,
};

use crate::core::error::EngineError;

use super::mesh_file_parsers::{
    read_file_to_string_except_engine_err,
    parse_tri_file,
};

// todo This file is out of date with other blas code
// Update this!

use rayon::prelude::*;
use glam::{ Vec3A, Vec3 };

// End of importing

pub type RawTriangleList = Vec<[f32; 9]>;

pub struct BvhTriBatch {
    vertices: Vec<RawTriangleList>,
    current_mem: usize,
}

impl BvhTriBatch {
    const MAX_MESH_NUM: usize = 24;
    const MAX_MEM_NUM: usize = 0x10_000_000; // 256 Megabytes
    
    fn new() -> Self {
        todo!()
    }
}


// This struct stores our raw list of triangles

// This function is meant to be run on a separate thread
// This is the public entry to this file
// It handles most of the annoying io and writes to the buffers
pub fn build_write_bvh(model_config_file_path: PathBuf, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) -> Result<(), EngineError> {
    // We get the parent to use for any other io operations using the contents of the model config toml file
    let config_parent_dir = model_config_file_path.parent()
        .unwrap_or(Path::new("."));

    let model_config_file_contents = read_file_to_string_except_engine_err(model_config_file_path.clone())?;
    let model_config: ModelConfig = toml::from_str(&model_config_file_contents.as_str())?;

    // We store a struct of raw triangle lists for rayon with par_iter
    // We get to some amount of raw triangles (either memory or number) then flush with into par iter rayon
    let mut bvh_tri_batch = BvhTriBatch::new();

    // Get the list of directories
    if let Some(dir_list) = model_config.directories {

        for dir in dir_list.model_folders {

            let full_dir_path = config_parent_dir.join(&dir);

            let file_iter = std::fs::read_dir(&full_dir_path)
                .map_err(|e| match e.kind() {
                    ErrorKind::NotFound => EngineError::ModelDirNotFound(dir.into()),
                    ErrorKind::NotADirectory => EngineError::InvalidDirectory(dir.into()),
                    _ => EngineError::Io(e),
                })?;

            for file_path_res in file_iter {
                let file_path = match file_path_res {
                    Ok(file_path) => file_path.path(),
                    Err(e) => {
                        eprintln!("Unreadable file in model dir: {}", e);
                        continue;
                    }
                };

                if !file_path.is_file() {
                    continue;
                }

                // We get the osstr from the extension and convert it to a str
                // We then match it against supported file extensions
                // For now we just have .tri, representing a soup of sets of 9 vertices
                let mut raw_triangles_op: Option<RawTriangleList> = None;

                if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
                    match extension {
                        "tri" => raw_triangles_op = Some(parse_tri_file(file_path.clone())?),
                        _ => (),
                    }
                }

                if let Some(raw_triangles) = raw_triangles_op {
                    let push_op = None; //bvh_tri_batch.push_and_check(raw_triangles);

                    if let Some(bvh_res_vec) = push_op {
                        buffers.write_blas_bvh(bvh_res_vec, queue);
                    }
                }
            }
        }
    }

    // Our loop over the model directories is done, now we do explicit directories

    // todo

    // After both runs, we flush any remaining models still in our batch
    let flush_op = None; //bvh_tri_batch.flush_option();

    if let Some(bvh_res_vec) = flush_op {
        buffers.write_blas_bvh(bvh_res_vec, queue);
    }

    Ok(())
}