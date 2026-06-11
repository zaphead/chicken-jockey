//! Input action mapping and polling state.

mod actions;
mod state;

pub use actions::{Action, DropHotbarRequest, InputState};
pub use state::{apply_mouse_motion, apply_winit_event};
