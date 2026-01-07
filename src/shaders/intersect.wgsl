

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

        To do this, we 
    */
}