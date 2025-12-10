use std::sync::Arc;

use crate::{config::LatrConfig, error::GpuError, engine::params::GpuUniformParams, LatrError};
use crate::engine::params::TriangleData;

use super::init_utils::{
    make_device_queue_surface_config,
    create_screen_texture,
    create_sampler,
};

use super::shaders::{
    ComputeRaytraceShader, RenderShader
};


pub struct GpuCore {
    compute_shader: ComputeRaytraceShader,
    render_shader: RenderShader,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuCore {
    pub fn new(
        engine_config: &LatrConfig,
        window: Arc<winit::window::Window>,
        gpu_uniform_params: &GpuUniformParams,
        vertex_params: &Vec<[f32; 3]>,
        triangle_params: &Vec<TriangleData>
    ) -> Result<Self, GpuError> {
        
        let (device, queue, surface, config) = make_device_queue_surface_config(window.clone())?;

        let (width, height) = window.inner_size().into();

        // Since we only want a 2d one, we just do width and height with the z axis as 1
        let texture_size = wgpu::Extent3d { width, height, depth_or_array_layers: 1, };
        let sampler = create_sampler(&device);

        // This is our main screen texture that is written to and read from throughout our program
        let screen_texture = create_screen_texture(&device, texture_size);
        let screen_texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_shader = ComputeRaytraceShader::new(
            &device, gpu_uniform_params,
            vertex_params,
            triangle_params,
            &screen_texture_view
        );

        let render_shader = RenderShader::new(
            &device,
            screen_texture_view,
            &sampler,
            &config
        );

        Ok(Self {
            compute_shader,
            render_shader,
            
            device, queue,
            surface, config,
        })
    }

    pub fn render(&mut self, uniform_params: &GpuUniformParams, vertices: &Vec<[f32; 3]>, triangle_data: &Vec<TriangleData>) -> Result<(), GpuError> {
        let output = self.surface.get_current_texture()?;

        let output_texture_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.compute_shader.check_for_buffer_overflow(
            &self.device,
            &self.render_shader.screen_texture_view,
            &vertices,
            &triangle_data
        );

        { self.wright_buffers(
            &uniform_params,
            &vertices,
            &triangle_data
        ) };

        // Make the command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        // Run compute stuff
        { self.compute_shader.run_compute_pass(
            &mut encoder,
            self.config.width,
            self.config.height
        ); };

        // The compute pass is now recorded in the encoder.

        // Now we run the render pass
        { self.render_shader.run_render_pass(
            &mut encoder,
            &output_texture_view
        ); };

        // The render pass is now recorded in the encoder.

        // Run the setup encoder
        self.queue.submit(std::iter::once(encoder.finish()));

        // Tell the window to present this queue
        output.present();

        Ok(())
    }

    pub fn wright_buffers(
        &self,
        uniform_params: &GpuUniformParams,
        vertices: &Vec<[f32; 3]>,
        triangle_data: &Vec<TriangleData>,
    ) {
        self.queue.write_buffer(
            &self.compute_shader.uniform_buffer,
            0,
            bytemuck::cast_slice(&[*uniform_params]),
        );

        self.queue.write_buffer(
            &self.compute_shader.vertex_buffer,
            0,
            bytemuck::cast_slice(vertices),
        );

        self.queue.write_buffer(
            &self.compute_shader.triangle_buffer,
            0,
            bytemuck::cast_slice(triangle_data),
        );
    }
}