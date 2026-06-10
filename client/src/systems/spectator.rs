use engine_core::SystemContext;
use engine_render::Camera;
use game::{player_spawn_center_z, UP};
use glam::Vec3;

use crate::systems::input::PendingWinitInput;

const FLY_SPEED: f32 = 4.0;
const SPRINT_MULTIPLIER: f32 = 2.0;
const MOUSE_SENSITIVITY: f32 = 0.0012;

#[derive(Debug, Clone, Copy)]
pub struct SpectatorCamera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

fn default_spectator_position() -> Vec3 {
    Vec3::new(0.5, 0.5, player_spawn_center_z() + 4.0)
}

impl Default for SpectatorCamera {
    fn default() -> Self {
        Self {
            position: default_spectator_position(),
            yaw: 0.0,
            pitch: -0.35,
        }
    }
}

pub fn spectator_camera_system(ctx: &mut SystemContext<'_>) {
    let delta = ctx.resources.get::<engine_core::Time>().map(|t| t.delta).unwrap_or(0.0);
    let (look_delta, move_axis, vertical_axis, sprint) = ctx
        .resources
        .get::<PendingWinitInput>()
        .map(|pending| {
            (
                pending.0.look_delta,
                pending.0.move_axis,
                pending.0.vertical_axis(),
                pending.0.sprint,
            )
        })
        .unwrap_or_default();
    let Some(camera) = ctx.resources.get_mut::<SpectatorCamera>() else {
        return;
    };

    camera.yaw += look_delta.x * MOUSE_SENSITIVITY;
    camera.pitch = (camera.pitch - look_delta.y * MOUSE_SENSITIVITY).clamp(-1.5, 1.5);

    let render_cam = Camera {
        position: camera.position,
        yaw: camera.yaw,
        pitch: camera.pitch,
        ..Camera::default()
    };

    let mut forward = render_cam.forward();
    forward.z = 0.0;
    if forward.length_squared() > 0.0 {
        forward = forward.normalize();
    }
    let mut right = render_cam.right();
    right.z = 0.0;
    if right.length_squared() > 0.0 {
        right = right.normalize();
    }

    let wish = forward * move_axis.y
        + right * move_axis.x
        + UP * vertical_axis;
    if wish.length_squared() > 0.0 {
        let speed = FLY_SPEED * if sprint { SPRINT_MULTIPLIER } else { 1.0 };
        camera.position += wish.normalize() * speed * delta;
    }
}
