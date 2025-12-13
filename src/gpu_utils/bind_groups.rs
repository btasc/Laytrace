use crate::engine::params::{GpuUniformParams, TriangleData, TriangleWorkUniformParams};

use std::mem::size_of;
use bytemuck;

pub fn create_render_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Bind Group Layout"),
        entries: &[
            // The texture view
            wgpu::BindGroupLayoutEntry {
                binding: 0, // @binding(0)
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // The sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1, // @binding(1)
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    };

    device.create_bind_group_layout(&bind_group_layout_desc)
}

pub fn create_render_bindgroup(
    device: &wgpu::Device,
    render_bindgroup_layout: &wgpu::BindGroupLayout,
    screen_texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler
) -> wgpu::BindGroup {
    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Render Bind Group"),
        layout: &render_bindgroup_layout,
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
    });

    render_bind_group
}

pub fn create_raytrace_compute_bindgroup_layout(
    device: &wgpu::Device
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Raytrace Compute Bind Group Layout"),
        entries: &[
            // Uniform buffer that holds all the small data like camera position and other simple stuff
            wgpu::BindGroupLayoutEntry {
                binding: 0, // @binding(0)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(size_of::<GpuUniformParams>() as u64),
                },
                count: None,
            },

            // Buffer that holds the texture that we write to for the fragment shader
            wgpu::BindGroupLayoutEntry {
                binding: 1, // @binding(1)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },

            // Our buffer that holds all the vertex information
            // ! Even though a vertex is [f32; 3], it needs to be [f32; 4] cus wgpu padding !
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    // Read only since we just want to read the vertices, not edit them
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    // None because vec size can change
                    min_binding_size: None,
                },
                count: None,
            },

            // This buffer holds all the triangle info
            // It's the same as our vertex one because its unsized, so It's pretty ambiguous what it holds
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    // Read only since we just want to read the vertices, not edit them
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    // None because vec size can change
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

pub fn create_raytrace_compute_bindgroup(
    device: &wgpu::Device,
    compute_bindgroup_layout: &wgpu::BindGroupLayout,
    screen_texture: &wgpu::TextureView,
    uniform_buffer: &wgpu::Buffer,
    vertex_buffer: &wgpu::Buffer,
    triangle_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Raytrace Compute Bindgroup"),
        layout: &compute_bindgroup_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(screen_texture),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: triangle_buffer.as_entire_binding(),
            },
        ]
    });

    compute_bind_group
}

pub fn create_raytrace_compute_buffers(
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let uniform_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Raytrace Uniform Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let vertex_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Raytrace Vertex Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let triangle_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Raytrace Triangle Data Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    (uniform_buffer, vertex_buffer, triangle_buffer)
}

pub fn create_transform_compute_bindgroup_layout(
    device: &wgpu::Device
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Compute Bind Group Layout"),
        entries: &[
            // Uniform buffer that holds the information the shader needs to decide where to send resources to do stuff
            wgpu::BindGroupLayoutEntry {
                binding: 0, // @binding(0)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(size_of::<TriangleWorkUniformParams>() as u64),
                },
                count: None,
            },

            // Holds the work orders
            wgpu::BindGroupLayoutEntry {
                binding: 1, // @binding(1)
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    // We only want to read
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    // Vec size can change, so i belive this should be false
                    // Not entirely sure
                    min_binding_size: None,
                },
                count: None,
            },

            // We literally just keep these the same except for read_only: false
            // They are just the same buffers from the raytrace one
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    // Read only on false for editing
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    // None because vec size can change
                    min_binding_size: None,
                },
                count: None,
            },

            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    // None because vec size can change
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

pub fn create_transform_compute_bindgroup(
    device: &wgpu::Device,
    compute_bindgroup_layout: &wgpu::BindGroupLayout,
    uniform_descriptor_buffer: &wgpu::Buffer,
    order_buffer: &wgpu::Buffer,
    vertex_buffer: &wgpu::Buffer,
    triangle_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bindgroup"),
        layout: &compute_bindgroup_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_descriptor_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: order_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: triangle_buffer.as_entire_binding(),
            }
        ]
    });

    compute_bind_group
}

pub fn create_transform_compute_buffers(
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::Buffer) {
    let uniform_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Transform Uniform Buffer"),
            size: size_of::<TriangleWorkUniformParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let triangle_order_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Transform Order Buffer"),
            size: 1024,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    (uniform_buffer, triangle_order_buffer)
}