struct TriangleData {
    // Index of vertex
    vertices: vec3<u32>,
    // Color red green blue alpha
    color_rgba: vec4<f32>,
}

@group(0) @binding(2) var<storage, read> vertex_buffer: array<f32>;
@group(0) @binding(3) var<storage, read> triangle_data_buffer: array<TriangleData>;

// Take in a Ray
// Return a color for that ray's pixel
fn compute_ray_color(ray: Ray) -> vec3<f32> {
    return vec3<f32>(0.9, 0.2, 0.5);
}
