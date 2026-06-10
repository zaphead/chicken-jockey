use crate::camera::Camera;
use crate::mesh::SolidMesh;
use crate::world_mesh::ChunkMeshCache;

#[derive(Debug, Default)]
pub struct RenderWorld {
    pub camera: Camera,
    pub meshes: Vec<SolidMesh>,
    pub ready: bool,
}

#[derive(Default)]
pub struct RenderExtractState {
    pub mesh_cache: ChunkMeshCache,
    pub world_mesh_queue: Vec<glam::IVec3>,
    pub terrain_bootstrapped: bool,
    /// Cleared after the first full rebuild following terrain bootstrap.
    pub pending_full_rebuild: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderSurfaceInfo {
    pub aspect: f32,
}
