//! Spectator fly camera — frame-delta thrust and ice drag (not MC survival physics).

use glam::Vec3;

pub const BASE_WALK_SPEED: f32 = 5.1;
pub const SPECTATOR_SPEED_SCALE: f32 = 2.0;
pub const SPECTATOR_SPRINT_MULT: f32 = 3.0;
pub const SPECTATOR_ACCEL: f32 = 90.0;
/// Lower = more coast when movement input is released.
pub const SPECTATOR_DRAG: f32 = 1.6;

pub fn max_fly_speed(sprint: bool) -> f32 {
    BASE_WALK_SPEED * SPECTATOR_SPEED_SCALE * if sprint {
        SPECTATOR_SPRINT_MULT
    } else {
        1.0
    }
}

pub fn accelerate_toward(current: Vec3, target: Vec3, delta: f32) -> Vec3 {
    let max_step = SPECTATOR_ACCEL * delta;
    let delta_v = target - current;
    let dist = delta_v.length();
    if dist > max_step && dist > 0.0 {
        current + delta_v / dist * max_step
    } else {
        target
    }
}

pub fn apply_ice_drag(velocity: Vec3, delta: f32) -> Vec3 {
    velocity * (-SPECTATOR_DRAG * delta).exp()
}
