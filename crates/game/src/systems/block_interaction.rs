use engine_assets::BlockRegistry;
use engine_core::{SystemContext, Time};
use engine_world::{BlockPos, WorldMutationQueue};
use glam::Vec3;
use hecs::Entity;

use crate::components::{Collider, Mounted, NetPlayerId, Player, Transform};
use crate::events::BlockChangeIntent;
use crate::input::resolve_input;
use crate::play_mode::{ActivePlayMode, PlayMode};
use crate::voxel_raycast::{
    block_overlaps_player, player_interaction_ray, raycast_voxel, BLOCK_REACH,
};

const BLOCK_INTERACTION_INTERVAL: u64 = 4;

pub fn block_interaction_system(ctx: &mut SystemContext<'_>) {
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| mode.0 != PlayMode::Survival)
    {
        return;
    }

    let tick = ctx.resources.get::<Time>().map(|time| time.sim_tick).unwrap_or(0);
    if tick % BLOCK_INTERACTION_INTERVAL != 0 {
        return;
    }

    let Some(registry) = ctx.resources.get::<BlockRegistry>().cloned() else {
        return;
    };
    let Some(air) = registry.id_by_name("air") else {
        return;
    };
    let Some(dirt) = registry.id_by_name("dirt") else {
        return;
    };

    let players: Vec<(Entity, Transform, Option<u32>, Vec3)> = ctx
        .world
        .query::<(&Player, &Transform, Option<&NetPlayerId>, &Collider)>()
        .iter()
        .map(|(entity, (_, transform, net_id, collider))| {
            (
                entity,
                *transform,
                net_id.map(|id| id.0),
                collider.half_extents,
            )
        })
        .collect();

    for (player_entity, transform, net_id, half_extents) in players {
        if ctx.world.get::<&Mounted>(player_entity).is_ok() {
            continue;
        }

        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        if !input.break_block && !input.place_block {
            continue;
        }

        let (origin, direction) = player_interaction_ray(&transform);
        let Some(world) = ctx.resources.get::<engine_world::SparseVoxelOctree>() else {
            return;
        };
        let Some(hit) = raycast_voxel(world, &registry, origin, direction, BLOCK_REACH) else {
            continue;
        };

        let place_pos = BlockPos::new(
            hit.block_pos.0.x + hit.normal.x,
            hit.block_pos.0.y + hit.normal.y,
            hit.block_pos.0.z + hit.normal.z,
        );
        let can_place = input.place_block
            && !block_overlaps_player(transform.position, half_extents, place_pos);

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
            queue.set_block(place_pos, dirt);
            ctx.events.send(BlockChangeIntent {
                position: place_pos,
                new_block: dirt,
            });
        }
    }
}
