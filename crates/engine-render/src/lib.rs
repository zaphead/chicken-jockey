//! wgpu renderer, chunk meshing, and render-world extraction.

mod camera;
mod compute_mesh;
mod extract;
mod mesh;
mod pipeline;
mod render_submit;
mod renderer;
mod world_mesh;

pub use camera::Camera;
pub use compute_mesh::ComputeMesher;
pub use extract::{RenderExtractState, RenderSurfaceInfo, RenderWorld};
pub use mesh::{cube_mesh, MeshVertex, SolidMesh};
pub use renderer::Renderer;
pub use world_mesh::{
    ChunkMeshCache, RenderScene, extract_render_scene, mesh_chunk, MAX_CHUNK_REBUILDS_PER_FRAME,
};

/// Screen-space LOD distance for chunk mesh generation (world units).
pub const CHUNK_MESH_LOD_DISTANCE: f32 = 192.0;
