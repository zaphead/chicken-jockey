//! wgpu renderer, chunk meshing, and render-world extraction.

mod camera;
mod ctm;
mod extract;
mod hud;
mod mesh;
mod mining_overlay;
mod outline;
mod pipeline;
mod renderer;
mod world_mesh;

pub use camera::Camera;
pub use hud::HudPipeline;
pub use extract::{MiningOverlay, RenderExtractState, RenderSurfaceInfo, RenderWorld};
pub use mining_overlay::{build_mining_overlay_mesh, MiningOverlayMesh, MiningOverlayVertex};
pub use mesh::{append_face, MeshBuckets, MeshVertex, SolidMesh, VERTEX_FLAG_OVERLAY};
pub use renderer::Renderer;
pub use world_mesh::{
    extract_render_scene, mesh_chunk, ChunkMeshCache, RebuildBudget, RenderScene,
    MAX_CHUNK_REBUILDS_PER_FRAME,
};

/// Max distance from the camera to build and retain chunk meshes (world units).
pub const CHUNK_MESH_RENDER_DISTANCE: f32 = 192.0;
