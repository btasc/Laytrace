use super::init_utils::TEXTURE_FORMAT;
use super::buffers::{ GpuUniformCamera, GpuBuffers };

fn create_compute_layout_entry(read_only: bool, binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: read_only }, 
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub fn create_raytrace_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Raytrace Bindgroup Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,

                    // We import and reuse the constant from gpu_utils
                    format: TEXTURE_FORMAT,

                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },

            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GpuUniformCamera>() as u64),
                },
                count: None,
            },

            // Buffers are made in order as mentioned in docs/buffers.md
            // except for the texture view and the camera

            // Instances
            // ! true = read_only
            create_compute_layout_entry(true, 2),
            
            // Triangle Data
            create_compute_layout_entry(true, 3),

            // Vertices
            create_compute_layout_entry(true, 4),

            // TLAS
            create_compute_layout_entry(true, 5),

            // BLAS
            create_compute_layout_entry(true, 6),
        ],
    })
}

pub fn create_raytrace_bindgroup(
    device: &wgpu::Device, 
    buffers: &GpuBuffers,
    screen_texture: &wgpu::TextureView,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: Some("Raytrace Bindgroup"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(screen_texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.camera_uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.instance_mesh_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: buffers.triangle_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: buffers.vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: buffers.tlas_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: buffers.blas_buffer.as_entire_binding(),
            }
        ],
    })
}

pub fn create_render_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Bindgroup Layout"),
        entries: &[
            // Texture view
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },

            // Sampler for the texture
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub fn create_render_bindgroup(
    device: &wgpu::Device,
    bindgroup_layout: &wgpu::BindGroupLayout,
    screen_texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Render Bind Group"),
        layout: &bindgroup_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(screen_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}