use std::path::{ PathBuf, Path };
use std::io::ErrorKind;

use crate::gpu::buffers::{
    GpuStorageBlasLeafNode as BlasLeaf,
    GpuStorageBlasTreeNode as BlasTree,
    GpuStorageTriangleData as TriangleData,
    GpuStorageVertex as Vertex,
    GpuBuffers,
};

use crate::config::{
    ModelConfig,
    ExplicitModelConfig,
    DirectoriesConfig,
};

use crate::error::{ EngineError };

use rayon::prelude::*;

// I make this into a type so that if I ever decide to change it to like Vec<f32> or like Vec<[[f32; 3]; 3]> I can just change the type
type RawTriangleList = Vec<[f32; 9]>;

// This struct is used in gpu::buffers in GpuBuffers.write_bvh_res
pub struct BvhRes {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<TriangleData>,
    pub blas_tree: Vec<BlasTree>,
    pub blas_leaves: Vec<BlasLeaf>,
}

// This function is meant to be run on a separate thread
// This is the public entry to this file
// It handles most of the annoying io and writes to the buffers
pub fn build_write_bvh(model_config_file_path: PathBuf, buffers: &mut GpuBuffers, queue: &mut wgpu::Queue) -> Result<(), EngineError> {
    // We get the parent to use for any other io operations using the contents of the model config toml file
    let config_parent_dir = model_config_file_path.parent()
        .unwrap_or(Path::new("."));

    let model_config_file_contents = read_file_to_string_except_engine_err(model_config_file_path.clone())?;
    let model_config: ModelConfig = toml::from_str(&model_config_file_contents.as_str())?;

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
                    let bvh_res = build_mesh_bvh(raw_triangles);

                    buffers.write_bvh_res(bvh_res);
                }
            }
        }

    }

    println!("We got here");

    Ok(())
}

fn build_mesh_bvh(triangles: RawTriangleList) -> BvhRes {
    todo!()
}

fn parse_tri_file(file_path: PathBuf) -> Result<RawTriangleList, EngineError> {
    let file_contents = read_file_to_string_except_engine_err(file_path)?;
    

    todo!()
}

fn read_file_to_string_except_engine_err(path: PathBuf) -> Result<String, EngineError> {
    // We use .map_err to run only if there is an error
    // It matches the kind of the io error to the engine error
    // We do this as thiserror cant let us convert an enum to an error
    // This code was a copy and paste snippet, be careful with unintended behavior

    let file_contents = std::fs::read_to_string(&path)
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => EngineError::ModelConfigNotFound(path),
            ErrorKind::InvalidData => EngineError::ModelConfigInvalidData(path),
            _ => EngineError::Io(e),
        })?;

    Ok(file_contents)
}