//! wgpu renderer, chunk meshing, and render-world extraction.

mod camera;
mod mesh;
mod pipeline;
mod renderer;
mod runtime;
mod world_mesh;

pub use camera::Camera;
pub use mesh::{cube_mesh, MeshVertex, SolidMesh};
pub use renderer::Renderer;
pub use runtime::{RenderFrame, RenderThread};
pub use world_mesh::{
    ChunkMeshCache, RenderScene, extract_render_scene, mesh_chunk, MAX_CHUNK_REBUILDS_PER_FRAME,
};

/// Screen-space LOD distance for chunk mesh generation (world units).
pub const CHUNK_MESH_LOD_DISTANCE: f32 = 192.0;
