use crate::core::error::GpuError;
use std::sync::Arc;

pub fn make_device_queue_surface_config(window_arc: Arc<winit::window::Window>) -> Result<(wgpu::Device, wgpu::Queue, wgpu::Surface<'static>, wgpu::SurfaceConfiguration), GpuError> {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window_arc.clone())?;

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))?;

    let (device, queue) = pollster::block_on(adapter.request_device( &wgpu::DeviceDescriptor {
        label: None,
        required_features: Default::default(),
        required_limits: Default::default(),
        experimental_features: Default::default(),
        memory_hints: Default::default(),
        trace: Default::default(),
    },))?;

    let size = window_arc.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format =
        surface_caps
        .formats
        .first()
        .ok_or(GpuError::NoSupportedFormats)?
        .clone();

    let alpha_mode =
        surface_caps
        .alpha_modes
        .first()
        .ok_or(GpuError::NoSupportedAlphaModes)?
        .clone();

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        desired_maximum_frame_latency: 0,
        alpha_mode: alpha_mode,
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    Ok((device, queue, surface, config))
}

// Creates the texture we write to with the compute shader and read from the fragment shader
// Important constants
    // Texture format that we're using for the screen texture
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub fn create_screen_texture(device: &wgpu::Device, texture_size: wgpu::Extent3d) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screen Texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        view_formats: &[],

        dimension: wgpu::TextureDimension::D2,
        // ! Compute buffer must output this format and the fragment shader must read this format correctly
        format: TEXTURE_FORMAT,
        
        // OR operator merges the bitflags to mean both texture binding and storage binding
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
    })
}

// Creates the sampler that is used for scaling up the screen when changing resolutions
// Note: Creates black bars when scaling up resolution
// Important constants
    // Method of scaling
    // Linear = scales up normally, just blurring slightly to scale up correctly
    const SCALE_METHOD: wgpu::FilterMode = wgpu::FilterMode::Linear;

pub fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: SCALE_METHOD,
        min_filter: SCALE_METHOD,

        ..Default::default()
    })
}