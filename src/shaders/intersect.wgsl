

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

    
    // This code is very unoptimized, right now i made it more to just understand the theory
    // I will go back and use a more standard approach later, but this works
    // We organize it as u, v, t, the same as our return order
    let var_consts_mat3: mat3x3 = mat3x3((v1 - v0), (v2 - v0), -1 * ray_dir);
    let knowns: vec3 = ray_origin - v0;

    let det = determinant(var_consts_mat3);

    // Ray is parralel to tri
    if(abs(det) < 0.00001) {
        return vec3f(0.0, 0.0, -1.0);
    }

    let matx = mat3x3(knowns, var_consts_mat3[1], var_consts_mat3[2]);
    let maty = mat3x3(var_consts_mat3[0], knowns, var_consts_mat3[2]);
    let matz = mat3x3(var_consts_mat3[0], var_consts_mat3[1], knowns);

    let u = determinant(matx) / det;
    let v = determinant(maty) / det;
    let t = determinant(matz) / det;

    if (u < 0.0 || u > 1.0 || v < 0.0 || u + v > 1.0 || t < 0.0) {
        return vec3f(0.0, 0.0, -1.0);
    }

    return vec3f(u, v, t);
}