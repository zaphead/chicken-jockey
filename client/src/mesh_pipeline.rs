use engine_assets::{BlockMaterialMap, BlockRegistry};
use engine_render::{RenderExtractState, RebuildBudget, CHUNK_MESH_RENDER_DISTANCE};
use engine_world::{SparseVoxelOctree, CHUNK_SIZE};
use game::{GRASS_PLANE_Z, WORLD_RADIUS};
use glam::{IVec3, Vec3};

pub const MESH_BATCH_SIZE: usize = 16;

fn terrain_chunk_coords() -> impl Iterator<Item = IVec3> {
    let min = -WORLD_RADIUS;
    let max = WORLD_RADIUS - 1;
    let min_cx = min.div_euclid(CHUNK_SIZE);
    let max_cx = max.div_euclid(CHUNK_SIZE);
    let min_cy = min.div_euclid(CHUNK_SIZE);
    let max_cy = max.div_euclid(CHUNK_SIZE);
    let chunk_z = GRASS_PLANE_Z.div_euclid(CHUNK_SIZE);
    (min_cx..=max_cx).flat_map(move |cx| {
        (min_cy..=max_cy).map(move |cy| IVec3::new(cx, cy, chunk_z))
    })
}

pub fn bootstrap_terrain_meshes(state: &mut RenderExtractState) {
    if state.terrain_bootstrapped {
        return;
    }
    state.mesh_cache = engine_render::ChunkMeshCache::default();
    for chunk in terrain_chunk_coords() {
        state.mesh_cache.mark_dirty(chunk);
    }
    state.terrain_bootstrapped = true;
    state.pending_full_rebuild = true;
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
    materials: &BlockMaterialMap,
    camera_position: Vec3,
    budget: RebuildBudget,
) -> usize {
    state.mesh_cache.rebuild(
        world,
        registry,
        materials,
        camera_position,
        budget,
        true,
    )
}

pub fn rebuild_budget_for_extract(state: &mut RenderExtractState) -> RebuildBudget {
    if state.pending_full_rebuild {
        state.pending_full_rebuild = false;
        RebuildBudget::all()
    } else {
        RebuildBudget::near(CHUNK_MESH_RENDER_DISTANCE)
    }
}
