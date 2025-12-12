use wgpu::BindGroup;
use crate::engine::params::{
    GpuUniformParams,
    TriangleData
};

use super::bind_groups::{
    create_raytrace_compute_buffers,
    create_raytrace_compute_bindgroup,
    create_raytrace_compute_bindgroup_layout,
    create_render_bindgroup,
    create_render_bindgroup_layout
};

use super::pipelines::{
    create_raytrace_compute_pipeline,
    create_render_pipeline
};

pub struct ComputeTransformShader {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,

    pub uniform_describe_buffer: wgpu::Buffer,
    pub order_buffer: wgpu::Buffer,

    // Also shares vertex buffer and triangle buffer with the raytrace shader
}

impl ComputeTransformShader {
    pub fn new() -> Self {
        let ()

        Self {

        }
    }
}

pub struct ComputeRaytraceShader {
    pub pipeline: wgpu::ComputePipeline,
    pub bindgroup: wgpu::BindGroup,
    pub bindgroup_layout: wgpu::BindGroupLayout,

    pub uniform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub triangle_buffer: wgpu::Buffer,
}

impl ComputeRaytraceShader {
    pub fn new(
        device: &wgpu::Device,
        screen_texture_view: &wgpu::TextureView,
    ) -> Self {
        let (compute_uniform_buffer, vertex_buffer, triangle_buffer) = create_raytrace_compute_buffers(&device);

        let compute_bindgroup_layout = create_raytrace_compute_bindgroup_layout(&device);

        let compute_bindgroup = create_raytrace_compute_bindgroup(
            &device,
            &compute_bindgroup_layout,
            &screen_texture_view,
            &compute_uniform_buffer,
            &vertex_buffer,
            &triangle_buffer,
        );

        let compute_pipeline = create_raytrace_compute_pipeline(&device, &compute_bindgroup_layout);

        Self {
            uniform_buffer: compute_uniform_buffer,
            pipeline: compute_pipeline,
            bindgroup: compute_bindgroup,
            bindgroup_layout: compute_bindgroup_layout,
            vertex_buffer,
            triangle_buffer,
        }
    }

    pub fn run_compute_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32
    ) {
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
        vertices: &Vec<[f32; 3]>,
        triangle_data: &Vec<TriangleData>,
    ) -> Option<(u64, u64)> {
        let mut was_changed: bool = false;

        let current_vertex_buffer_size = self.vertex_buffer.size() as u64;
        let required_vertices_size = (size_of::<[f32; 3]>() * vertices.len()) as u64;

        let mut new_vertex_size: u64 = current_vertex_buffer_size;

        if(current_vertex_buffer_size < required_vertices_size) {
            new_vertex_size = required_vertices_size * 2;

            was_changed = true;
        }

        let current_triangle_buffer_size = self.triangle_buffer.size() as u64;
        let required_triangle_size = (size_of::<TriangleData>() * triangle_data.len()) as u64;

        let mut new_triangle_size: u64 = current_triangle_buffer_size;

        if(current_triangle_buffer_size < required_triangle_size) {
            new_triangle_size = required_triangle_size * 2;

            was_changed = true;
        }

        match was_changed {
            true => Some((new_vertex_size, new_triangle_size)),
            false => None
        }
    }

    pub fn rebind_buffers_size(
        &mut self,
        device: &wgpu::Device,
        new_vertex_size: u64,
        new_triangle_size: u64
    ) {
        self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
           label: Some("Vertex storage buffer"),
            size: new_vertex_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            // This is used to give a raw pointer to the mem on the gpu from the cpu
            // We dont need this at all
            // At least i think thats what it does, not sure
            mapped_at_creation: false,
        });

        self.triangle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Triangle storage buffer"),
            size: new_triangle_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    pub(crate) fn rebind_bindgroup(
        &mut self,
        device: &wgpu::Device,
        screen_texture: &wgpu::TextureView
    ) {
        self.bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bindgroup_layout,
            label: Some("Compute Bindgroup"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(screen_texture),
                },
                //
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.triangle_buffer.as_entire_binding(),
                },
            ],
        });
    }
}

pub struct RenderShader {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroup: wgpu::BindGroup,
    pub screen_texture_view: wgpu::TextureView,
}

impl RenderShader {
    pub fn new(device: &wgpu::Device, screen_texture_view: wgpu::TextureView, sampler: &wgpu::Sampler, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let render_bindgroup_layout = create_render_bindgroup_layout(&device);
        let render_bindgroup = create_render_bindgroup(&device, &render_bindgroup_layout, &screen_texture_view, &sampler);

        let render_pipeline = create_render_pipeline(&device, &surface_config, &render_bindgroup_layout);


        Self {
            bindgroup: render_bindgroup,
            pipeline: render_pipeline,
            screen_texture_view,
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