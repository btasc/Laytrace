use wgpu::util::DeviceExt;
use crate::engine::params::{GpuUniformParams, TriangleData};

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

pub fn create_compute_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute Bind Group Layout"),
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

pub fn create_compute_bindgroup(
    device: &wgpu::Device,
    compute_bindgroup_layout: &wgpu::BindGroupLayout,
    screen_texture: &wgpu::TextureView,
    uniform_buffer: &wgpu::Buffer,
    vertex_buffer: &wgpu::Buffer,
    triangle_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Compute Bindgroup"),
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

pub fn create_buffers(device: &wgpu::Device, params: &GpuUniformParams, vertices: &Vec<[f32; 3]>, triangle_data: &Vec<TriangleData>) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let uniform_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(params),
            // UNIFORM marks this as special, and COPY_DST lets us update the buffer
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            // We dont need to do .as_slice because its a reference
            contents: bytemuck::cast_slice(vertices),
            // STORAGE marks it as a storage buffer, and copy_dst lets is change it
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }
    );

    let triangle_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Data Buffer"),
            // Same as the vertex
            contents: bytemuck::cast_slice(triangle_data),
            // STORAGE marks it as a storage buffer, and copy_dst lets is change it
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }
    );

    (uniform_buffer, vertex_buffer, triangle_buffer)
}