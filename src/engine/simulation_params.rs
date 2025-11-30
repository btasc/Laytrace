
// This is what is actually passed into the compute shader
// This is in engine and not in gpu to make it easier to change something
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationParams {

}