use std::collections::HashSet;

use engine_core::SystemContext;
use glam::Vec3;
use hecs::Entity;

use crate::axes::player_view_position;
use crate::components::{Collider, DroppedItem, Player, Transform};
use crate::inventory::{InventoryCommand, InventoryCommandQueue};

/// MC `ItemEntity` pickup search: player bounds inflated 1 block (XY) and 0.5 (vertical).
const PICKUP_INFLATE_XY: f32 = 1.0;
const PICKUP_INFLATE_Z: f32 = 0.5;
const PICKUP_ABSORB_RADIUS: f32 = 0.55;

pub fn item_pickup_system(ctx: &mut SystemContext<'_>) {
    let players: Vec<(Entity, Transform, Collider)> = ctx
        .world
        .query::<(&Player, &Transform, &Collider)>()
        .iter()
        .map(|(entity, (_, transform, collider))| (entity, *transform, *collider))
        .collect();

    let items: Vec<(Entity, Transform, DroppedItem)> = ctx
        .world
        .query::<(&DroppedItem, &Transform)>()
        .iter()
        .map(|(entity, (item, transform))| (entity, *transform, *item))
        .collect();

    let mut delay_ticks: Vec<(Entity, u8)> = Vec::new();
    for (entity, (item,)) in ctx.world.query::<(&mut DroppedItem,)>().iter() {
        if item.pickup_delay_ticks > 0 {
            delay_ticks.push((entity, item.pickup_delay_ticks.saturating_sub(1)));
        }
    }
    for (entity, next) in delay_ticks {
        if let Ok(mut item) = ctx.world.get::<&mut DroppedItem>(entity) {
            item.pickup_delay_ticks = next;
        }
    }

    let Some(queue) = ctx.resources.get_mut::<InventoryCommandQueue>() else {
        return;
    };

    let mut claimed: HashSet<Entity> = HashSet::new();

    for (player_entity, player_transform, collider) in players {
        for (item_entity, item_transform, item) in &items {
            if claimed.contains(item_entity) {
                continue;
            }
            if item.pickup_delay_ticks > 0 {
                continue;
            }
            if !item_within_pickup_reach(
                player_transform.position,
                collider.half_extents,
                item_transform.position,
            ) {
                continue;
            }
            if !item_ready_to_collect(
                player_transform.position,
                collider.half_extents,
                player_transform.yaw,
                item_transform.position,
            ) {
                continue;
            }
            claimed.insert(*item_entity);
            queue.push(InventoryCommand::Insert {
                player: player_entity,
                stack: item.stack,
                world_item: Some(*item_entity),
            });
        }
    }
}

/// `Transform.position` is the player collider center (feet ≈ center − Z half-extent).
pub fn player_pickup_search_aabb(center: Vec3, half_extents: Vec3) -> (Vec3, Vec3) {
    let inflate = Vec3::new(PICKUP_INFLATE_XY, PICKUP_INFLATE_XY, PICKUP_INFLATE_Z);
    (center - half_extents - inflate, center + half_extents + inflate)
}

pub fn item_within_pickup_reach(center: Vec3, half_extents: Vec3, item_pos: Vec3) -> bool {
    let (min, max) = player_pickup_search_aabb(center, half_extents);
    point_in_aabb(item_pos, min, max)
}

pub fn item_ready_to_collect(
    center: Vec3,
    half_extents: Vec3,
    yaw: f32,
    item_pos: Vec3,
) -> bool {
    if point_in_aabb(item_pos, center - half_extents, center + half_extents) {
        return true;
    }
    item_pos.distance(player_view_position(center, yaw)) <= PICKUP_ABSORB_RADIUS
}

pub fn player_suck_target(transform: &Transform, half_extents: Vec3) -> Vec3 {
    let _ = half_extents;
    player_view_position(transform.position, transform.yaw)
}

fn point_in_aabb(point: Vec3, min: Vec3, max: Vec3) -> bool {
    point.x >= min.x
        && point.x <= max.x
        && point.y >= min.y
        && point.y <= max.y
        && point.z >= min.z
        && point.z <= max.z
}

pub fn drop_position_in_front(transform: &Transform) -> glam::Vec3 {
    let forward = crate::axes::horizontal_forward(transform.yaw);
    transform.position + forward * 0.75 + glam::Vec3::new(0.0, 0.0, 0.5)
}
