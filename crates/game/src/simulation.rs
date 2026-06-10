use std::collections::HashMap;

use engine_input::InputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimulationMode {
    #[default]
    Local,
    AuthoritativeServer,
    NetworkClient,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocalPlayer {
    pub id: Option<u32>,
    pub spawned: bool,
}

#[derive(Debug, Default)]
pub struct RemoteInputs {
    inputs: HashMap<u32, InputState>,
}

impl RemoteInputs {
    pub fn set(&mut self, player_id: u32, input: InputState) {
        self.inputs.insert(player_id, input);
    }

    pub fn get(&self, player_id: u32) -> Option<InputState> {
        self.inputs.get(&player_id).cloned()
    }

    pub fn clear_frame(&mut self) {
        for input in self.inputs.values_mut() {
            input.clear_frame_state();
        }
    }
}
