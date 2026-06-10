use engine_assets::BlockRegistry;
use engine_core::SystemContext;
use engine_world::{BlockPos, SparseVoxelOctree, WorldMutationQueue};
use glam::Vec3;
use hecs::Entity;

use crate::components::{Mounted, NetPlayerId, Player, Transform};
use crate::events::BlockChangeIntent;
use crate::input::resolve_input;

const REACH: f32 = 6.0;

pub fn block_interaction_system(ctx: &mut SystemContext<'_>) {
    let Some(registry) = ctx.resources.get::<BlockRegistry>() else {
        return;
    };
    let Some(air) = registry.id_by_name("air") else {
        return;
    };
    let Some(stone) = registry.id_by_name("stone") else {
        return;
    };

    let players: Vec<(Entity, Transform, Option<u32>)> = ctx
        .world
        .query::<(&Player, &Transform, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, transform, net_id))| {
            (entity, *transform, net_id.map(|id| id.0))
        })
        .collect();

    for (player_entity, transform, net_id) in players {
        if ctx.world.get::<&Mounted>(player_entity).is_ok() {
            continue;
        }

        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        if !input.break_block && !input.place_block {
            continue;
        }

        let origin = transform.position + Vec3::new(0.0, 1.5, 0.0);
        let direction = forward(&transform);

        let Some(hit) = raycast_voxel(ctx, origin, direction, REACH) else {
            continue;
        };

        let place_pos = BlockPos::new(
            hit.block_pos.0.x + hit.normal.x,
            hit.block_pos.0.y + hit.normal.y,
            hit.block_pos.0.z + hit.normal.z,
        );
        let can_place = input.place_block && !occupies_player(transform.position, place_pos);

        let Some(queue) = ctx.resources.get_mut::<WorldMutationQueue>() else {
            return;
        };

        if input.break_block {
            queue.set_block(hit.block_pos, air);
            ctx.events.send(BlockChangeIntent {
                position: hit.block_pos,
                new_block: air,
            });
        } else if can_place {
            queue.set_block(place_pos, stone);
            ctx.events.send(BlockChangeIntent {
                position: place_pos,
                new_block: stone,
            });
        }
    }
}

struct RayHit {
    block_pos: BlockPos,
    normal: glam::IVec3,
}

fn forward(transform: &Transform) -> Vec3 {
    let (sy, cy) = transform.yaw.sin_cos();
    let (sp, cp) = transform.pitch.sin_cos();
    Vec3::new(sy * cp, sp, cy * cp).normalize()
}

fn raycast_voxel(
    ctx: &SystemContext<'_>,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<RayHit> {
    let world = ctx.resources.get::<SparseVoxelOctree>()?;
    let registry = ctx.resources.get::<BlockRegistry>()?;

    let mut t = 0.0;
    let mut current = origin.floor().as_ivec3();
    let step = direction.signum().as_ivec3();
    let mut t_max = Vec3::ZERO;
    let mut t_delta = Vec3::splat(f32::INFINITY);

    for axis in 0..3 {
        if direction[axis] != 0.0 {
            let boundary = if step[axis] > 0 {
                current[axis] as f32 + 1.0
            } else {
                current[axis] as f32
            };
            t_max[axis] = (boundary - origin[axis]) / direction[axis];
            t_delta[axis] = (step[axis] as f32 / direction[axis]).abs();
        }
    }

    let mut last_step = glam::IVec3::ZERO;

    while t <= max_distance {
        let pos = BlockPos::new(current.x, current.y, current.z);
        if registry.is_solid(world.get_block(pos)) {
            return Some(RayHit {
                block_pos: pos,
                normal: -last_step,
            });
        }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            t = t_max.x;
            t_max.x += t_delta.x;
            last_step = glam::IVec3::X * step.x;
            current.x += step.x;
        } else if t_max.y < t_max.z {
            t = t_max.y;
            t_max.y += t_delta.y;
            last_step = glam::IVec3::Y * step.y;
            current.y += step.y;
        } else {
            t = t_max.z;
            t_max.z += t_delta.z;
            last_step = glam::IVec3::Z * step.z;
            current.z += step.z;
        }
    }

    None
}

fn occupies_player(player_position: Vec3, position: BlockPos) -> bool {
    player_position.floor().as_ivec3() == position.0
}
