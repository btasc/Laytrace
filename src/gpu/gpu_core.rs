use std::sync::Arc;

use crate::{config::LatrConfig, error::GpuError, LatrError};

use super::init_utils::{
    make_device_queue_surface_config,
    create_screen_texture,
    create_sampler,
};

use super::shaders::{
    ComputeRaytraceShader,
    RenderShader,
};

use super::buffers::GpuBuffers;

pub struct GpuCore {
    compute_raytrace_shader: ComputeRaytraceShader,
    render_shader: RenderShader,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl GpuCore {
    pub fn new(window: Arc<winit::window::Window>, ) -> Result<Self, GpuError> {
        
        let (device, queue, surface, config) = make_device_queue_surface_config(window.clone())?;
        let (width, height) = window.inner_size().into();

        // Since we only want a 2d one, we just do width and height with the z axis as 1
        let texture_size = wgpu::Extent3d { width, height, depth_or_array_layers: 1, };
        let sampler = create_sampler(&device);

        // This is our main screen texture that is written to and read from throughout our program
        let screen_texture = create_screen_texture(&device, texture_size);
        let screen_texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let buffers = GpuBuffers::new(&device);

        let compute_raytrace_shader = ComputeRaytraceShader::new(
            &device,
            &buffers,
            &screen_texture_view,
        );

        let render_shader = RenderShader::new(
            &device,

            // We pass these in as owned
            screen_texture_view,
            sampler,
        );

        Ok(Self {
            compute_raytrace_shader,
            render_shader,
            
            device, queue,
            surface, config,
        })
    }

    pub fn render(&mut self) -> Result<(), GpuError> {
        let output = self.surface.get_current_texture()?;

        let output_texture_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Make the command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        // Run compute raytracer
        { self.compute_raytrace_shader.run_compute_pass(
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

    pub fn wright_orders(&mut self, orders: &Vec<TriangleWorkOrder>) {
        
    }
}