use engine_core::SystemContext;
use engine_net::{InventorySync, WorldItemSnapshot};
use game::{
    inventory_from_wire, stack_from_wire, DroppedItem, LocalPlayerId, NetPlayerId, PlayerInventory,
    Transform, WorldItemId,
};
use glam::Vec3;

pub fn apply_world_items(ctx: &mut SystemContext<'_>, snapshots: Vec<WorldItemSnapshot>) {
    let mut live_ids: std::collections::HashSet<u32> = snapshots.iter().map(|snap| snap.id).collect();

    let existing: Vec<_> = ctx
        .world
        .query::<&WorldItemId>()
        .iter()
        .map(|(entity, id)| (entity, id.0))
        .collect();

    for (entity, id) in existing {
        if !live_ids.contains(&id) {
            ctx.commands.push(move |world| {
                let _ = world.despawn(entity);
            });
        }
    }

    for snap in snapshots {
        let position = Vec3::from_array(snap.position);
        let stack = stack_from_wire(snap.stack);
        if let Some(entity) = ctx
            .world
            .query::<&WorldItemId>()
            .iter()
            .find_map(|(entity, id)| (id.0 == snap.id).then_some(entity))
        {
            if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
                transform.position = position;
            }
            if let Ok(mut item) = ctx.world.get::<&mut DroppedItem>(entity) {
                item.stack = stack;
            }
        } else {
            let id = snap.id;
            ctx.commands.push(move |world| {
                let _ = world.spawn((
                    DroppedItem {
                        stack,
                        pickup_delay_ticks: 0,
                    },
                    WorldItemId(id),
                    Transform {
                        position,
                        yaw: 0.0,
                        pitch: 0.0,
                    },
                ));
            });
        }
        live_ids.remove(&snap.id);
    }
}

pub fn apply_inventory_sync(ctx: &mut SystemContext<'_>, sync: InventorySync) {
    let local_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id);
    let Some(local_id) = local_id else {
        return;
    };
    if sync.player_id != local_id {
        return;
    }

    for (entity, (id,)) in ctx.world.query::<(&NetPlayerId,)>().iter() {
        if id.0 != local_id {
            continue;
        }
        if let Ok(mut inventory) = ctx.world.get::<&mut PlayerInventory>(entity) {
            *inventory = inventory_from_wire(sync.slots.clone(), sync.selected);
        }
    }
}
