use engine_assets::BlockRegistry;
use engine_core::{SystemContext, Time};
use engine_world::{BlockPos, WorldMutationQueue};
use glam::Vec3;
use hecs::Entity;

use crate::axes::view_forward;
use crate::components::{
    Collider, DisplayedPlayerView, LocomotionState, Mounted, NetPlayerId, Player,
    PlayerAnimation, Transform,
};
use crate::movement::MC_TICK_DT;
use crate::events::BlockChangeIntent;
use crate::input::{resolve_input, LocalPlayerId};
use crate::play_mode::{ActivePlayMode, PlayMode};
use crate::voxel_raycast::{
    block_overlaps_player, player_interaction_ray, raycast_voxel, BLOCK_REACH,
};

/// Vanilla place spacing is ~4 ticks at 20 Hz; this is ~3 ticks (~150 ms at 60 Hz).
const PLACE_COOLDOWN_MC_TICKS: f32 = 3.0;

fn place_cooldown_sim_steps(sim_dt: f32) -> u8 {
    (PLACE_COOLDOWN_MC_TICKS * MC_TICK_DT / sim_dt).ceil() as u8
}

pub fn block_interaction_system(ctx: &mut SystemContext<'_>) {
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| mode.0 != PlayMode::Survival)
    {
        return;
    }

    let Some(registry) = ctx.resources.get::<BlockRegistry>().cloned() else {
        return;
    };
    let Some(dirt) = registry.id_by_name("dirt") else {
        return;
    };

    let sim_dt = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(1.0 / 60.0);
    let place_cooldown_reset = place_cooldown_sim_steps(sim_dt);

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
        if let Ok(mut locomotion) = ctx.world.get::<&mut LocomotionState>(player_entity) {
            if locomotion.place_cooldown > 0 {
                locomotion.place_cooldown -= 1;
            }
        }
        if ctx.world.get::<&Mounted>(player_entity).is_ok() {
            continue;
        }

        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        if !input.place_block {
            continue;
        }

        let (origin, direction) = interaction_ray(ctx, net_id, &transform);
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
        let place_ready = ctx
            .world
            .get::<&LocomotionState>(player_entity)
            .map(|locomotion| locomotion.place_cooldown == 0)
            .unwrap_or(true);
        let can_place = place_ready
            && !block_overlaps_player(transform.position, half_extents, place_pos);

        if !can_place {
            continue;
        }

        let Some(queue) = ctx.resources.get_mut::<WorldMutationQueue>() else {
            return;
        };

        queue.set_block(place_pos, dirt);
        ctx.events.send(BlockChangeIntent {
            position: place_pos,
            new_block: dirt,
        });
        if let Ok(mut locomotion) = ctx.world.get::<&mut LocomotionState>(player_entity) {
            locomotion.place_cooldown = place_cooldown_reset;
        }
        if let Ok(mut anim) = ctx.world.get::<&mut PlayerAnimation>(player_entity) {
            anim.trigger_place_swing();
        }
    }
}

fn interaction_ray(
    ctx: &SystemContext<'_>,
    net_id: Option<u32>,
    transform: &Transform,
) -> (Vec3, Vec3) {
    let local_id = ctx.resources.get::<LocalPlayerId>().and_then(|local| local.id);
    let is_local = net_id == local_id;

    if is_local {
        if let Some(view) = ctx.resources.get::<DisplayedPlayerView>() {
            if view.valid {
                return (view.eye, view_forward(view.yaw, view.pitch));
            }
        }
    }

    player_interaction_ray(transform)
}
