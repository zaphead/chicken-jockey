//! wgpu renderer, chunk meshing, and render-world extraction.

mod camera;
mod extract;
mod mesh;
mod pipeline;
mod renderer;
mod world_mesh;

pub use camera::Camera;
pub use extract::{RenderExtractState, RenderSurfaceInfo, RenderWorld};
pub use mesh::{append_face, MeshVertex, SolidMesh};
pub use renderer::Renderer;
pub use world_mesh::{
    extract_render_scene, mesh_chunk, ChunkMeshCache, RebuildBudget, RenderScene,
    MAX_CHUNK_REBUILDS_PER_FRAME,
};

/// Max distance from the camera to build and retain chunk meshes (world units).
pub const CHUNK_MESH_RENDER_DISTANCE: f32 = 192.0;
