use rayon::prelude::*;

use crate::gpu::buffers::{
    GpuStorageBlasLeafNode as BlasLeaf,
    GpuStorageBlasTreeNode as BlasTree,
    GpuStorageTriangleData as TriangleData,
    GpuStorageVertex as Vertex,
};

use glam::{
    Vec3, Vec3A,
    UVec3,
};

const DEFAULT_TRI_RGBA: [f32; 4] = [0.5, 0.5, 1.0, 1.0];

// I make this into a type so that if I ever decide to change it I have a premade struct
pub type RawTriangleList = Vec<[Vec3A; 3]>;

// This struct is used in gpu::buffers in GpuBuffers.write_bvh_res
pub struct BvhRes {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<TriangleData>,
    pub blas_tree: Vec<BlasTree>,
    pub blas_leaves: Vec<BlasLeaf>,
}

struct TempTriangle {
    vertices: [usize; 3],
    centroid: Vec3A,
    min_bounds: Vec3A,
    max_bounds: Vec3A,
}

// This is our public entrypoint to building a BVH
// This takes all the work from starting at raw vertex list to a full write set for the buffers
pub fn build_mesh_bvh(raw_triangles: RawTriangleList) -> BvhRes {
    // We index the vertices and the triangles for memory saving
    let (vertices, triangles): (Vec<Vec3A>, Vec<TempTriangle>) = sort_sweep_vertices(raw_triangles);

    // Now we use those to actually build our BVH
    // Remember that for a bvh's children, its children our the index, and its index plus one
    let blas_tree: Vec<BlasTree> = Vec::new();
    let blas_leaves: Vec<BlasLeaf> = Vec::new();

    // For this bvh recursion setup, I make a massive list of every tri index
    // I then pass this into our bvh, where it recurses, splitting this list over and over
    // Eventually, it should return our list of branches and leaves
    // * we also give it our centroids
    let tri_index_list = (0..triangles.len()).collect::<Vec<usize>>();

    // todo past this

    // Convert the temp things into the gpu ready setup, and then return it
    convert_to_bvh_res(vertices, triangles, blas_tree, blas_leaves)
}

fn recurse_bvh(tris_idx: Vec<usize>, triangles: &Vec<TempTriangle>, vertices: &Vec<Vertex>) -> (Vec<BlasTree>, Vec<BlasLeaf>) {

    // First, check
}

