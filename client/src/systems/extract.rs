use std::sync::Arc;

use engine_assets::{BlockRegistry, PackedBlockTextures};
use engine_core::SystemContext;
use engine_render::{Camera, RenderExtractState, RenderSurfaceInfo, RenderWorld};
use engine_world::{BlockChanged, SparseVoxelOctree};
use game::WorldInitialized;

use crate::mesh_pipeline::{bootstrap_terrain_meshes, rebuild_budget_for_extract, rebuild_chunk_meshes};
use crate::systems::spectator::SpectatorCamera;

pub fn sync_block_changes_system(ctx: &mut SystemContext<'_>) {
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    let changes: Vec<BlockChanged> = ctx.events.drain::<BlockChanged>();
    // Bulk terrain fill emits one event per block; initial mesh bootstrap handles that.
    if changes.len() > 64 {
        return;
    }
    for change in changes {
        state.mesh_cache.mark_dirty_neighbors(change.position);
    }
}

pub fn queue_initial_world_meshes_system(ctx: &mut SystemContext<'_>) {
    let initialized = ctx
        .resources
        .get::<WorldInitialized>()
        .map(|flag| flag.0)
        .unwrap_or(false);
    if !initialized {
        return;
    }
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    bootstrap_terrain_meshes(state);
}

pub fn extract_render_world_system(ctx: &mut SystemContext<'_>) {
    let aspect = ctx
        .resources
        .get::<RenderSurfaceInfo>()
        .map(|info| info.aspect)
        .unwrap_or(16.0 / 9.0);
    let camera = extract_camera(ctx, aspect);

    let Some(packed) = ctx.resources.get::<Arc<PackedBlockTextures>>().cloned() else {
        return;
    };
    let meshes = ctx
        .resources
        .with_triple::<SparseVoxelOctree, BlockRegistry, RenderExtractState, _>(|world, registry, state| {
            if state.mesh_cache.has_dirty_chunks() {
                let budget = rebuild_budget_for_extract(state);
                rebuild_chunk_meshes(
                    state,
                    world,
                    registry,
                    &packed.materials,
                    camera.position,
                    budget,
                );
            }
            state.mesh_cache.all_meshes()
        })
        .unwrap_or_default();

    if let Some(render_world) = ctx.resources.get_mut::<RenderWorld>() {
        render_world.camera = camera;
        render_world.meshes = meshes;
        render_world.ready = true;
    }
}

fn extract_camera(ctx: &SystemContext<'_>, aspect: f32) -> Camera {
    let spectator = ctx
        .resources
        .get::<SpectatorCamera>()
        .expect("SpectatorCamera must be registered");
    Camera {
        position: spectator.position,
        yaw: spectator.yaw,
        pitch: spectator.pitch,
        aspect,
        ..Camera::default()
    }
}
