use std::sync::Arc;

use crate::{
    config::LatrConfig,
    error::GpuError,
    engine::simulation_params::SimulationParams,
};

use super::init_utils::{
    make_device_queue_surface_config,
    create_screen_texture,
    create_sampler,
};

use super::bind_groups::{
    create_compute_bindgroup_layout,
    create_render_bindgroup_layout,
    create_compute_bindgroup,
    create_render_bindgroup,
    create_uniform_buffer,
};

use super::pipelines::{
    create_compute_pipeline,
    create_render_pipeline
};

pub struct GpuCore {
    compute_shader: ComputeShader,
    render_shader: RenderShader,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuCore {
    pub fn new(engine_config: &LatrConfig, window: Arc<winit::window::Window>, starting_params: &SimulationParams) -> Result<Self, GpuError> {
        
        let (device, queue, surface, config) = make_device_queue_surface_config(window.clone())?;

        let (width, height) = window.inner_size().into();

        // Since we only want a 2d one, we just do width and height with the z axis as 1
        let texture_size = wgpu::Extent3d { width, height, depth_or_array_layers: 1, };
        let sampler = create_sampler(&device);

        // This is our main screen texture that is written to and read from throughout our program
        let screen_texture = create_screen_texture(&device, texture_size);
        let screen_texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_shader = ComputeShader::new(&device, starting_params, &screen_texture_view);
        let render_shader = RenderShader::new(&device, &screen_texture_view, &sampler, &config);

        Ok(Self {
            compute_shader,
            render_shader,
            
            device, queue,
            surface, config,
        })
    }
}

pub struct ComputeShader {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bindgroup: wgpu::BindGroup,
}

impl ComputeShader {
    pub fn new(device: &wgpu::Device, starting_params: &SimulationParams, screen_texture_view: &wgpu::TextureView) -> Self {
        let compute_uniform_buffer = create_uniform_buffer(&device, starting_params);

        let compute_bindgroup_layout = create_compute_bindgroup_layout(&device);
        let compute_bindgroup = create_compute_bindgroup(&device, &compute_bindgroup_layout, &screen_texture_view, &compute_uniform_buffer);

        let compute_pipeline = create_compute_pipeline(&device, &compute_bindgroup_layout);

        Self {
            compute_pipeline,
            compute_bindgroup,
        }
    }
}

pub struct RenderShader {
    pub render_pipeline: wgpu::RenderPipeline,
    pub render_bindgroup: wgpu::BindGroup,
}

impl RenderShader {
    pub fn new(device: &wgpu::Device, screen_texture_view: &wgpu::TextureView, sampler: &wgpu::Sampler, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let render_bindgroup_layout = create_render_bindgroup_layout(&device);
        let render_bindgroup = create_render_bindgroup(&device, &render_bindgroup_layout, &screen_texture_view, &sampler);

        let render_pipeline = create_render_pipeline(&device, &surface_config, &render_bindgroup_layout);


        Self {
            render_bindgroup,
            render_pipeline,
        }
    }
}