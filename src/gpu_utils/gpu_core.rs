use std::sync::Arc;

use crate::engine::engine_conf::{
    RunMode, EngineConfig,
};

use super::init_utils::{make_device_queue_surface_config};

use super::gpu_error::GpuError;

pub struct GpuCore {
    compute_shader: ComputeShader,
    render_shader: RenderShader,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuCore {
    pub fn new(engine_config: &EngineConfig, window: Arc<winit::window::Window>) -> Result<Self, GpuError> {
        
        let (device, queue, surface, config) = make_device_queue_surface_config(window.clone())?;
        
        let compute_shader = ComputeShader::new();
        let render_shader = RenderShader::new();

        Ok(Self {
            compute_shader,
            render_shader,
            
            device, queue,
            surface, config,
        })
    }
}

pub struct ComputeShader {

}

impl ComputeShader {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub struct RenderShader {
    
}

impl RenderShader {
    pub fn new() -> Self {
        Self {

        }
    }
}