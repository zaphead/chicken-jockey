use engine_world::BlockPos;

use crate::camera::Camera;
use crate::gui::GuiFrame;
use crate::lighting::LightingSnapshot;
use crate::mesh::SolidMesh;
use crate::mining_overlay::MiningOverlayMesh;
use crate::particles::ParticleMesh;
use crate::world_mesh::ChunkMeshCache;

#[derive(Debug, Clone)]
pub struct MiningOverlay {
    pub mesh: MiningOverlayMesh,
}

#[derive(Debug)]
pub struct RenderWorld {
    pub camera: Camera,
    pub opaque: SolidMesh,
    pub cutout: SolidMesh,
    pub animation_tick: u32,
    pub lighting: LightingSnapshot,
    pub target_block: Option<BlockPos>,
    pub mining_overlay: Option<MiningOverlay>,
    pub particles: ParticleMesh,
    pub mesh_generation: u64,
    pub ready: bool,
    /// Global UI scale from client settings (HUD, crosshair, menus).
    pub gui_scale: f32,
    pub gui: GuiFrame,
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            opaque: SolidMesh::default(),
            cutout: SolidMesh::default(),
            animation_tick: 0,
            lighting: LightingSnapshot::default(),
            target_block: None,
            mining_overlay: None,
            particles: ParticleMesh::default(),
            mesh_generation: 0,
            ready: false,
            gui_scale: 4.0,
            gui: GuiFrame::default(),
        }
    }
}

impl RenderWorld {
    /// Legacy helper for tests counting all vertices.
    pub fn meshes(&self) -> Vec<SolidMesh> {
        let mut out = Vec::new();
        if !self.opaque.vertices.is_empty() {
            out.push(self.opaque.clone());
        }
        if !self.cutout.vertices.is_empty() {
            out.push(self.cutout.clone());
        }
        out
    }
}

#[derive(Default)]
pub struct RenderExtractState {
    pub mesh_cache: ChunkMeshCache,
    pub world_mesh_queue: Vec<glam::IVec3>,
    pub terrain_bootstrapped: bool,
    pub pending_full_rebuild: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RenderSurfaceInfo {
    pub width: u32,
    pub height: u32,
    pub aspect: f32,
}
