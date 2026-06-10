use std::collections::HashSet;

use engine_core::{SystemContext, Time};
use engine_world::SparseVoxelOctree;
use glam::{Vec2, Vec3};
use hecs::Entity;

use crate::axes::{grounded_probe_offset, horizontal_forward};
use crate::components::{
    Collider, LocomotionState, Mounted, NetPlayerId, Player, Transform, Velocity,
};
use crate::input::resolve_input;
use crate::movement::{
    apply_vertical_post_move, jump_cooldown_sim_steps, update_horizontal_velocity,
    wish_direction_horizontal, JUMP_VELOCITY, McMovementInput, MC_TICK_DT,
};
use crate::play_mode::{ActivePlayMode, PlayMode};
use crate::systems::physics::collision::collides_aabb;

fn survival_active(ctx: &SystemContext<'_>) -> bool {
    ctx.resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.0 == PlayMode::Survival)
}

pub fn player_look_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }
    let mounted: HashSet<Entity> = mounted_players(ctx);
    let players: Vec<(Entity, Option<u32>)> = ctx
        .world
        .query::<(&Player, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, net_id))| (entity, net_id.map(|id| id.0)))
        .collect();

    for (entity, net_id) in players {
        if mounted.contains(&entity) {
            continue;
        }
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
            crate::movement::apply_look_delta(&mut transform, input.look_delta);
        }
    }
}

pub fn player_locomotion_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }

    let delta = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);
    let jump_cooldown_reset = jump_cooldown_sim_steps(delta);
    let mounted = mounted_players(ctx);

    let entities: Vec<(Entity, Option<u32>, Vec3, Vec3, f32)> = ctx
        .world
        .query::<(&Player, &Transform, &Collider, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, transform, collider, net_id))| {
            (
                entity,
                net_id.map(|id| id.0),
                transform.position,
                collider.half_extents,
                transform.yaw,
            )
        })
        .collect();

    for (entity, net_id, position, half_extents, yaw) in entities {
        if mounted.contains(&entity) {
            continue;
        }
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };

        let on_ground = is_grounded(ctx, position, half_extents);

        let mut position = position;
        {
            let Ok(mut velocity) = ctx.world.get::<&mut Velocity>(entity) else {
                continue;
            };
            let Ok(mut locomotion) = ctx.world.get::<&mut LocomotionState>(entity) else {
                continue;
            };

            let will_jump =
                input.jump && locomotion.was_on_ground && locomotion.jump_cooldown == 0;

            let wish = wish_direction_horizontal(yaw, input.move_axis);
            let facing = horizontal_forward(yaw);
            let mc_input = McMovementInput {
                wish_dir: wish,
                facing_dir: facing,
                move_axis: input.move_axis,
                sprint: input.sprint,
            };

            let mut horiz = Vec2::new(velocity.0.x, velocity.0.y);
            locomotion.horizontal_tick_accum += delta;

            if will_jump {
                horiz = update_horizontal_velocity(
                    horiz,
                    &mc_input,
                    locomotion.was_on_ground,
                    on_ground,
                    true,
                    MC_TICK_DT,
                );
                velocity.0.z = JUMP_VELOCITY;
                locomotion.jump_cooldown = jump_cooldown_reset;
                locomotion.horizontal_tick_accum = locomotion
                    .horizontal_tick_accum
                    .rem_euclid(MC_TICK_DT);
            } else {
                while locomotion.horizontal_tick_accum >= MC_TICK_DT {
                    locomotion.horizontal_tick_accum -= MC_TICK_DT;
                    horiz = update_horizontal_velocity(
                        horiz,
                        &mc_input,
                        locomotion.was_on_ground,
                        on_ground,
                        false,
                        MC_TICK_DT,
                    );
                }
            }

            velocity.0.x = horiz.x;
            velocity.0.y = horiz.y;

            let mut vel = velocity.0;

            for axis in 0..3 {
                let delta_axis = vel[axis] * delta;
                if delta_axis == 0.0 {
                    continue;
                }
                position[axis] += delta_axis;
                if collides_at(ctx, position, half_extents) {
                    position[axis] -= delta_axis;
                    vel[axis] = 0.0;
                }
            }

            vel.z = apply_vertical_post_move(vel.z, delta);

            velocity.0 = vel;
            locomotion.on_ground = on_ground;
            locomotion.was_on_ground = on_ground;
            if locomotion.jump_cooldown > 0 {
                locomotion.jump_cooldown -= 1;
            }
        }

        if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
            transform.position = position;
        }
    }
}

fn mounted_players(ctx: &SystemContext<'_>) -> HashSet<Entity> {
    ctx.world
        .query::<(&Player, &Mounted)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect()
}

fn is_grounded(ctx: &SystemContext<'_>, position: Vec3, half_extents: Vec3) -> bool {
    collides_at(
        ctx,
        position + grounded_probe_offset(half_extents.z),
        Vec3::new(half_extents.x, half_extents.y, 0.05),
    )
}

pub(crate) fn collides_at(ctx: &SystemContext<'_>, position: Vec3, half_extents: Vec3) -> bool {
    let Some(registry) = ctx.resources.get::<engine_assets::BlockRegistry>() else {
        return false;
    };
    let Some(world) = ctx.resources.get::<SparseVoxelOctree>() else {
        return false;
    };
    collides_aabb(world, registry, position, half_extents)
}
