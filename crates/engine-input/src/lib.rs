//! Input action mapping and polling state.

mod actions;
mod state;

pub use actions::{Action, InputState};
pub use state::{apply_mouse_motion, apply_winit_event};
