mod minecraft;
mod spectator;

use glam::{Vec2, Vec3};

use crate::axes::{horizontal_forward, horizontal_right, UP};
use crate::components::Transform;

pub use minecraft::{
    apply_vertical_post_move, jump_cooldown_sim_steps, update_horizontal_velocity,
    JUMP_VELOCITY, McMovementInput, MC_TICK_DT,
};
pub use spectator::{accelerate_toward, apply_ice_drag, max_fly_speed};

pub const MOUSE_SENSITIVITY: f32 = 0.0012;

pub fn apply_look_delta(transform: &mut Transform, look_delta: Vec2) {
    transform.yaw += look_delta.x * MOUSE_SENSITIVITY;
    transform.pitch =
        (transform.pitch - look_delta.y * MOUSE_SENSITIVITY).clamp(-1.5, 1.5);
}

pub fn wish_direction_horizontal(yaw: f32, move_axis: Vec2) -> Vec3 {
    let forward = horizontal_forward(yaw);
    let right = horizontal_right(yaw);
    (forward * move_axis.y + right * move_axis.x).normalize_or_zero()
}

pub fn wish_direction_fly(yaw: f32, move_axis: Vec2, vertical_axis: f32) -> Vec3 {
    let horizontal = wish_direction_horizontal(yaw, move_axis);
    let vertical = UP * vertical_axis;
    (horizontal + vertical).normalize_or_zero()
}
