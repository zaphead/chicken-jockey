use engine_core::{SystemContext, Time};
use engine_render::{Camera, DEFAULT_FOV_Y};

use crate::systems::input::PendingWinitInput;
use crate::systems::ui_state::ClientUiState;

pub const ZOOM_MAGNIFICATION: f32 = 3.0;
const ZOOM_ANIM_DURATION: f32 = 0.15;

#[derive(Debug, Clone, Copy)]
pub struct CameraZoom {
    /// 0 = normal, 1 = fully zoomed.
    pub progress: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self { progress: 0.0 }
    }
}

impl CameraZoom {
    pub fn factor(&self) -> f32 {
        1.0 + (ZOOM_MAGNIFICATION - 1.0) * self.progress
    }

    pub fn fov_y(&self) -> f32 {
        DEFAULT_FOV_Y / self.factor()
    }
}

pub fn update_camera_zoom_system(ctx: &mut SystemContext<'_>) {
    let blocked = ctx
        .resources
        .get::<ClientUiState>()
        .is_some_and(|ui| ui.blocks_world());
    let zoom_held = ctx
        .resources
        .get::<PendingWinitInput>()
        .is_some_and(|pending| pending.0.cursor_locked && pending.0.zoom_held && !blocked);
    let target_progress = if zoom_held { 1.0 } else { 0.0 };

    let dt = ctx
        .resources
        .get::<Time>()
        .map(|time| time.frame_delta)
        .unwrap_or(1.0 / 60.0);
    let step = dt / ZOOM_ANIM_DURATION;

    let Some(zoom) = ctx.resources.get_mut::<CameraZoom>() else {
        return;
    };
    if zoom.progress < target_progress {
        zoom.progress = (zoom.progress + step).min(target_progress);
    } else if zoom.progress > target_progress {
        zoom.progress = (zoom.progress - step).max(target_progress);
    }
}

pub fn apply_zoom_to_camera(camera: &mut Camera, zoom: &CameraZoom) {
    camera.fov_y = zoom.fov_y();
}
