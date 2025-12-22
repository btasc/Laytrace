use super::pipelines::{
    create_raytrace_compute_pipeline,
    create_render_pipeline,
    create_transform_compute_pipeline,
};

use super::buffers::GpuBuffers;

pub struct ComputeRaytraceShader {
    pub pipeline: wgpu::ComputePipeline,
    pub bindgroup: wgpu::BindGroup,
    pub bindgroup_layout: wgpu::BindGroupLayout,
}

impl ComputeRaytraceShader {
    pub fn new(
        device: &wgpu::Device,
        screen_texture_view: &wgpu::TextureView,
    ) -> Self {
        let compute_pipeline = create_raytrace_compute_pipeline(&device, &bindgroup_layout);

        Self {
            pipeline,
            bindgroup,
            bindgroup_layout,
        }
    }

    pub fn run_compute_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        buffers: &GpuBuffers,
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