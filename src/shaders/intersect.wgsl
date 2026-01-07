

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
    if(abs(det) < 0.00001) return vec3f(0.0, 0.0, -1.0);

    let inv_det = 1.0 / det;

    let u = dot(DcE2, T) * inv_det;
    if(u < 0.0 || u > 1.0) return vec3f(0.0, 0.0, -1.0);

    let v = dot(TcE1, ray_dir) * inv_det;
    if(v < 0.0 || u + v > 1.0) return vec3f(0.0, 0.0, -1.0);

    let t = dot(TcE1, E2) * inv_det;

    return vec3f(u, v, t);
}