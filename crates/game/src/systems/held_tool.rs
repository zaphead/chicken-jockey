use engine_core::SystemContext;

use crate::components::{NetPlayerId, Player, PlayerInventory};
use crate::input::resolve_input;
use crate::play_mode::survival_active;

pub fn held_tool_select_system(ctx: &mut SystemContext<'_>) {
    if !survival_active(ctx) {
        return;
    }

    let players: Vec<(hecs::Entity, Option<u32>)> = ctx
        .world
        .query::<(&Player, Option<&NetPlayerId>)>()
        .iter()
        .map(|(entity, (_, net_id))| (entity, net_id.map(|id| id.0)))
        .collect();

    for (entity, net_id) in players {
        let Some(input) = resolve_input(ctx, net_id) else {
            continue;
        };
        if let Ok(mut inventory) = ctx.world.get::<&mut PlayerInventory>(entity) {
            inventory.selected_hotbar = input.tool_slot.min(8);
        }
    }
}
