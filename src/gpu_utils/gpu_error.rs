#[derive(thiserror::Error, Debug)]
pub enum GpuError {
    // Initizlation Errors
    #[error("Failed to get wgpu device and queue during initialzation: {0}")]
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