use std::path::{ PathBuf, Path };
use std::io::ErrorKind;

use crate::gpu::buffers::{
    GpuBuffers,
};

use crate::config::{
    ModelConfig,
    ExplicitModelConfig,
    DirectoriesConfig,
};

use super::bvh_core::{
    BvhRes,
    RawTriangleList,
    build_mesh_bvh,
};

use super::mesh_file_parsers::{
    read_file_to_string_except_engine_err,
    parse_tri_file,
};

use crate::error::{ EngineError };

use rayon::prelude::*;

pub struct BvhTriBatch {
    vertices: Vec<RawTriangleList>,
    current_mem: usize,
}

impl BvhTriBatch {
    const MAX_MESH_NUM: usize = 24;
    const MAX_MEM_NUM: usize = 0x10_000_000; // 256 Megabytes

    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            current_mem: 0,
        }
    }

    pub fn push_and_check(&mut self, new_triangles: RawTriangleList) -> Option<Vec<BvhRes>> {
        let new_mem_size = new_triangles.len() * std::mem::size_of::<[f32; 9]>();

        let would_overflow_mem = (self.current_mem + new_mem_size) > Self::MAX_MEM_NUM;
        let hit_count_limit = self.vertices.len() >= Self::MAX_MESH_NUM;

        if (hit_count_limit || would_overflow_mem) && !self.vertices.is_empty() {
            let results = self.flush();

            self.push_vertices(new_triangles, new_mem_size);

            return Some(results);
        }

        self.push_vertices(new_triangles, new_mem_size);
        None
    }

    // We have this as a separate flush so that it returns an option
    pub fn flush_option(&mut self) -> Option<Vec<BvhRes>> {
        if self.vertices.is_empty() {
            return None;
        }

        Some(self.flush())
    }

    fn push_vertices(&mut self, data: RawTriangleList, size: usize) {
        self.current_mem += size;
        self.vertices.push(data);
    }

    // Flush option, returns
    fn flush(&mut self) -> Vec<BvhRes> {
        // std::mem::take lets us take ownership of the vec with just an access level of &mut self
        let batch_to_process = std::mem::take(&mut self.vertices);

        self.current_mem = 0;

        batch_to_process
            .into_par_iter()
            .map(|raw_triangle_list| build_mesh_bvh(raw_triangle_list))
            .collect()
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
                    let push_op = bvh_tri_batch.push_and_check(raw_triangles);

                    if let Some(bvh_res_vec) = push_op {
                        buffers.write_bvh_res_batch(bvh_res_vec, queue);
                    }
                }
            }
        }
    }

    // Our loop over the model directories is done, now we do explicit directories

    // todo

    // After both runs, we flush any remaining models still in our batch
    let flush_op = bvh_tri_batch.flush_option();

    if let Some(bvh_res_vec) = flush_op {
        buffers.write_bvh_res_batch(bvh_res_vec, queue);
    }

    Ok(())
}