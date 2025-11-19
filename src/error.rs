use crate::gpu_utils::gpu_error::GpuError;

#[derive(thiserror::Error, Debug)]
pub enum LatrError {
    #[error("The GPU ran into an error: {0}")]
    Gpu(#[from] GpuError),
}