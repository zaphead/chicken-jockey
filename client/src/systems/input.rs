use engine_core::SystemContext;
use engine_input::InputState;
use engine_input::DropHotbarRequest;
use game::{
    apply_look_delta, local_player_entity, ActivePlayMode, DropAmount, GameplayInput, LocalPlayerId,
    PlayMode, PlayerInputs, Transform,
};

fn map_drop_hotbar(request: DropHotbarRequest) -> DropAmount {
    match request {
        DropHotbarRequest::One => DropAmount::One,
        DropHotbarRequest::Half => DropAmount::Half,
        DropHotbarRequest::All => DropAmount::All,
    }
}

use crate::systems::ui_state::ClientUiState;

pub struct PendingWinitInput(pub InputState);

pub fn apply_local_look_system(ctx: &mut SystemContext<'_>) {
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| mode.0 != PlayMode::Survival)
    {
        return;
    }

    let player_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id)
        .unwrap_or(0);

    let Some(input) = ctx
        .resources
        .get::<PlayerInputs>()
        .and_then(|inputs| inputs.get(player_id))
    else {
        return;
    };

    if input.look_delta.length_squared() == 0.0 {
        return;
    }

    let Some(entity) = local_player_entity(ctx) else {
        return;
    };

    if let Ok(mut transform) = ctx.world.get::<&mut Transform>(entity) {
        apply_look_delta(&mut transform, input.look_delta);
    }

    if let Some(inputs) = ctx.resources.get_mut::<PlayerInputs>() {
        inputs.clear_look(player_id);
    }
}

pub fn sync_local_input_system(ctx: &mut SystemContext<'_>) {
    let pending = ctx.resources.get::<PendingWinitInput>().expect("pending input");
    let player_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id)
        .unwrap_or(0);

    let survival = ctx
        .resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.0 == PlayMode::Survival);
    let modal_open = ctx
        .resources
        .get::<ClientUiState>()
        .is_some_and(|ui| ui.blocks_world());

    let gameplay = if modal_open {
        GameplayInput::default()
    } else {
        GameplayInput {
        move_axis: pending.0.move_axis,
        look_delta: pending.0.look_delta,
        vertical_axis: pending.0.vertical_axis(),
        sprint: pending.0.sprint,
        jump: pending.0.jump || (survival && pending.0.ascend),
        interact: pending.0.interact,
        break_block: pending.0.break_held,
        place_block: pending.0.place_held,
        tool_slot: pending.0.selected_tool_slot,
        drop_hotbar: pending.0.drop_hotbar.map(map_drop_hotbar),
        }
    };

    if let Some(inputs) = ctx.resources.get_mut::<PlayerInputs>() {
        inputs.set(player_id, gameplay);
    }
}
