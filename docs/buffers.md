# Buffers

Note 1:   
For indices that are of type i32, assume that they are an option between 2 different indices. If the index is 0 or greater, then simply use the index as normal. If the index is \-1 or less, follow bitwise NOT (\!) to get its secondary index value. This is equivalent to adding one and flipping its sign as shown in the examples.

Example:

```rust
-1 => (-1 + 1) * -1 => 0,
3 => 3,
-4 => (-4 + 1) * -1 => 3,
```

Note 2:   
When using storage buffers (std430), and specifically when making a vec3 / \[f/u/i32; 3\], make sure the value always starts on a byte multiple of 16\. For simplicity, always make sure a 16 byte alignment starts on a 16 byte. The easiest way to do this is to declare these at the beginning of the struct, but extra padding is needed if there are multiple of these. When all data is placed, add up the total memory and add end padding to get it to a multiple of 16

Example:

```rust
struct UnsafeStorage {
	data1: f32,
	data_arr: [f32; 3],
	data2: f32,
}

// Must be converted to

struct SafeStorage {
	// We start at the beginning, guaranteeing 16 byte alignment assuming our struct is built correctly
	data_arr: [f32; 3],

	// We don't have padding after, as we only have 4 byte scalar values left
	data1: f32,
	data2: f32,

	// Finally, we pad the end to get the total bytes (3 * 4 + 2 * 4 = 20 bytes) to 32 bytes
	_pad: [f32; 3],
}
```

Note 3: 
For parsing the TLAS and BLAS trees, we use the convention of the i32 index being its first child, and that index increment being the second child.

Example:

```rust
TlasNode : { index: 1 } -> TlasBuffer[1] && TlasBuffer[2]
```

## Mesh data

Note:  
	There used to be another storage buffer that held the mesh, however eventually all of its components were abstracted out. Now, the instance mesh directly holds the BLAS entry point, and the BLAS leaves hold the references to the triangles of the mesh.

#### instance\_mesh\_buffer: Vec\<GpuStorageInstanceMesh\>

- Buffer that holds individual instances of some mesh  
- References back to a BLAS entry point, from which can be traversed to get the rest of the mesh data. The inverse transformation matrix is used to move the ray to the local space of the mesh

```rust
struct GpuStorageInstanceMesh {
	// Inverse of the matrix that transforms the origin mesh to the mesh / entry to the BLAS
	// We precalculate the inverse on the cpu as to not waste gpu resources
	inverse_transformation_matrix: [f32; 16],

	// BLAS entry point
	// See note 1 and 3
	// Negative value points to a BLAS leaf, positive value points to a branch
	blas_entry: i32,

	// See note 2
	_pad: [u32; 3],
}
```

#### triangle\_data\_buffer: Vec\<GpuStorageTriangleData\>

- Buffer that holds all the raw information of each triangle, such as lighting or texture information. Referenced from the BLAS leaf nodes

```rust
struct GpuStorageTriangleData {
	// Vertex indices
	vertices: [u32; 3],

	// See note 2
	_pad: u32,

	// Texture data
	rgba: [f32; 4],

	// TBD on other data
}
```

#### vertex\_buffer: Vec\<GpuStorageVertex\>

- Buffer of every raw vertex. Referenced from GpuStorageTriangleData. We use our own struct to make the storage buffer pack tightly at 12 bytes, and as to save performance from more complex indexing with a Vec\<f32\>  
- Note: These cannot be converted into a vec3 in WGSL, they must stay in their own xyz struct

```rust
struct GpuStorageVertex {
	x: f32,
	y: f32,
	z: f32,
}
```

## BVH Buffers

#### tlas\_buffer: Vec\<GpuStorageTlasNode\>

- Holds all nodes for the TLAS BVH tree  
- **\! First (index 0\) node is the entry node for scene**

```rust
struct GpuStorageTlasNode {
	// Two bounds for the box
	min_bound: [f32; 3],
	
	// See note 2
	_pad: u32,

	max_bound: [f32; 3],
	
	// See note 1. Positive value leads to another node, negative value leads to the instance buffer
	node_or_instance: i32,
}
```

#### blas\_tree\_buffer: Vec\<GpuStorageBlasTreeNode\>

- Holds a local BVH tree for each mesh.   
- The child\_node is an i32. If its negative minus 1, its the index of a leaf, if its positive, its the index of a branch  
- **\! Reference the instance mesh to find the starting node**

```rust
struct GpuStorageBlasTreeNode{
	// Two bounds for the box
	min_bound: [f32; 3],

	// See note 2
	_pad: u32,

	max_bound: [f32; 3],
	
	// See note 1 and note 3. Positive leads to another node, negative leads to a leaf
	node_or_leaf: i32,
}
```

#### blas\_leaf\_buffer: Vec\<GpuStorageBlasLeafNode\>

- Holds a local leaf with 8 triangles for the origin mesh

```rust
struct GpuStorageBlasLeafNode {
	// Indices for the 8 triangles in the box
	triangles: [u32; 8],

	// Triangle count, 0-8
	// If triangle count is less than 8, assume all triangles past are junk
	triangle_count: u32,

	// Note 2
	_pad: [u32; 3],
}
```

Note: Eventually, I plan to implement a buffer / compute shader system for transforming the instance meshes, but for now that is still handled on the cpu manually.

## Rendering Buffers

#### camera\_uniform\_buffer: GpuUniformCamera

- Uniform buffer  
- Holds all the camera data 

```rust
struct GpuUniformCamera {
	// TBD on what will be included here yet, the coordinate system still has to be fleshed out a bit 
}
```

