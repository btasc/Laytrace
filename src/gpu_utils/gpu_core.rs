use std::sync::Arc;

use crate::{config::LatrConfig, error::GpuError, engine::params::GpuUniformParams, LatrError};

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
    pub fn new(engine_config: &LatrConfig, window: Arc<winit::window::Window>, starting_params: &GpuUniformParams) -> Result<Self, GpuError> {
        
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

    pub fn render(&mut self, uniform_buffer: &GpuUniformParams) -> Result<(), GpuError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Put the &uniform buffer we passed into this function into
        self.queue.write_buffer(
            &self.compute_shader.compute_uniform_buffer,
            0,
            bytemuck::cast_slice(&[*uniform_buffer]),
        );

        // Write the vertices storage buffer when we get to that

        // Make the command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        // Run compute stuff
        {
            let mut compute_pass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Ray Tracing Pass"),
                    timestamp_writes: None,
                });

            compute_pass.set_pipeline(&self.compute_shader.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_shader.compute_uniform_bindgroup, &[]);

            let workgroup_x = (self.config.width + 7) / 8;
            let workgroup_y = (self.config.height + 7) / 8;
            compute_pass.dispatch_workgroups(workgroup_x, workgroup_y, 1);
        }
        // The compute pass is now recorded in the encoder.

        // Now we run the render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, // Reference to the view that we need to draw on that we made earlier
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set the correct pipeline process
            render_pass.set_pipeline(&self.render_shader.render_pipeline);

            // Pass in the texture and sampler
            render_pass.set_bind_group(0, &self.render_shader.render_bindgroup, &[]);

            // This draws 2 triangles (6 vertices) to cover the screen.
            render_pass.draw(0..4, 0..1);
        }

        // The render pass is now recorded in the encoder.

        // Run the setup encoder
        self.queue.submit(std::iter::once(encoder.finish()));

        // Tell the window to present this queue
        output.present();

        Ok(())
    }
}

pub struct ComputeShader {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_uniform_bindgroup: wgpu::BindGroup,
    pub compute_uniform_buffer: wgpu::Buffer,
}

impl ComputeShader {
    pub fn new(device: &wgpu::Device, starting_params: &GpuUniformParams, screen_texture_view: &wgpu::TextureView) -> Self {
        let compute_uniform_buffer = create_uniform_buffer(&device, starting_params);

        let compute_uniform_bindgroup_layout = create_compute_bindgroup_layout(&device);
        let compute_uniform_bindgroup = create_compute_bindgroup(&device, &compute_uniform_bindgroup_layout, &screen_texture_view, &compute_uniform_buffer);

        let compute_pipeline = create_compute_pipeline(&device, &compute_uniform_bindgroup_layout);

        Self {
            compute_uniform_buffer,
            compute_pipeline,
            compute_uniform_bindgroup,
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