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

use super::bvh::{BvhNode, BvhPrimitive, AABB};

use rayon::prelude::*;
use glam::{ Vec3A, Vec3 };

// End of importing

pub trait RawTriangleParse {
    fn from_9_floats(floats: [f32; 9]) -> RawTriangle;
}

pub type RawTriangle = [Vec3; 3];

impl RawTriangleParse for RawTriangle {
    fn from_9_floats(floats: [f32; 9]) -> RawTriangle {
        [
            Vec3::new(floats[0], floats[1], floats[2]),
            Vec3::new(floats[3], floats[4], floats[5]),
            Vec3::new(floats[6], floats[7], floats[8])
        ]
    }
}

impl BvhPrimitive for RawTriangle {
    fn get_aabb(&self) -> AABB {
        let mut aabb = AABB::new_max_inv();

        for i in 0..3 {
            aabb.grow_from_point(self[i]);
        }

        aabb
    }

    fn get_centroid(&self) -> Vec3 { (self[0] + self[1] + self[2]) / 3.0 }
}

pub struct BvhTriBatch {
    raw_meshes: Vec<Vec<RawTriangle>>,
    current_mem: usize,
}

impl BvhTriBatch {
    const MAX_MESH_NUM: usize = 24;
    const MAX_MEM_NUM: usize = 0x10_000_000; // 256 Megabytes
    
    fn new() -> Self {
        BvhTriBatch {
            raw_meshes: Vec::new(),
            current_mem: 0,
        }
    }

    fn push(&mut self, vertices: Vec<RawTriangle>) {
        self.current_mem += size_of::<RawTriangle>() * vertices.len();
        self.raw_meshes.push(vertices);
    }

    fn check_push(mut self, vertices: Vec<RawTriangle>, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) -> Self {
        if self.raw_meshes.len() >= Self::MAX_MESH_NUM || size_of::<RawTriangle>() * vertices.len() + self.current_mem >= Self::MAX_MEM_NUM {
            self = self.flush(buffers, queue);
        }

        self.push(vertices);
        self
    }

    fn flush(mut self, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) -> Self {
        if self.raw_meshes.is_empty() {
            return Self::new();
        }

        let res: Vec<BvhNode> = self.raw_meshes.into_par_iter().map(|v| BvhNode::build(v)).collect();
        let gpu_batches: Vec<Vec<GpuStorageBvhNode>> = res.into_iter().map(|r| r.flatten_to_blas()).collect();

        for batch in gpu_batches {
            buffers.write_blas_bvh(batch, queue);
        }

        Self::new()
    }
}


// This struct stores our raw list of triangles

// This function is meant to be run on a separate thread
// This is the public entry to this file
// It handles most of the annoying io and writes to the buffers
pub fn build_blas(model_config_file_path: PathBuf, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) -> Result<(), EngineError> {
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
                let mut raw_triangles_op: Option<Vec<RawTriangle>> = None;

                if let Some(extension) = file_path.extension().and_then(|s| s.to_str()) {
                    match extension {
                        // todo todo todo please
                        "tri" => raw_triangles_op = parse_tri_file(&file_path),
                        _ => (),
                    }
                }

                if let Some(raw_triangles) = raw_triangles_op {
                    bvh_tri_batch = bvh_tri_batch.check_push(raw_triangles, buffers, queue);
                }
            }
        }
    }

    // Our loop over the model directories is done, now we do explicit directories

    // todo

    // After both runs, we flush any remaining models still in our batch
    bvh_tri_batch.flush(buffers, queue);

    Ok(())
}