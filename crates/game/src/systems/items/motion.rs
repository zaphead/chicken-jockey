use crate::inventory::stacks_fit_together;
use engine_core::{SystemContext, Time};
use engine_world::{BlockPos, SparseVoxelOctree};
use glam::Vec3;
use hecs::Entity;

use crate::components::{Collider, DroppedItem, Player, Transform, Velocity, WorldItemId};
use crate::systems::items::pickup::{item_within_pickup_reach, player_suck_target};
use crate::world_items::WorldItemBook;

const ITEM_GRAVITY: f32 = 18.0;
const ITEM_GROUND_HALF: f32 = 0.125;
const ITEM_GROUND_FRICTION: f32 = 0.72;
const ITEM_MAGNET_RANGE: f32 = 1.25;
const ITEM_MAGNET_ACCEL: f32 = 10.0;
const PICKUP_SUCK_MIN_SPEED: f32 = 7.0;
const PICKUP_SUCK_MAX_SPEED: f32 = 26.0;
const PICKUP_SUCK_DIST_SCALE: f32 = 20.0;

pub fn item_motion_system(ctx: &mut SystemContext<'_>) {
    if ctx.resources.get::<SparseVoxelOctree>().is_none() {
        return;
    }
    let dt = ctx
        .resources
        .get::<Time>()
        .map(|time| time.fixed_delta)
        .unwrap_or(1.0 / 60.0);

    let items: Vec<(Entity, WorldItemId, DroppedItem)> = ctx
        .world
        .query::<(&DroppedItem, &WorldItemId)>()
        .iter()
        .map(|(entity, (item, id))| (entity, *id, *item))
        .collect();

    let positions: Vec<Vec3> = items
        .iter()
        .map(|(entity, _, _)| {
            ctx.world
                .get::<&Transform>(*entity)
                .map(|transform| transform.position)
                .unwrap_or_default()
        })
        .collect();

    let players: Vec<(Transform, Collider)> = ctx
        .world
        .query::<(&Player, &Transform, &Collider)>()
        .iter()
        .map(|(_, (_, transform, collider))| (*transform, *collider))
        .collect();

    let mut accelerations: Vec<Vec3> = vec![Vec3::ZERO; items.len()];
    let mut suck_targets: Vec<Option<Vec3>> = vec![None; items.len()];

    for i in 0..items.len() {
        let (_, _, item_a) = items[i];
        let pos_a = positions[i];

        for j in (i + 1)..items.len() {
            let (_, _, item_b) = items[j];
            if !stacks_fit_together(&item_a.stack, &item_b.stack) {
                continue;
            }

            let delta = positions[j] - pos_a;
            let dist = delta.length();
            if dist < 1e-4 || dist > ITEM_MAGNET_RANGE {
                continue;
            }

            let strength = (1.0 - dist / ITEM_MAGNET_RANGE).max(0.0);
            let pull = delta.normalize() * (ITEM_MAGNET_ACCEL * strength);
            accelerations[i] += pull;
            accelerations[j] -= pull;
        }

        if item_a.pickup_delay_ticks > 0 {
            continue;
        }
        for (player_transform, collider) in &players {
            if !item_within_pickup_reach(
                player_transform.position,
                collider.half_extents,
                pos_a,
            ) {
                continue;
            }
            suck_targets[i] = Some(player_suck_target(player_transform, collider.half_extents));
            break;
        }
    }

    for (index, (entity, world_id, _)) in items.iter().enumerate() {
        let Ok(mut transform) = ctx.world.get::<&mut Transform>(*entity) else {
            continue;
        };

        if let Some(eye) = suck_targets[index] {
            let delta = eye - transform.position;
            let dist = delta.length();
            let speed = (dist * PICKUP_SUCK_DIST_SCALE).clamp(PICKUP_SUCK_MIN_SPEED, PICKUP_SUCK_MAX_SPEED);
            let velocity = if dist > 1e-4 {
                delta.normalize() * speed
            } else {
                Vec3::ZERO
            };
            transform.position += velocity * dt;
            if let Ok(mut vel) = ctx.world.get::<&mut Velocity>(*entity) {
                vel.0 = velocity;
            }
        } else {
            let mut velocity = ctx
                .world
                .get::<&Velocity>(*entity)
                .map(|velocity| velocity.0)
                .unwrap_or_default();

            velocity.z -= ITEM_GRAVITY * dt;
            velocity += accelerations[index] * dt;
            velocity *= 0.98;
            transform.position += velocity * dt;
            resolve_item_ground(ctx, &mut transform.position, &mut velocity);

            if let Ok(mut vel) = ctx.world.get::<&mut Velocity>(*entity) {
                vel.0 = velocity;
            }
        }

        if let Some(book) = ctx.resources.get_mut::<WorldItemBook>() {
            book.update_position(*world_id, transform.position);
        }
    }
}

fn resolve_item_ground(ctx: &SystemContext<'_>, position: &mut Vec3, velocity: &mut Vec3) {
    let Some(world) = ctx.resources.get::<SparseVoxelOctree>() else {
        return;
    };
    let bottom_z = position.z - ITEM_GROUND_HALF;
    let below = BlockPos::new(
        position.x.floor() as i32,
        position.y.floor() as i32,
        bottom_z.floor() as i32,
    );
    if !world.is_solid(below) {
        return;
    }
    let floor_z = below.0.z as f32 + 1.0;
    if bottom_z >= floor_z {
        return;
    }
    position.z = floor_z + ITEM_GROUND_HALF;
    if velocity.z < 0.0 {
        velocity.z = 0.0;
    }
    velocity.x *= ITEM_GROUND_FRICTION;
    velocity.y *= ITEM_GROUND_FRICTION;
}
