use std::collections::HashMap;

use engine_core::SystemContext;
use glam::Vec2;

#[derive(Debug, Default, Clone)]
pub struct GameplayInput {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub vertical_axis: f32,
    pub sprint: bool,
    pub jump: bool,
    pub interact: bool,
    pub break_block: bool,
    pub place_block: bool,
    pub tool_slot: u8,
}

impl GameplayInput {
    pub fn clear_frame(&mut self) {
        self.look_delta = Vec2::ZERO;
        self.jump = false;
        self.interact = false;
        self.break_block = false;
        self.place_block = false;
    }
}

#[derive(Debug, Default)]
pub struct PlayerInputs {
    inputs: HashMap<u32, GameplayInput>,
}

impl PlayerInputs {
    pub fn set(&mut self, player_id: u32, input: GameplayInput) {
        self.inputs.insert(player_id, input);
    }

    pub fn get(&self, player_id: u32) -> Option<GameplayInput> {
        self.inputs.get(&player_id).cloned()
    }

    pub fn clear_frame(&mut self) {
        for input in self.inputs.values_mut() {
            input.clear_frame();
        }
    }

    pub fn clear_look(&mut self, player_id: u32) {
        if let Some(input) = self.inputs.get_mut(&player_id) {
            input.look_delta = Vec2::ZERO;
        }
    }
}

/// Local client's assigned network player id (None until Welcome in network mode).
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalPlayerId {
    pub id: Option<u32>,
    pub spawned: bool,
}

pub fn resolve_input(ctx: &SystemContext<'_>, net_id: Option<u32>) -> Option<GameplayInput> {
    let player_id =
        net_id.or_else(|| ctx.resources.get::<LocalPlayerId>().and_then(|local| local.id))?;
    ctx.resources.get::<PlayerInputs>()?.get(player_id)
}

pub fn local_player_entity(ctx: &SystemContext<'_>) -> Option<hecs::Entity> {
    let local_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id);

    if let Some(local_id) = local_id {
        for (entity, (id,)) in ctx
            .world
            .query::<(&crate::components::NetPlayerId,)>()
            .iter()
        {
            if id.0 == local_id {
                return Some(entity);
            }
        }
        return None;
    }

    ctx.world
        .query::<&crate::components::Player>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
}
