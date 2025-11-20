use crate::error::GpuError;
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