fn convert_to_bvh_res(vertices: Vec<Vec3A>, triangles: Vec<TempTriangle>, blas_tree: Vec<BlasTree>, blas_leaves: Vec<BlasLeaf>) -> BvhRes {
    let gpu_vertices: Vec<Vertex> = vertices.into_iter().map(|v| {
        Vertex {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }).collect();

    let gpu_triangle_data: Vec<TriangleData> = triangles.into_iter().map(|temp_tri| {
        let tri_indices: [u32; 3] = [temp_tri.vertices[0] as u32, temp_tri.vertices[1] as u32, temp_tri.vertices[2] as u32];

        TriangleData {
            vertices: tri_indices,
            _pad: 0,
            // todo Right now, we just make it into the default triangle, but eventually we will add the other info data here
            rgba: DEFAULT_TRI_RGBA,
        }
    }).collect();

    BvhRes {
        vertices: gpu_vertices,
        triangles: gpu_triangle_data,
        blas_tree: blas_tree,
        blas_leaves: blas_leaves,
    }
}

// To index our raw vertices into indexed vertices, we use sort sweep
// This algorithm simply sorts the f32's, then groups together the similar ones

// Data structures we want
// Holds a vertex position and its original triangles position
struct VertexTriIdx {
    coordinate: Vec3A,
    tri_idx: usize,
}

fn sort_sweep_vertices(raw_triangles: RawTriangleList) -> (Vec<Vec3A>, Vec<TempTriangle>) {
    let tri_count = raw_triangles.len();

    // We sort it with rayon
    // In this case we use PDQSort, a rust provided method, sped up with rayon multithreading
    // We could use radix sort, but that would only be faster for massive meshes, and it is difficult to use with floats
    let mut indexed_vertices: Vec<Vec3A> = Vec::new();

    // Calculate the centroids
    let centroids: Vec<Vec3A> = calculate_centroids(&raw_triangles);

    // Create a blank set of triangles
    let (mut indexed_triangles, mut triangle_vertex_count): (Vec<TempTriangle>, Vec<u8>) = (0..tri_count)
        .into_iter()
        .map(|centroid| {
            (TempTriangle {
                vertices: [0, 0, 0],
                centroid: Vec3A::NAN,
                min_bounds: Vec3A::NAN,
                max_bounds: Vec3A::NAN
            }, 0)
        }).collect();

    let mut vertex_tri_idx_vec = Vec::with_capacity(tri_count * 3);

    // Here we build the list of VertexTriPos-s to sort later
    for (original_tri_idx, raw_triangle) in raw_triangles.iter().enumerate() {
        for i in 0..3 {
            vertex_tri_idx_vec.push(VertexTriIdx {
                coordinate: raw_triangle[i],
                tri_idx: original_tri_idx,
            });
        }
    }

    // We use rayon par sort to multithread to speed up the process
    // We also use .total_cmp because it handles NANs and -0s safely, incase any issues are with the raw triangles
    // We use sort by because, with the struct, we have to have custom logic
    vertex_tri_idx_vec.par_sort_unstable_by(|a, b| {
        // We go lexigraphically, using x, then y, then z
        a.coordinate[0].total_cmp(&b.coordinate[0])
            .then_with(|| a.coordinate[1].total_cmp(&b.coordinate[1]))
            .then_with(|| a.coordinate[2].total_cmp(&b.coordinate[2]))
    });

    // Now we go through and fill out indexed_vertices
    // Tally of num of groups
    // This gives us our index to the current group of vertex
    let mut current_vertex_group_idx: usize = 0;

    // We run our first pass for the first element
    // We make sure it's not empty before we do this. There are a lot of other ways to check, but it doesn't really matter
    if !vertex_tri_idx_vec.is_empty() {
        // We recreate the first variable to make the code easier to read
        let vertex_idx: usize = 0;

        let tri_idx = vertex_tri_idx_vec[vertex_idx].tri_idx;

        // We push it since its the first
        indexed_vertices.push(vertex_tri_idx_vec[vertex_idx].coordinate);

        // We only have one path since its the first, so we skip all the equal logic
        let v_tri_count = triangle_vertex_count[tri_idx];
        indexed_triangles[tri_idx].vertices[v_tri_count as usize] = current_vertex_group_idx;

        // We inc the tally since we used it
        triangle_vertex_count[tri_idx] += 1;
    }

    // We start at one as we already handled the first element
    for vertex_idx in 1..vertex_tri_idx_vec.len() {
        let tri_idx = vertex_tri_idx_vec[vertex_idx].tri_idx;

        // We can safely index into previous pos as we start at 1
        let current_pos = vertex_tri_idx_vec[vertex_idx].coordinate;
        let previous_pos = vertex_tri_idx_vec[vertex_idx - 1].coordinate;

        let is_equal_pos: bool = current_pos == previous_pos;

        // If we are on a new group of vertx, we inc the group idx and push that new vertex
        if !is_equal_pos {
            current_vertex_group_idx += 1;
            indexed_vertices.push(current_pos);
        }

        let vertex_tri_idx = triangle_vertex_count[tri_idx];
        indexed_triangles[tri_idx].vertices[vertex_tri_idx as usize] = current_vertex_group_idx;

        // We inc the tally since we used it
        triangle_vertex_count[tri_idx] += 1;
    }

    // This assertion checks that all the tallies are at 3
    // Since the tallies track the num of vertices attached to each triangle, something has gone wrong if a triangle doesnt have 3 vertices
    // We also keep this in debug just for better release performance, as this is the sort of thing that shouldn't fail assuming its done well
    #[cfg(debug_assertions)]
    {
        for tri_tally in triangle_vertex_count.iter() {
            assert_eq!(*tri_tally, 3, "Debug: Tri used tally is not fully set to 3 at end of loop. Assertion failed");
        }
    }

    // Go through and get the bounds boxes made and calculate the centroids
    for tri in &mut indexed_triangles {
        let vertices = tri.vertices.map(|idx| indexed_vertices[idx]);
        tri.centroid = (vertices[0] + vertices[1] + vertices[2]) / 3.0;

        // .min takes the min element of each field and compares it with the other Vec3A, perfect for this
        tri.min_bounds = vertices[0].min(vertices[1]).min(vertices[2]);
        tri.max_bounds = vertices[0].max(vertices[1]).max(vertices[2]);
    }

    


    (indexed_vertices, indexed_triangles)
}