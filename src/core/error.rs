use std::path::PathBuf;
use std::io::ErrorKind;

#[derive(thiserror::Error, Debug)]
pub enum LatrError {
    #[error("The GPU ran into an error: {0}")]
    Gpu(#[from] GpuError),

    #[error("The Winit Window ran into an error: {0}")]
    Window(#[from] WindowError),

    #[error("The Engine ran into an error: {0}")]
    Engine(#[from] EngineError),
}

#[derive(thiserror::Error, Debug)]
pub enum WindowError {
    #[error("Error occurred with event loop when initializing Winit window: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error("Error occurred when building Winit window with Winit: {0}")]
    WindowInit(#[from] winit::error::OsError),

    #[error("Event loop exited for an unknown reason")]
    EventLoopExited,
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    // Model config parsing errors
    #[error("Model config file not found at specified location: {0}")]
    ModelConfigNotFound(PathBuf),

    #[error("Model directory is not found at specified location: {0}")]
    ModelDirNotFound(PathBuf),

    #[error("Invalid directory passed in model config. Directory: {0}")]
    InvalidDirectory(PathBuf),

    #[error("Model config file has invalid data. Path: {0}")]
    ModelConfigInvalidData(PathBuf),

    #[error("Model config ran into an unknown IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model config parse error: {0}")]
    ModelParse(#[from] toml::de::Error),

    #[error("Tri file parse error. Invalid data. Check for data that is not a standard floating point number: {0}")]
    TriFileParse(#[from] std::num::ParseFloatError),

    #[error("Tri file parse error. Floats in .tri file are not in a multiple of 9. Each triangle should be nine floats.")]
    TriFileFloatNum,
}

impl EngineError {
    pub const POISON_ERR: &'static str = "Main thread panicked; Ending engine process";
}

#[derive(thiserror::Error, Debug)]
pub enum GpuError {
    // Initiation Errors
    #[error("Failed to get wgpu device and queue during initialization: {0}")]
    DeviceError(#[from] wgpu::RequestDeviceError),

    #[error("Failed to find suitable adapter during initialization")]
    AdapterNotFound(#[from] wgpu::RequestAdapterError),

    #[error("Failed to create surface during initialization: {0}")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("Failed to find any supported formats on adapter")]
    NoSupportedFormats,

    #[error("Failed to find any supported alpha modes on adapter")]
    NoSupportedAlphaModes,
    
    #[error("Failed during encoding and submitting process")]
    EncoderError(#[from] wgpu::SurfaceError),
}