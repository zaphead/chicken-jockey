use engine_core::SystemContext;
use hecs::Entity;

use crate::components::WorldItemId;
use crate::inventory::{
    drop_from_slot, mark_inventory_dirty, quick_move, swap_slots, try_insert, InsertResult,
    InventoryCommand, InventoryCommandQueue, PLAYER_DROP_PICKUP_DELAY_TICKS,
};
use crate::systems::items::pickup::drop_position_in_front;
use crate::systems::items::spawn::spawn_drop_at;
use crate::world_items::WorldItemBook;

pub fn apply_inventory_commands_system(ctx: &mut SystemContext<'_>) {
    let Some(mut commands) = ctx
        .resources
        .get_mut::<InventoryCommandQueue>()
        .map(|queue| queue.drain())
    else {
        return;
    };

    for command in commands.drain(..) {
        match command {
            InventoryCommand::Insert {
                player,
                stack,
                world_item,
            } => {
                let result = ctx
                    .world
                    .get::<&mut crate::components::PlayerInventory>(player)
                    .ok()
                    .map(|mut inventory| try_insert(&mut inventory, stack));
                match result {
                    Some(InsertResult::Complete) => {
                        mark_inventory_dirty(ctx, player);
                        if let Some(entity) = world_item {
                            despawn_world_item(ctx, entity);
                        }
                    }
                    Some(InsertResult::Partial { remainder }) => {
                        mark_inventory_dirty(ctx, player);
                        if let Some(entity) = world_item {
                            if let Ok(mut item) =
                                ctx.world.get::<&mut crate::components::DroppedItem>(entity)
                            {
                                item.stack = remainder;
                            }
                            if let (Ok(id), Some(book)) = (
                                ctx.world.get::<&WorldItemId>(entity),
                                ctx.resources.get_mut::<WorldItemBook>(),
                            ) {
                                book.update_stack(*id, remainder);
                            }
                        }
                    }
                    None => {}
                }
            }
            InventoryCommand::MoveSlot { player, from, to } => {
                let changed = ctx
                    .world
                    .get::<&mut crate::components::PlayerInventory>(player)
                    .ok()
                    .map(|mut inventory| {
                        swap_slots(&mut inventory, from, to);
                        true
                    })
                    .unwrap_or(false);
                if changed {
                    mark_inventory_dirty(ctx, player);
                }
            }
            InventoryCommand::QuickMove { player, slot } => {
                let changed = ctx
                    .world
                    .get::<&mut crate::components::PlayerInventory>(player)
                    .ok()
                    .map(|mut inventory| quick_move(&mut inventory, slot))
                    .unwrap_or(false);
                if changed {
                    mark_inventory_dirty(ctx, player);
                }
            }
            InventoryCommand::SwapCarried {
                player,
                slot,
                carried,
            } => {
                let slot = slot as usize;
                if slot >= crate::components::INVENTORY_SLOTS {
                    continue;
                }
                let replaced = ctx
                    .world
                    .get::<&mut crate::components::PlayerInventory>(player)
                    .ok()
                    .map(|mut inventory| {
                        let previous = inventory.slots[slot];
                        inventory.set_slot(slot, carried);
                        previous
                    });
                if replaced.is_some() {
                    mark_inventory_dirty(ctx, player);
                }
            }
            InventoryCommand::Drop { player, slot, amount } => {
                let dropped = ctx
                    .world
                    .get::<&mut crate::components::PlayerInventory>(player)
                    .ok()
                    .and_then(|mut inventory| drop_from_slot(&mut inventory, slot, amount));
                if let Some(stack) = dropped {
                    mark_inventory_dirty(ctx, player);
                    let position = ctx
                        .world
                        .get::<&crate::components::Transform>(player)
                        .map(|transform| drop_position_in_front(&*transform))
                        .unwrap_or_default();
                    spawn_drop_at(ctx, position, stack, PLAYER_DROP_PICKUP_DELAY_TICKS);
                }
            }
        }
    }
}

fn despawn_world_item(ctx: &mut SystemContext<'_>, entity: Entity) {
    let world_id = ctx
        .world
        .get::<&WorldItemId>(entity)
        .ok()
        .map(|id| *id);
    if let Some(id) = world_id {
        if let Some(book) = ctx.resources.get_mut::<WorldItemBook>() {
            book.remove(id);
        }
    }
    ctx.commands.push(move |world| {
        let _ = world.despawn(entity);
    });
}
