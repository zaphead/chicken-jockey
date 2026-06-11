//! wgpu renderer, chunk meshing, and render-world extraction.

mod camera;
mod ctm;
mod extract;
mod gui;
mod hud;
pub mod screen_text;
mod lighting;
mod mesh;
mod mining_overlay;
mod outline;
mod particles;
mod pipeline;
mod post;
mod render_passes;
mod shader_source;
mod sky;
mod renderer;
mod world_mesh;

pub use camera::Camera;
pub use gui::{GuiButton, GuiFrame, GuiLabel, GuiPanel, GuiPipeline, GuiRect, GuiSpriteInstance};
pub use hud::HudPipeline;
pub use extract::{MiningOverlay, RenderExtractState, RenderSurfaceInfo, RenderWorld};
pub use lighting::LightingSnapshot;
pub use mining_overlay::{build_mining_overlay_mesh, MiningOverlayMesh, MiningOverlayVertex};
pub use particles::{ParticleMesh, ParticlePipeline, ParticleSystem, ParticleVertex};
pub use mesh::{append_face, MeshBuckets, MeshVertex, SolidMesh, VERTEX_FLAG_OVERLAY};
pub use renderer::Renderer;
pub use world_mesh::{
    extract_render_scene, mesh_chunk, ChunkMeshCache, RebuildBudget, RenderScene,
    MAX_CHUNK_REBUILDS_PER_FRAME,
};

/// Max distance from the camera to build and retain chunk meshes (world units).
pub const CHUNK_MESH_RENDER_DISTANCE: f32 = 192.0;
