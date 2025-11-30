// Code from previous project, not implemented yet todo!
// !-- Vertex Shader Code --! //
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// An array of the four corner positions of the screen quad.
const POSITIONS = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, 1.0),  // top-left
    vec2<f32>(-1.0, -1.0), // bottom-left
    vec2<f32>(1.0, 1.0),   // top-right
    vec2<f32>(1.0, -1.0),  // bottom-right
);

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let pos = POSITIONS[idx];

    // Pos y is flipped because it is the distance down, not like a coordinate grid y
    let uv = vec2<f32>(
        (pos.x + 1.0) * 0.5, // (pos.x * 0.5) + 0.5
        (1.0 - pos.y) * 0.5  // (pos.y * -0.5) + 0.5
    );

    return VertexOutput(vec4<f32>(pos, 0.0, 1.0), uv);
}

// !-- Fragment Shader Code --! //
@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var my_sampler: sampler;

@fragment
fn fs_main(@location(0) in_uv: vec2<f32>) -> @location(0) vec4<f32> {
    // Function to sample the texture using the provided sampler at the UV coordinate
    // Just goes through and takes the pixel color and samples it from the texture
    let final_color = textureSample(input_texture, my_sampler, in_uv);

    return final_color;
}