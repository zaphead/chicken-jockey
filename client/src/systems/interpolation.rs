use engine_core::SystemContext;
use glam::Vec3;

use game::{local_player_entity, Transform};

#[derive(Debug, Clone, Copy, Default)]
pub struct TransformSnapshot {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl From<&Transform> for TransformSnapshot {
    fn from(transform: &Transform) -> Self {
        Self {
            position: transform.position,
            yaw: transform.yaw,
            pitch: transform.pitch,
        }
    }
}

/// Previous sim pose for the local player (client render interpolation only).
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviousPlayerTransform(pub Option<TransformSnapshot>);

/// Stores the latest sim pose after Extract for next frame's interpolation.
pub fn commit_player_transform_snapshot_system(ctx: &mut SystemContext<'_>) {
    let Some(entity) = local_player_entity(ctx) else {
        return;
    };
    let Ok(transform) = ctx.world.get::<&Transform>(entity) else {
        return;
    };
    if let Some(prev) = ctx.resources.get_mut::<PreviousPlayerTransform>() {
        prev.0 = Some(TransformSnapshot::from(&*transform));
    }
}

pub fn lerp_transform_snapshot(
    previous: TransformSnapshot,
    current: &Transform,
    alpha: f32,
) -> TransformSnapshot {
    TransformSnapshot {
        position: previous.position.lerp(current.position, alpha),
        yaw: previous.yaw + (current.yaw - previous.yaw) * alpha,
        pitch: previous.pitch + (current.pitch - previous.pitch) * alpha,
    }
}
