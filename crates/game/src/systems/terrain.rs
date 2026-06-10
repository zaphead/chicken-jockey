use engine_assets::BlockRegistry;
use engine_core::SystemContext;
use engine_world::{BlockPos, WorldMutationQueue};

use crate::components::TerrainGeneration;
use crate::debug_world::{ActiveDebugWorld, DebugWorldKind};

pub const WORLD_RADIUS: i32 = 16;
pub const FLAT_WORLD_RADIUS: i32 = 64;
pub const FLAT_SURFACE_Z: i32 = 4;
pub const GRASS_PLANE_Z: i32 = FLAT_SURFACE_Z;

const PLAYER_HALF_HEIGHT: f32 = 1.0;
pub const PLAYER_SPAWN_PITCH: f32 = -0.25;

pub fn generate_terrain_system(ctx: &mut SystemContext<'_>) {
    let Some(progress) = ctx.resources.get::<TerrainGeneration>().copied() else {
        return;
    };
    if progress.complete {
        return;
    }

    let Some(registry) = ctx.resources.get::<BlockRegistry>() else {
        return;
    };
    let Some(grass) = registry.id_by_name("grass") else {
        return;
    };
    let Some(dirt) = registry.id_by_name("dirt") else {
        return;
    };
    let Some(stone) = registry.id_by_name("stone") else {
        return;
    };

    let world = ctx
        .resources
        .get::<ActiveDebugWorld>()
        .map(|active| active.0)
        .unwrap_or(DebugWorldKind::Flat);

    let Some(queue) = ctx.resources.get_mut::<WorldMutationQueue>() else {
        return;
    };

    match world {
        DebugWorldKind::Flat => generate_flat_world(queue, grass, dirt, stone),
    }

    if let Some(progress) = ctx.resources.get_mut::<TerrainGeneration>() {
        progress.complete = true;
    }
}

fn generate_flat_world(
    queue: &mut WorldMutationQueue,
    grass: engine_world::BlockId,
    dirt: engine_world::BlockId,
    stone: engine_world::BlockId,
) {
    for x in -FLAT_WORLD_RADIUS..FLAT_WORLD_RADIUS {
        for y in -FLAT_WORLD_RADIUS..FLAT_WORLD_RADIUS {
            for z in 0..=FLAT_SURFACE_Z {
                let block = if z == FLAT_SURFACE_Z {
                    grass
                } else if z >= FLAT_SURFACE_Z - 3 {
                    dirt
                } else {
                    stone
                };
                queue.set_block(BlockPos::new(x, y, z), block);
            }
        }
    }
}

pub fn terrain_surface_z(_x: i32, _y: i32, world: DebugWorldKind, _seed: u32) -> i32 {
    match world {
        DebugWorldKind::Flat => FLAT_SURFACE_Z,
    }
}

pub fn player_ground_center_z_at(x: i32, y: i32, world: DebugWorldKind, seed: u32) -> f32 {
    terrain_surface_z(x, y, world, seed) as f32 + 1.0 + PLAYER_HALF_HEIGHT
}

pub fn player_spawn_center_z_at(x: i32, y: i32, world: DebugWorldKind, seed: u32) -> f32 {
    player_ground_center_z_at(x, y, world, seed)
}

pub fn player_spawn_center_z(world: DebugWorldKind, seed: u32) -> f32 {
    player_spawn_center_z_at(0, 0, world, seed)
}
