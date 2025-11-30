use winit::error::EventLoopError;

#[derive(thiserror::Error, Debug)]
pub enum LatrError {
    #[error("The GPU ran into an error: {0}")]
    Gpu(#[from] GpuError),

    #[error("The Winit Window ran into an error: {0}")]
    Window(#[from] WindowError),
    
    #[error("The Engine ran into an error: {0}")]
    Engine(#[from] EngineError),
    
    #[error("The configuration ran into an error: {0}")]
    Config(#[from] ConfigError),
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
    
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Cannot find config file: {0}")]
    NotFound(#[from] std::io::Error),

    #[error("Cannot parse config file file: {0}")]
    Parse(#[from] toml::de::Error),
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