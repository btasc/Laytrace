use rayon::prelude::*;

use crate::gpu::buffers::{
    GpuStorageBlasLeafNode as BlasLeaf,
    GpuStorageBlasTreeNode as BlasTree,
    GpuStorageTriangleData as TriangleData,
    GpuStorageVertex as Vertex,
};

const DEFAULT_TRI_RGBA: [f32; 4] = [0.5, 0.5, 1.0, 1.0];

// I make this into a type so that if I ever decide to change it to like Vec<f32> or like Vec<[[f32; 3]; 3]> I can just change the type
// Its raw since we don't use our already made Vertex struct, its being taken directly from a file
pub type RawTriangleList = Vec<[f32; 9]>;

// This struct is used in gpu::buffers in GpuBuffers.write_bvh_res
pub struct BvhRes {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<TriangleData>,
    pub blas_tree: Vec<BlasTree>,
    pub blas_leaves: Vec<BlasLeaf>,
}

// This is our public entrypoint to building a BVH
// This takes all the work from starting at raw vertex list to a full write set for the buffers
pub fn build_mesh_bvh(raw_triangles: RawTriangleList) -> BvhRes {
    // First we calculate the centroids of all the triangles
    // We do this now because we still want access to the raw triangles
    // The next step takes the data, so this is our last chance to use a slice
    let centroids: Vec<[f32; 3]> = calculate_centroids(&raw_triangles);

    // We index the vertices and the triangles for memory saving
    let (vertices, triangles) = sort_sweep_vertices(raw_triangles);

    // ! Note - With how this function works, the indexes of the triangles are still matched with the raw ones
    // This means that the centroids are still accurate, even though they are made from different arrays
    // This also means that we can map attributes from other data besides the raw triangles to our indexed triangles
    // For example, later we can build some more advanced struct that holds all the tri data and match that

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

    BvhRes {
        vertices,
        triangles,

        blas_tree: vec![],
        blas_leaves: vec![],
    }
}

fn recurse_bvh(tris_idx: &[usize], triangles: &Vec<TriangleData>, vertices: &Vec<Vertex>) {

}

fn calculate_centroids(raw_triangles: &[[f32; 9]]) -> Vec<[f32; 3]> {
    raw_triangles.par_iter().map(|raw_tri| {
        [
            (raw_tri[0] + raw_tri[3] + raw_tri[6]) / 3.0,
            (raw_tri[1] + raw_tri[4] + raw_tri[7]) / 3.0,
            (raw_tri[2] + raw_tri[5] + raw_tri[8]) / 3.0,
        ]
    }).collect::<Vec<[f32; 3]>>()
}


// To index our raw vertices into indexed vertices, we use sort sweep
// This algorithm simply sorts the f32's, then groups together the similar ones

// Data structures we want
// Holds a vertex position and its original triangles position
struct VertexTriPos {
    coordinate: [f32; 3],
    tri_idx: usize,
}

fn sort_sweep_vertices(raw_triangles: RawTriangleList) -> (Vec<Vertex>, Vec<TriangleData>) {
    let tri_count = raw_triangles.len();

    // We sort it with rayon
    // In this case we use PDQSort, a rust provided method, sped up with rayon multithreading
    // We could use radix sort, but that would only be faster for massive meshes, and it is difficult to use with floats
    let mut indexed_vertices: Vec<Vertex> = Vec::new();

    let mut indexed_triangles: Vec<TriangleData> = (0..tri_count)
        .map(|_| TriangleData {
            vertices: [0, 0, 0],
            _pad: 0,
            rgba: DEFAULT_TRI_RGBA,
        })
        .collect();

    // This is used to keep track of how many vertices have been linked back to this triangle
    // At the end, it should be entirely a list of 3's
    let mut indexed_triangles_used_tally: Vec<u8> = vec![0; tri_count];

    let mut vertex_tri_pos_vec: Vec<VertexTriPos> = Vec::with_capacity(tri_count * 3);

    // Here we build the list of VertexTriPos-s to sort later
    for (original_tri_idx, raw_triangle) in raw_triangles.iter().enumerate() {
        for i in 0..3 {
            vertex_tri_pos_vec.push(VertexTriPos {
                coordinate: [
                    raw_triangle[i * 3],
                    raw_triangle[i * 3 + 1],
                    raw_triangle[i * 3 + 2],
                ],
                tri_idx: original_tri_idx,
            });
        }
    }

    // We use rayon par sort to multithread to speed up the process
    // We also use .total_cmp because it handles NANs and -0s safely, incase any issues are with the raw triangles
    // We use sort by because, with the struct, we have to have custom logic
    vertex_tri_pos_vec.par_sort_unstable_by(|a, b| {
        a.coordinate[0].total_cmp(&b.coordinate[0])
            .then_with(|| a.coordinate[1].total_cmp(&b.coordinate[1]))
            .then_with(|| a.coordinate[2].total_cmp(&b.coordinate[2]))
    });

    // Now we go through and fill out indexed_vertices
    // Tally of num of groups
    // This gives us our index to the current group of vertex
    let mut current_vertex_group_idx: u32 = 0;

    // We run our first pass for the first element
    // We make sure it's not empty before we do this. There are a lot of other ways to check, but it doesn't really matter
    if !vertex_tri_pos_vec.is_empty() {
        // We recreate the first variable to make the code easier to read
        let vertex_idx: usize = 0;

        let tri_idx = vertex_tri_pos_vec[vertex_idx].tri_idx;
        let current_pos = vertex_tri_pos_vec[vertex_idx].coordinate;

        // We push it since its the first
        indexed_vertices.push(Vertex::from_arr(current_pos));

        // We only have one path since its the first, so we skip all the equal logic
        let triangle_vertex_count = indexed_triangles_used_tally[tri_idx];
        indexed_triangles[tri_idx].vertices[triangle_vertex_count as usize] = current_vertex_group_idx;

        // We inc the tally since we used it
        indexed_triangles_used_tally[tri_idx] += 1;
    }

    // We start at one as we already handled the first element
    for vertex_idx in 1..vertex_tri_pos_vec.len() {
        let tri_idx = vertex_tri_pos_vec[vertex_idx].tri_idx;

        // We can safely index into previous pos as we start at 1
        let current_pos = vertex_tri_pos_vec[vertex_idx].coordinate;
        let previous_pos = vertex_tri_pos_vec[vertex_idx - 1].coordinate;

        let is_equal_pos: bool =
            current_pos[0] == previous_pos[0] &&
            current_pos[1] == previous_pos[1] &&
            current_pos[2] == previous_pos[2];

        // If we are on a new group of vertx, we inc the group idx and push that new vertex
        if !is_equal_pos {
            current_vertex_group_idx += 1;
            indexed_vertices.push(Vertex::from_arr(current_pos));
        }

        let triangle_vertex_count = indexed_triangles_used_tally[tri_idx];
        indexed_triangles[tri_idx].vertices[triangle_vertex_count as usize] = current_vertex_group_idx;

        // We inc the tally since we used it
        indexed_triangles_used_tally[tri_idx] += 1;
    }

    // This assertion checks that all the tallies are at 3
    // Since the tallies track the num of vertices attached to each triangle, something has gone wrong if a triangle doesnt have 3 vertices
    // We also keep this in debug just for better release performance, as this is the sort of thing that shouldn't fail assuming its done well
    #[cfg(debug_assertions)]
    {
        for tri_tally in indexed_triangles_used_tally {
            assert_eq!(tri_tally, 3, "Tri used tally is not fully set to 3 at end of loop. Assertion failed");
        }
    }

    (indexed_vertices, indexed_triangles)
}