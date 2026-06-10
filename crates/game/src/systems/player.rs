use std::collections::HashSet;

use engine_core::{SystemContext, Time};
use engine_world::SparseVoxelOctree;
use glam::{Vec2, Vec3};
use hecs::Entity;

use crate::axes::{grounded_probe_offset, UP};
use crate::components::{Collider, GroundContact, Mounted, NetPlayerId, Player, Transform, Velocity};
use crate::input::resolve_input;
use crate::movement::{
    accelerate_horizontal, apply_horizontal_drag, apply_look_delta, wish_direction_horizontal,
    AIR_ACCEL_FACTOR, AIR_DRAG, AIR_SPEED_FACTOR, GROUND_ACCEL, LocomotionConfig,
};
use crate::play_mode::{ActivePlayMode, PlayMode};
use crate::systems::physics::collision::collides_aabb;

const GRAVITY: f32 = 32.0;
const JUMP_SPEED: f32 = 9.6;

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
            apply_look_delta(&mut transform, input.look_delta);
        }
    }
}

pub fn player_movement_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }
    let delta = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);
    let config = LocomotionConfig::for_mode(PlayMode::Survival);
    let mounted = mounted_players(ctx);
    let players: Vec<(Entity, Option<u32>, Vec3, Vec3)> = ctx
        .world
        .query::<(&Player, &Transform, &Collider, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, transform, collider, net_id))| {
            (
                entity,
                net_id.map(|id| id.0),
                transform.position,
                collider.half_extents,
            )
        })
        .collect();

    for (entity, net_id, position, half_extents) in players {
        if mounted.contains(&entity) {
            continue;
        }
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        let Ok(mut velocity) = ctx.world.get::<&mut Velocity>(entity) else {
            continue;
        };
        let Ok(transform) = ctx.world.get::<&Transform>(entity) else {
            continue;
        };

        let wish = wish_direction_horizontal(transform.yaw, input.move_axis);
        let grounded = is_grounded(ctx, position, half_extents);
        let was_grounded = ctx
            .world
            .get::<&GroundContact>(entity)
            .map(|contact| contact.grounded)
            .unwrap_or(false);
        let mut horiz = Vec2::new(velocity.0.x, velocity.0.y);

        if grounded {
            let speed = crate::movement::max_speed(config, input.sprint);
            let target = Vec2::new(wish.x * speed, wish.y * speed);
            if !was_grounded {
                horiz = target;
            } else {
                horiz = accelerate_horizontal(horiz, target, GROUND_ACCEL * delta);
            }
        } else {
            let air_speed_cap = if input.sprint {
                crate::movement::max_speed(config, true)
            } else {
                crate::movement::max_speed(config, false) * AIR_SPEED_FACTOR
            };
            let air_accel_step = GROUND_ACCEL * AIR_ACCEL_FACTOR * delta;
            if wish.length_squared() > 0.0 {
                let wish_h = Vec2::new(wish.x, wish.y);
                let along = horiz.dot(wish_h);
                if along < air_speed_cap {
                    horiz += wish_h * air_accel_step.min(air_speed_cap - along);
                }
            } else {
                horiz = apply_horizontal_drag(horiz, AIR_DRAG, delta);
            }
        }

        velocity.0.x = horiz.x;
        velocity.0.y = horiz.y;

        if input.jump && grounded {
            velocity.0.z = JUMP_SPEED;
        }

        if let Ok(mut contact) = ctx.world.get::<&mut GroundContact>(entity) {
            contact.grounded = grounded;
        }
    }
}

pub fn player_physics_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }
    let delta = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(0.0);
    let mounted = mounted_players(ctx);

    let updates: Vec<(Entity, Vec3, Vec3, Vec3)> = ctx
        .world
        .query::<(&Player, &Transform, &Velocity, &Collider)>()
        .iter()
        .filter(|(entity, _)| !mounted.contains(entity))
        .map(|(entity, (_, transform, velocity, collider))| {
            (entity, transform.position, velocity.0, collider.half_extents)
        })
        .collect();

    for (entity, start_position, mut velocity, half_extents) in updates {
        velocity -= UP * GRAVITY * delta;
        let mut position = start_position;

        for axis in 0..3 {
            let delta_axis = velocity[axis] * delta;
            if delta_axis == 0.0 {
                continue;
            }
            position[axis] += delta_axis;
            if collides_at(ctx, position, half_extents) {
                position[axis] -= delta_axis;
                velocity[axis] = 0.0;
            }
        }

        if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
            transform.position = position;
        }
        if let Ok(mut velocity_ref) = ctx.world.get::<&mut Velocity>(entity) {
            velocity_ref.0 = velocity;
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
