use winit::error::EventLoopError;

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
    Window(#[from] winit::error::OsError),
}

#[derive(thiserror::Error, Debug)]
pub enum EngineError {

}

#[derive(thiserror::Error, Debug)]
pub enum GpuError {
    // Initiation Errors
    #[error("Failed to get wgpu device and queue during initialization: {0}")]
    DeviceError(#[from] wgpu::RequestDeviceError),

    #[error("Failed to find suitable adapter during initialzation")]
    AdapterNotFound(#[from] wgpu::RequestAdapterError),

    #[error("Failed to create surface during initialzation: {0}")]
    SurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("Failed to find any supported formats on adapter")]
    NoSupportedFormats,

    #[error("Failed to find any supported alpha modes on adapter")]
    NoSupportedAlphaModes,
}