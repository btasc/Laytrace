pub const COMPUTE_WGSL: &'static str = concat!(
    include_str!("../shaders/header.wgsl"), "\n\n",
    include_str!("../shaders/collision_bvh.wgsl"), "\n\n",
    include_str!("../shaders/tracer.wgsl"),
);

pub const RENDER_WGSL: &'static str = include_str!("../shaders/blit.wgsl");
