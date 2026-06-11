use engine_assets::ToolId;

use crate::systems::menu::PauseScreen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientModal {
    #[default]
    None,
    Pause(PauseScreen),
    Inventory,
}

#[derive(Debug, Clone, Default)]
pub struct ClientUiState {
    pub modal: ClientModal,
    pub carried: Option<ToolId>,
}

impl ClientUiState {
    pub fn blocks_world(&self) -> bool {
        !matches!(self.modal, ClientModal::None)
    }

    pub fn pause_screen(&self) -> Option<PauseScreen> {
        match self.modal {
            ClientModal::Pause(screen) => Some(screen),
            _ => None,
        }
    }
}
