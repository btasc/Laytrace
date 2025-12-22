// Right now this file is in progress
// It just writes to the texture as red for every pixel
// I have it as red because that way I can know that it works

struct Ray {
    // Position in x, y, z
    pos: vec3<f32>,

    // Momentum in dx, dy, dz
    mom: vec3<f32>,
}

fn default_ray() -> Ray {
    let ray = Ray(
        vec3<f32>(0.0, 0.0, 0.0,),
        vec3<f32>(0.0, 0.0, 0.0,),
    );

    return ray;
}

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(8, 8, 1) // Workgroup size is just temporary for now, but 8x8 seems like a good standard
fn main(
  @builtin(global_invocation_id) global_id: vec3<u32> /* xy coordinate of pixel, we dont need the z */
) {
    /*
        In our system, we use y as up and down, x as forward and backward, and z as back and forth
        Bascially, its a 2d grid with an added on 3d axis facing the viewer, positve for being closer to the viewer
    */
    let texture_coord = vec2<i32>(global_id.xy);
    let math_coord = vec2<f32>(global_id.xy);

    var pixel_color = vec3<f32>(1.0, 1.0, 1.0);

    let ray: Ray = get_ray_from_screen_coord(math_coord);

    // Just sets all pixels to red to test that this works
    pixel_color = compute_ray_color(ray);

    textureStore(
        output_texture,
        texture_coord,
        vec4<f32>(pixel_color, 1.0),
    );
}

fn get_ray_from_screen_coord(screen_coord: vec2<f32>) -> Ray {
    var ray: Ray = default_ray();
    ray.pos = uniform_params.camera_pos;

    // This should be the coordinate of the 2d coordinate but in 3d space relative to the camera
    // As said, this is relative, so its value is actually the same as the momentum
    var screen_3d_coord = vec3<f32>(0.0, 0.0, 0.0);

    // We just have 1 for now to symbolize the camera facing forward
    // This means that our camera's coordinates should be negative to look forward at some object at 0, 0, 0
    screen_3d_coord = vec3<f32>(screen_coord.x, screen_coord.y, 1);

    ray.mom = screen_3d_coord;

    return ray;
}

fn point_distance(p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    return sqrt(
        (p1.x - p2.x) * (p1.x - p2.x) +
        (p1.y - p2.y) * (p1.y - p2.y)
    );
}