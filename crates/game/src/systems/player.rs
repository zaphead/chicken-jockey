use std::collections::HashSet;

use engine_core::{SystemContext, Time};
use engine_world::SparseVoxelOctree;
use glam::Vec3;
use hecs::Entity;

use crate::components::{Collider, Mounted, NetPlayerId, Player, Transform, Velocity};
use crate::input::resolve_input;
use crate::systems::physics::collision::collides_aabb;

const WALK_SPEED: f32 = 6.0;
const JUMP_SPEED: f32 = 8.5;
const GRAVITY: f32 = 24.0;
const MOUSE_SENSITIVITY: f32 = 0.002;

pub fn player_look_system(ctx: &mut SystemContext<'_>) {
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
            transform.yaw += input.look_delta.x * MOUSE_SENSITIVITY;
            transform.pitch = (transform.pitch - input.look_delta.y * MOUSE_SENSITIVITY)
                .clamp(-1.5, 1.5);
        }
    }
}

pub fn player_movement_system(ctx: &mut SystemContext<'_>) {
    let mounted = mounted_players(ctx);
    let players: Vec<(Entity, Option<u32>, bool)> = ctx
        .world
        .query::<(&Player, &Transform, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, transform, net_id))| {
            (
                entity,
                net_id.map(|id| id.0),
                is_grounded(ctx, transform.position),
            )
        })
        .collect();

    for (entity, net_id, grounded) in players {
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

        let forward = Vec3::new(transform.yaw.sin(), 0.0, transform.yaw.cos());
        let right = Vec3::new(forward.z, 0.0, -forward.x);
        let wish = (forward * input.move_axis.y + right * input.move_axis.x).normalize_or_zero();

        velocity.0.x = wish.x * WALK_SPEED;
        velocity.0.z = wish.z * WALK_SPEED;

        if input.jump && grounded {
            velocity.0.y = JUMP_SPEED;
        }
    }
}

pub fn player_physics_system(ctx: &mut SystemContext<'_>) {
    let delta = ctx.resources.get::<Time>().map(|time| time.delta).unwrap_or(0.0);
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
        velocity.y -= GRAVITY * delta;
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

fn is_grounded(ctx: &SystemContext<'_>, position: Vec3) -> bool {
    collides_at(
        ctx,
        position + Vec3::new(0.0, -1.05, 0.0),
        Vec3::new(0.35, 0.05, 0.35),
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
