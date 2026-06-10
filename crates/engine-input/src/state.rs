use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::actions::InputState;

pub fn apply_winit_event(input: &mut InputState, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput { event, .. } => apply_keyboard(input, event),
        WindowEvent::MouseInput { state, button, .. } => match button {
            MouseButton::Left => {
                input.break_held = *state == ElementState::Pressed;
            }
            MouseButton::Right => {
                input.place_held = *state == ElementState::Pressed;
            }
            _ => {}
        },
        _ => {}
    }
}

/// Raw mouse deltas from `DeviceEvent::MouseMotion` (use when the cursor is grabbed).
pub fn apply_mouse_motion(input: &mut InputState, delta: (f64, f64)) {
    if input.cursor_locked {
        input.look_delta.x += delta.0 as f32;
        input.look_delta.y += delta.1 as f32;
    }
}

fn apply_keyboard(input: &mut InputState, event: &KeyEvent) {
    let pressed = event.state == ElementState::Pressed;
    let PhysicalKey::Code(code) = event.physical_key else {
        return;
    };

    match code {
        KeyCode::KeyW => input.move_axis.y = if pressed { 1.0 } else { input.move_axis.y.min(0.0) },
        KeyCode::KeyS => input.move_axis.y = if pressed { -1.0 } else { input.move_axis.y.max(0.0) },
        KeyCode::KeyA => input.move_axis.x = if pressed { -1.0 } else { input.move_axis.x.max(0.0) },
        KeyCode::KeyD => input.move_axis.x = if pressed { 1.0 } else { input.move_axis.x.min(0.0) },
        KeyCode::Space => input.ascend = pressed,
        KeyCode::ControlLeft | KeyCode::ControlRight => input.descend = pressed,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => input.sprint = pressed,
        KeyCode::KeyE => {
            if pressed {
                input.interact = true;
            }
        }
        KeyCode::KeyM => {
            if pressed {
                input.toggle_play_mode = true;
            }
        }
        KeyCode::Digit1 if pressed => input.selected_tool_slot = 0,
        KeyCode::Digit2 if pressed => input.selected_tool_slot = 1,
        KeyCode::Digit3 if pressed => input.selected_tool_slot = 2,
        KeyCode::Digit4 if pressed => input.selected_tool_slot = 3,
        KeyCode::Digit5 if pressed => input.selected_tool_slot = 4,
        KeyCode::Digit6 if pressed => input.selected_tool_slot = 5,
        KeyCode::Digit7 if pressed => input.selected_tool_slot = 6,
        KeyCode::Digit8 if pressed => input.selected_tool_slot = 7,
        KeyCode::Digit9 if pressed => input.selected_tool_slot = 8,
        _ => {}
    }
}
