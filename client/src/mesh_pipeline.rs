use engine_assets::BlockRegistry;
use engine_render::{RenderExtractState, CHUNK_MESH_LOD_DISTANCE};
use engine_world::{SparseVoxelOctree, CHUNK_SIZE};
use game::WORLD_RADIUS;
use glam::{IVec3, Vec3};

pub const MESH_BATCH_SIZE: usize = 16;

pub fn queue_initial_world_chunks(state: &mut RenderExtractState) {
    if state.world_mesh_queued {
        return;
    }
    let radius = WORLD_RADIUS / CHUNK_SIZE + 1;
    for cx in -radius..radius {
        for cz in -radius..radius {
            for cy in 0..2 {
                state.world_mesh_queue.push(IVec3::new(cx, cy, cz));
            }
        }
    }
    state.world_mesh_queued = true;
}

pub fn enqueue_mesh_batch(state: &mut RenderExtractState) {
    let batch = state.world_mesh_queue.len().min(MESH_BATCH_SIZE);
    for chunk in state.world_mesh_queue.drain(..batch) {
        state.mesh_cache.mark_dirty(chunk);
    }
}

pub fn rebuild_chunk_meshes(
    state: &mut RenderExtractState,
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    camera_position: Vec3,
) -> usize {
    state.mesh_cache.rebuild_dirty_near(
        world,
        registry,
        camera_position,
        CHUNK_MESH_LOD_DISTANCE,
    )
}
