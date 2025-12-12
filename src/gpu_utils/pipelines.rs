pub const RAYTRACE_COMPUTE_WGSL: &'static str = concat!(
    include_str!("../shaders/main.wgsl"), "\n\n",
    include_str!("../shaders/collision.wgsl"),
);

pub const TRANSFORM_COMPUTE_WGSL: &'static str = include_str!("../shaders/transformer.wgsl");

pub const RENDER_WGSL: &'static str = include_str!("../shaders/blit.wgsl");

pub fn create_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    render_bindgroup_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fullscreen Shader"),
        source: wgpu::ShaderSource::Wgsl(RENDER_WGSL.into()),
    });

    let vertex_state = wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
    };

    let fragment_state = wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),

        targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })],
    };

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&render_bindgroup_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Fullscreen Render Pipeline"),
        layout: Some(&render_pipeline_layout),

        vertex: vertex_state,
        fragment: Some(fragment_state),

        depth_stencil: None,
        multiview: None,
        cache: None,

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },

        multisample: wgpu::MultisampleState::default(),
    });

    render_pipeline
}

pub fn create_raytrace_compute_pipeline(
    device: &wgpu::Device,
    compute_bindgroup_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Raytrace Compute Shader Module"),
        source: wgpu::ShaderSource::Wgsl(RAYTRACE_COMPUTE_WGSL.into()),
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Raytrace Compute Pipeline Layout"),
        bind_group_layouts: &[&compute_bindgroup_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Raytrace Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: Some("main"), // The function to call in compute.wgsl
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    compute_pipeline
}

pub fn create_transform_compute_pipeline(
    device: &wgpu::Device,
    compute_bindgroup_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Transform Compute Shader Module"),
        source: wgpu::ShaderSource::Wgsl(TRANSFORM_COMPUTE_WGSL.into()),
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Transform Compute Pipeline Layout"),
        bind_group_layouts: &[&compute_bindgroup_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Transform Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: Some("main"), // The function to call in compute.wgsl
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    compute_pipeline
}