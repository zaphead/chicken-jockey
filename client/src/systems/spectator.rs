use engine_core::SystemContext;
use game::{
    player_ground_center_z_at, resolve_input, accelerate_toward, apply_ice_drag, max_fly_speed,
    wish_direction_fly, ActivePlayMode, DebugWorldKind, LocalPlayerId, MOUSE_SENSITIVITY,
    PlayMode,
};
use glam::Vec3;

pub fn reset_spectator_for_world(world: DebugWorldKind, seed: u32) -> SpectatorCamera {
    SpectatorCamera {
        position: Vec3::new(0.5, 0.5, player_ground_center_z_at(0, 0, world, seed) + 2.0),
        velocity: Vec3::ZERO,
        yaw: 0.0,
        pitch: -0.25,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpectatorCamera {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for SpectatorCamera {
    fn default() -> Self {
        reset_spectator_for_world(DebugWorldKind::Flat, 0)
    }
}

pub fn spectator_camera_system(ctx: &mut SystemContext<'_>) {
    if ctx
        .resources
        .get::<ActivePlayMode>()
        .is_some_and(|mode| mode.0 != PlayMode::Spectator)
    {
        return;
    }

    let delta = ctx
        .resources
        .get::<engine_core::Time>()
        .map(|t| t.frame_delta)
        .unwrap_or(0.0);
    let player_id = ctx
        .resources
        .get::<LocalPlayerId>()
        .and_then(|local| local.id)
        .unwrap_or(0);
    let Some(input) = resolve_input(ctx, Some(player_id)) else {
        return;
    };
    let Some(camera) = ctx.resources.get_mut::<SpectatorCamera>() else {
        return;
    };

    camera.yaw += input.look_delta.x * MOUSE_SENSITIVITY;
    camera.pitch = (camera.pitch - input.look_delta.y * MOUSE_SENSITIVITY).clamp(-1.5, 1.5);

    let wish = wish_direction_fly(camera.yaw, input.move_axis, input.vertical_axis);
    let speed = max_fly_speed(input.sprint);

    if wish.length_squared() > 0.0 {
        let target = wish * speed;
        camera.velocity = accelerate_toward(camera.velocity, target, delta);
    } else {
        camera.velocity = apply_ice_drag(camera.velocity, delta);
    }

    camera.position += camera.velocity * delta;

    if let Some(inputs) = ctx.resources.get_mut::<game::PlayerInputs>() {
        inputs.clear_look(player_id);
    }
}
