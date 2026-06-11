use engine_assets::{BlockRegistry, ToolRegistry};
use engine_core::{SystemContext, Time};
use engine_world::{BlockPos, WorldMutationQueue};
use glam::IVec3;

use crate::components::{
    BlockMiningState, Mounted, NetPlayerId, Player, PlayerInventory, Transform,
};
use crate::events::{BlockBroken, BlockChangeIntent, BlockMiningProgress};
use crate::input::resolve_input;
use crate::mining::{can_harvest_block, mining_delta, tool_efficiency};
use crate::play_mode::survival_active;
use crate::voxel_raycast::{authoritative_interaction_ray, raycast_voxel, BLOCK_REACH};

pub fn block_mining_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }

    let Some(registry) = ctx.resources.get::<BlockRegistry>().cloned() else {
        return;
    };
    let Some(tools) = ctx.resources.get::<ToolRegistry>().cloned() else {
        return;
    };
    let Some(air) = registry.id_by_name("air") else {
        return;
    };
    let sim_dt = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(1.0 / 60.0);

    let players: Vec<(hecs::Entity, Transform, Option<u32>)> = ctx
        .world
        .query::<(&Player, &Transform, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, transform, net_id))| {
            (entity, *transform, net_id.map(|id| id.0))
        })
        .collect();

    for (player_entity, transform, net_id) in players {
        if ctx.world.get::<&Mounted>(player_entity).is_ok() {
            reset_mining(ctx, player_entity);
            continue;
        }

        let Some(input) = resolve_input(ctx, net_id) else {
            reset_mining(ctx, player_entity);
            continue;
        };

        if !input.break_block {
            reset_mining(ctx, player_entity);
            continue;
        }

        let (origin, direction) = authoritative_interaction_ray(ctx, net_id, &transform);
        let Some(world) = ctx.resources.get::<engine_world::SparseVoxelOctree>() else {
            continue;
        };
        let Some(hit) = raycast_voxel(world, &registry, origin, direction, BLOCK_REACH) else {
            reset_mining(ctx, player_entity);
            continue;
        };

        let block_id = world.get_block(hit.block_pos);
        if !registry.is_breakable(block_id) {
            reset_mining(ctx, player_entity);
            continue;
        }

        let Ok(mut mining) = ctx.world.get::<&mut BlockMiningState>(player_entity) else {
            continue;
        };
        let Ok(inventory) = ctx.world.get::<&PlayerInventory>(player_entity) else {
            continue;
        };

        let same_target = mining
            .target
            .is_some_and(|pos| pos == hit.block_pos && mining.target_block == block_id);

        if !same_target {
            mining.target = Some(hit.block_pos);
            mining.target_block = block_id;
            mining.face_normal = hit.normal;
            mining.progress = 0.0;
        }

        let active_tool = inventory.active_tool();
        let hardness = registry.hardness(block_id);
        let can_harvest = can_harvest_block(&registry, &tools, block_id, active_tool);
        let efficiency = tool_efficiency(&tools, active_tool);
        mining.progress += mining_delta(sim_dt, hardness, efficiency, can_harvest);

        if mining.progress >= 1.0 {
            let pos = hit.block_pos;
            let harvested = can_harvest;
            if let Some(queue) = ctx.resources.get_mut::<WorldMutationQueue>() {
                queue.set_block(pos, air);
            }
            ctx.events.send(BlockChangeIntent {
                position: pos,
                new_block: air,
            });
            ctx.events.send(BlockBroken {
                position: pos,
                block_id,
                harvested,
            });
            mining.target = None;
            mining.progress = 0.0;
            ctx.events.send(BlockMiningProgress {
                position: pos,
                face_normal: hit.normal,
                progress: 0.0,
            });
        } else {
            ctx.events.send(BlockMiningProgress {
                position: hit.block_pos,
                face_normal: mining.face_normal,
                progress: mining.progress,
            });
        }
    }
}

fn reset_mining(ctx: &mut SystemContext<'_>, player_entity: hecs::Entity) {
    let Ok(mut mining) = ctx.world.get::<&mut BlockMiningState>(player_entity) else {
        return;
    };
    if mining.target.is_some() || mining.progress > 0.0 {
        mining.target = None;
        mining.progress = 0.0;
        ctx.events.send(BlockMiningProgress {
            position: BlockPos::new(0, 0, 0),
            face_normal: IVec3::ZERO,
            progress: -1.0,
        });
    }
}

