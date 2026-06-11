use engine_core::SystemContext;

use crate::components::{NetPlayerId, Player};
use crate::input::resolve_input;
use crate::inventory::{InventoryCommand, InventoryCommandQueue};
use crate::play_mode::survival_active;

pub fn player_drop_items_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }

    let players: Vec<(hecs::Entity, Option<u32>)> = ctx
        .world
        .query::<(&Player, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, net_id))| (entity, net_id.map(|id| id.0)))
        .collect();

    let mut pending: Vec<InventoryCommand> = Vec::new();

    for (entity, net_id) in players {
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        let Some(amount) = input.drop_hotbar else {
            continue;
        };
        let Ok(inventory) = ctx.world.get::<&crate::components::PlayerInventory>(entity) else {
            continue;
        };
        pending.push(InventoryCommand::Drop {
            player: entity,
            slot: inventory.selected_hotbar,
            amount,
        });
    }

    if let Some(queue) = ctx.resources.get_mut::<InventoryCommandQueue>() {
        for command in pending {
            queue.push(command);
        }
    }
}
