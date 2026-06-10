use engine_core::SystemContext;

use crate::components::{HeldTool, NetPlayerId, Player};
use crate::input::resolve_input;
use crate::play_mode::{ActivePlayMode, PlayMode};

pub fn held_tool_select_system(ctx: &mut SystemContext<'_>) {
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| mode.0 != PlayMode::Survival)
    {
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
        if let Ok(mut held) = ctx.world.get::<&mut HeldTool>(entity) {
            held.selected = input.tool_slot.min(8);
        }
    }
}
