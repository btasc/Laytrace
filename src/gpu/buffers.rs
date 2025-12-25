use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

// Since wgpu::Buffer is a ref count, we can just derive clone
#[derive(Clone)]
pub struct GpuBuffers {
    // Model data
    pub(crate) instance_mesh_buffer: wgpu::Buffer,
    pub(crate) triangle_data_buffer: wgpu::Buffer,
    pub(crate) vertex_buffer: wgpu::Buffer,

    // BVH data
    pub(crate) tlas_buffer: wgpu::Buffer,
    pub(crate) blas_tree_buffer: wgpu::Buffer,
    pub(crate) blas_leaf_buffer: wgpu::Buffer,

    // Rendering data
    pub(crate) camera_uniform_buffer: wgpu::Buffer,
}

impl GpuBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let instance_mesh_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageInstanceMesh>() as u64, "Instance Mesh Storage Buffer"
        );

        let triangle_data_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageTriangleData>() as u64, "Triangle Data Storage Buffer"
        );

        let vertex_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageVertex>() as u64, "Vertex Storage Buffer"
        );

        let tlas_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageTlasNode>() as u64, "TLAS Storage Buffer"
        );

        let blas_tree_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageBlasTreeNode>() as u64, "BLAS Tree Storage Buffer"
        );

        let blas_leaf_buffer = Self::create_storage_buffer(
            &device, size_of::<GpuStorageBlasLeafNode>() as u64, "BLAS Leaf Storage Buffer"
        );

        let camera_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size: size_of::<GpuUniformCamera>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            instance_mesh_buffer, triangle_data_buffer,
            vertex_buffer, tlas_buffer, blas_tree_buffer,
            blas_leaf_buffer, camera_uniform_buffer,
        }
    }

    fn create_storage_buffer(
        device: &wgpu::Device,
        size_of_struct: u64,
        label: &'static str,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size_of_struct,
            
            // Storage buffer specific descriptors
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}



// Code of buffers taken directly from docs, see docs for any referenced notes
// Docs located at ~docs/buffers.md

// Instance Mesh Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuStorageInstanceMesh {
	// Inverse of the matrix that transforms the origin model to the world model
	// We precalculate the inverse on the cpu as to not waste gpu resources
	inverse_transformation_matrix: [f32; 16],

	// BLAS entry point
	// See note 1 and 3
	// Negative value points to a BLAS leaf, positive value points to a branch
	blas_entry: i32,

	// See note 2
	_pad: [u32; 3],
}

// Triangle Data Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuStorageTriangleData {
	// Vertex indices
	vertices: [u32; 3],

	// See note 2
	_pad: u32,

	// Texture data
	rgba: [f32; 4],

	// TBD on other data
}

// Vertex Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuStorageVertex {
	x: f32,
	y: f32,
	z: f32,
}

// TLAS Node Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuStorageTlasNode {
	// Two bounds for the box
	min_bound: [f32; 3],
	
	// See note 2
	_pad: u32,

	max_bound: [f32; 3],
	
	// See note 1. Positive value leads to another node, negative value leads to the instance buffer
	node_or_instance: i32,
}

// BLAS Tree Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuStorageBlasTreeNode{
	// Two bounds for the box
	pub min_bound: [f32; 3],

	// See note 2
	pub _pad: u32,

	pub max_bound: [f32; 3],
	
	// See note 1 and note 3. Positive leads to another node, negative leads to a leaf
	pub node_or_leaf: i32,
}

// BLAS Leaf Buffer
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuStorageBlasLeafNode {
	// Indices for the 8 triangles in the box
	pub triangles: [u32; 8],

	// Triangle count, 0-8
	// If triangle count is less than 8, assume all triangles past are junk
	pub triangle_count: u32,

	// Note 2
	pub _pad: [u32; 3],
}

// Camera Uniform Buffer
// Uniforms have to be multiples of 16, so we align 16
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuUniformCamera {
    pub pos: [f32; 3],
    _pad: u32,

	// TBD on what will be included here yet, the coordinate system still has to be fleshed out a bit 
}

impl Default for GpuUniformCamera {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            _pad: 0,
        }
    }
}