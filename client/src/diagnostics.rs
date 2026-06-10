use engine_assets::BlockRegistry;
use engine_core::App;
use engine_render::RenderWorld;
use engine_world::{BlockPos, SparseVoxelOctree};
use game::{TerrainGeneration, WorldInitialized};

/// Snapshot of client pipeline health for logs and automated diagnosis.
#[derive(Debug, Default, Clone)]
pub struct ClientDiagnostics {
    pub frame: u64,
    pub terrain_complete: bool,
    pub world_initialized: bool,
    pub registry_ready: bool,
    pub origin_solid: bool,
    pub mesh_count: usize,
    pub vertex_count: usize,
    pub renderer_ready: bool,
    pub last_present_meshes: usize,
}

impl ClientDiagnostics {
    pub fn sample(app: &App, renderer_ready: bool, last_present_meshes: usize) -> Self {
        let registry_ready = app
            .resource::<BlockRegistry>()
            .and_then(|registry| registry.id_by_name("stone"))
            .is_some();
        let terrain_complete = app
            .resource::<TerrainGeneration>()
            .map(|t| t.complete)
            .unwrap_or(false);
        let world_initialized = app
            .resource::<WorldInitialized>()
            .map(|w| w.0)
            .unwrap_or(false);
        let origin_solid = match (
            app.resource::<SparseVoxelOctree>(),
            app.resource::<BlockRegistry>(),
        ) {
            (Some(world), Some(registry)) => {
                registry.is_solid(world.get_block(BlockPos::new(0, 10, 0)))
            }
            _ => false,
        };
        let (mesh_count, vertex_count) = app
            .resource::<RenderWorld>()
            .map(|world| {
                let count = world.meshes.len();
                let vertices = world.meshes.iter().map(|m| m.vertices.len()).sum();
                (count, vertices)
            })
            .unwrap_or((last_present_meshes, 0));

        Self {
            frame: 0,
            terrain_complete,
            world_initialized,
            registry_ready,
            origin_solid,
            mesh_count,
            vertex_count,
            renderer_ready,
            last_present_meshes,
        }
    }

    pub fn log_line(&self) -> String {
        format!(
            "frame={} registry={} terrain={} world_init={} origin_solid={} meshes={} vertices={} renderer={} presented={}",
            self.frame,
            self.registry_ready,
            self.terrain_complete,
            self.world_initialized,
            self.origin_solid,
            self.mesh_count,
            self.vertex_count,
            self.renderer_ready,
            self.last_present_meshes,
        )
    }

    pub fn is_healthy(&self) -> bool {
        self.registry_ready
            && self.terrain_complete
            && self.world_initialized
            && self.origin_solid
            && self.mesh_count > 0
            && self.vertex_count > 0
    }
}
