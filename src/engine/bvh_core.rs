use std::convert;
use rayon::prelude::*;

use crate::gpu::buffers::{
    GpuStorageBlasLeafNode as BlasLeaf,
    GpuStorageBlasTreeNode as BlasTree,
    GpuStorageTriangleData as TriangleData,
    GpuStorageVertex as Vertex,
};

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
    // First, we create a hashmap with the vertices to reduce memory
    let (vertices, triangles) = sort_sweep_vertices(raw_triangles);

    BvhRes {
        vertices,
        triangles,

    }
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
    // We sort it with rayon
    // In this case we use PDQSort, a rust provided method, sped up with rayon multithreading
    // We could use radix sort, but that would only be faster for massive meshes, and it is difficult to use with floats
    let mut indexed_vertices: Vec<Vertex> = Vec::new();
    let mut indexed_triangles: Vec<TriangleData> = Vec::new();

    let mut vertex_tri_pos_vec: Vec<VertexTriPos> = Vec::new();
    vertex_tri_pos_vec.reserve(raw_triangles.len() * 3);

    for raw_triangle in raw_triangles.into_iter().enumerate() {
        for triple_tri_idx in 0..3 {
            // Goes through every vertex in the triangle
            let vertex: [f32; 3] = [
                raw_triangle.1[triple_tri_idx * 3],
                raw_triangle.1[triple_tri_idx * 3 + 1],
                raw_triangle.1[triple_tri_idx * 3 + 2],
            ];

            let original_tri_idx = raw_triangle.0;

            let vertex_tri_pos = VertexTriPos { coordinate: vertex, tri_idx: original_tri_idx };

            vertex_tri_pos_vec.push(vertex_tri_pos);
        }
    }

    // We use our custom compare floats to compare the triples
    vertex_tri_pos_vec.par_sort_unstable_by(|a, b| compare_floats(a.coordinate, b.coordinate));

    // Now we go through and fill out indexed_vertices
    // Tally of num of groups
    // This gives us our index to the current group of vertex
    let mut group_tally: usize = 0;

    for idx in 0..vertex_tri_pos_vec.len() {

        // We have 0 map to NAN so that it will always be equal to the current coordinate
        // See compare floats for this logic

        let previous_vertex_pos: [f32; 3] = match idx {
            0 => [f32::NAN, f32::NAN, f32::NAN],
            _ => vertex_tri_pos_vec[idx - 1].coordinate,
        };

        let current_vertex_pos: [f32; 3] = vertex_tri_pos_vec[idx].coordinate;

        match compare_floats(previous_vertex_pos, current_vertex_pos) {
            // Equal
            // We are in the same group, so we reuse our group_tally
            std::cmp::Ordering::Equal => (),
            // Not equal (either greater or less)
            // This means we are in a new group
            _ => {
                group_tally += 1;
                indexed_vertices.push(Vertex::from_arr(current_vertex_pos));
            },
        }


    }

    (indexed_vertices, indexed_triangles)
}

// Compare 3 floats lexigraphically
// Go float by float of the 3, returning equal if they all are the same
fn compare_floats(a: [f32; 3], b: [f32; 3]) -> std::cmp::Ordering {
    for i in 0..3 {
        // partial_cmp returns None if float is NaN.
        let ord = a[i].partial_cmp(&b[i]).unwrap_or(std::cmp::Ordering::Equal);

        // if they aren't equal, we return that
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    // If X, Y, and Z are all equal
    std::cmp::Ordering::Equal
}