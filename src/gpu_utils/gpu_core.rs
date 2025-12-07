use std::sync::Arc;

use crate::{config::LatrConfig, error::GpuError, engine::params::GpuUniformParams, LatrError};
use crate::engine::params::TriangleData;
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
    create_buffers,
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

        let compute_shader = ComputeShader::new(
            &device, gpu_uniform_params,
            vertex_params,
            triangle_params,
            &screen_texture_view
        );

        let render_shader = RenderShader::new(&device, &screen_texture_view, &sampler, &config);

        Ok(Self {
            compute_shader,
            render_shader,
            
            device, queue,
            surface, config,
        })
    }

    pub fn render(&mut self, uniform_params: &GpuUniformParams, vertices: &Vec<[f32; 3]>, triangle_data: &Vec<TriangleData>) -> Result<(), GpuError> {
        let output = self.surface.get_current_texture()?;
        let texture_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

        self.compute_shader.check_for_buffer_overflow(&vertices, &triangle_data);

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
            &texture_view
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

pub struct ComputeShader {
    pub pipeline: wgpu::ComputePipeline,
    pub bindgroup: wgpu::BindGroup,
    pub bindgroup_layout: wgpu::BindGroupLayout,
    pub uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub triangle_buffer: wgpu::Buffer,
}

impl ComputeShader {
    pub fn new(
        device: &wgpu::Device,
        uniform_params: &GpuUniformParams,
        vertices: &Vec<[f32; 3]>,
        triangle_data: &Vec<TriangleData>,
        screen_texture_view: &wgpu::TextureView,
    ) -> Self {
        let (compute_uniform_buffer, vertex_buffer, triangle_buffer) = create_buffers(
            &device,
            uniform_params,
            vertices,
            triangle_data,
        );

        let compute_bindgroup_layout = create_compute_bindgroup_layout(&device);

        let compute_bindgroup = create_compute_bindgroup(
            &device,
            &compute_bindgroup_layout,
            &screen_texture_view,
            &compute_uniform_buffer,
            &vertex_buffer,
            &triangle_buffer,
        );

        let compute_pipeline = create_compute_pipeline(&device, &compute_bindgroup_layout);

        Self {
            uniform_buffer: compute_uniform_buffer,
            pipeline: compute_pipeline,
            bindgroup: compute_bindgroup,
            bindgroup_layout: compute_bindgroup_layout,
            vertex_buffer,
            triangle_buffer,
        }
    }

    pub fn run_compute_pass(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
                timestamp_writes: None,
            });

        compute_pass.set_pipeline(&self.pipeline);

        compute_pass.set_bind_group(0, &self.bindgroup, &[]);

        let workgroup_x = (width + 7) / 8;
        let workgroup_y = (height + 7) / 8;
        compute_pass.dispatch_workgroups(workgroup_x, workgroup_y, 1);
    }

    pub fn check_for_buffer_overflow(
        &mut self,
        device: &wgpu::Device,
        vertices: &Vec<[f32; 3]>,
        triangle_data: &Vec<TriangleData>,
    ) {
        let current_vertex_buffer_size = self.vertex_buffer.size() as u64;
        let required_vertices_size = (size_of::<[f32; 3]>() * vertices.len()) as u64;

        let mut new_vertex_size: u64 = current_vertex_buffer_size;

        if(current_vertex_buffer_size < required_vertices_size) {
            new_vertex_size = current_vertex_buffer_size * 2;
        }

        let current_triangle_buffer_size = self.triangle_buffer.size() as u64;
        let required_triangle_size = (size_of::<TriangleData>() * triangle_data.len()) as u64;

        let mut new_triangle_size: u64 = current_triangle_buffer_size;

        if(current_triangle_buffer_size < required_triangle_size) {
            new_triangle_size = current_triangle_buffer_size * 2;
        }

        self.rebind_vertex_triangle_bindgroup_size(device, new_vertex_size, new_triangle_size);
    }

    fn rebind_vertex_triangle_bindgroup_size(&mut self, device: &wgpu::Device, new_vertex_size: u64, new_triangle_size: u64) {
        self.bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bindgroup_layout,
            label: Some("Compute Bindgroup"),
            entries: &[
                // Rebind the uniform as normal
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                // Texture view also stays the same
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.screen_texture),
                },
                //
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.triangle_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
    }
}

pub struct RenderShader {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroup: wgpu::BindGroup,
}

impl RenderShader {
    pub fn new(device: &wgpu::Device, screen_texture_view: &wgpu::TextureView, sampler: &wgpu::Sampler, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let render_bindgroup_layout = create_render_bindgroup_layout(&device);
        let render_bindgroup = create_render_bindgroup(&device, &render_bindgroup_layout, &screen_texture_view, &sampler);

        let render_pipeline = create_render_pipeline(&device, &surface_config, &render_bindgroup_layout);


        Self {
            bindgroup: render_bindgroup,
            pipeline: render_pipeline,
        }
    }

    pub fn run_render_pass(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
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
        render_pass.set_pipeline(&self.pipeline);

        // Pass in the texture and sampler
        render_pass.set_bind_group(0, &self.bindgroup, &[]);

        // This draws 2 triangles (6 vertices) to cover the screen.
        render_pass.draw(0..4, 0..1);
    }
}