// Right now this file is in progress
// It just writes to the texture as red for every pixel
// I have it as red because that way I can know that it works

struct InstanceMesh {
    inverse_transformation_matrix: mat4x4,
    blas_entry: i32,
}

struct InstanceMeshWrapper {
    data: array<InstanceMesh>,
}

struct TriangleData {
    vertices: vec3<u32>,
    rgba: vec4<f32>,
}

struct TriangleDataWrapper {
    data: array<TriangleData>,
}

struct Vertex {
    x: f32,
    y: f32,
    z: f32,
}

struct VertexWrapper {
    data: array<Vertex>,
}

struct BvhNode {
	// min bounds = 48 bytes
	min_x: vec4<f32>,
	min_y: vec4<f32>,
	min_z: vec4<f32>,

	// max bounds = 48 bytes
	max_x: vec4<f32>,
	max_y: vec4<f32>,
	max_z: vec4<f32>,

	// indices = 16 bytes
	indices: vec4<i32>

	// 112 bytes, we pad to 128 for caching 
	_pad: vec4<u32>
}

struct BvhNodeWrapper {
    nodes: array<BvhNode>
}

struct Camera {

}

struct Ray {
    origin: vec3f,
    direction: vec3f,
}

fn default_ray() -> Ray {
    let ray = Ray(
        vec3<f32>(0.0, 0.0, 0.0,),
        vec3<f32>(0.0, 0.0, 0.0,),
    );

    return ray;
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform, read> camera_uniform: Camera;

@group(0) @binding(2)
var<storage, read> instance_storage: InstanceMeshWrapper;

@group(0) @binding(3)
var<storage, read> triangle_storage: TriangleDataWrapper;

@group(0) @binding(4)
var<storage, read> vertex_storage: VertexDataWrapper;

@group(0) @binding(5)
var<storage, read> tlas_storage: BvhNodeWrapper;

@group(0) @binding(6)
var<storage, read> blas_storage: BvhNodeWrapper;

@compute
@workgroup_size(8, 8, 1) // Workgroup size is just temporary for now, but 8x8 seems like a good standard
fn main(
  @builtin(global_invocation_id) global_id: vec3<u32> /* xy coordinate of pixel, we dont need the z */
) {
    let texture_dims = textureDimensions(output_texture); // We fetch the size of the texture to compare to our workgroup
    let texture_coord = vec2<u32>(global_id.xy);

    if(texture_coord.x >= texture_dims.x || texture_coord.y >= texture_dims.y) {
        return ();
    }

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




fn intersect_tri(ray_origin: vec3<f32>, ray_dir: vec3<f32>, tri: mat3x3) -> vec3<f32> /* Returns u, v, t*/ {
    /*
        Here, we use the moller trumbore algorithm

        To start, theh values we want back from this is the uv coordinate in the triangle,
        and the t variable, which is the distance along the ray, used for other shading

        First, lets start off by defining our intersection point as P.
        Using barycentric coordinates, we get P = wv0 + uv1 + vv2

        Since w + u + v = 1, we can rewrite as
        P = (1 - u - v)v0 + uv1 + vv2 = v0 + u(v1 - v0) + v(v2 - v0)

        Along with this definition of P, we also can define it using t.
        P = ray_origin + t * ray_dir

        Now, we can simply set these equal, getting
        ray_origin + t * ray_dir = v0 + u(v1 - v0) + v(v2 - v0)

        Using some algebra, we can simply group our unkowns on one side to get
        ray_origin - v0 = -t * ray_dir + u(v1 - v0) + v(v2 - v0)

        Now, notice that all our knowns are vec3's, and our unkowns are scalars.
        This means that we have enough data to make a systems of equations which is solvable.
        The method of solving is using Cramer's rule.

        See link for full explanation
        https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html
    */

    let v0 = tri[0];
    let v1 = tri[1];
    let v2 = tri[2];

    
    let E1 = v1 - v0;
    let E2 = v2 - v0;

    let T = ray_origin - v0;
    
    // XcY = X cross Y
    let DcE2 = cross(ray_dir, E2);
    let TcE1 = cross(T, E1);

    let det = dot(DcE2, E1);

    // This abs statement disables backface culling
    // For now, we have this here for development, but for a final release version, removing this saves a lot of performance
    // Note: This also requires us to check if our model correcty uses backface culling
    // We also have to make sure our algorithm on the cpu side keeps the order of vertices for backface culling
    if(abs(det) < 0.00001) {
        return vec3f(0.0, 0.0, -1.0)
    };

    let inv_det = 1.0 / det;

    let u = dot(DcE2, T) * inv_det;

    // We return early - we get less data, but its data that we dont need that saves time
    if(u < 0.0 || u > 1.0) {
        return vec3f(0.0, 0.0, -1.0)
    };

    let v = dot(TcE1, ray_dir) * inv_det;

    if(v < 0.0 || u + v > 1.0) {
        return vec3f(0.0, 0.0, -1.0)
    };

    let t = dot(TcE1, E2) * inv_det;

    return vec3f(u, v, t);
}