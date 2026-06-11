use engine_render::LightingSnapshot;

pub fn render_lighting(snap: game::LightingSnapshot) -> LightingSnapshot {
    LightingSnapshot {
        sun_dir: snap.sun_dir,
        moon_dir: snap.moon_dir,
        sun_color: snap.sun_color,
        moon_color: snap.moon_color,
        ambient_color: snap.ambient_color,
        horizon_color: snap.horizon_color,
        sun_strength: snap.sun_strength,
        moon_strength: snap.moon_strength,
        star_visibility: snap.star_visibility,
        night_darkness: snap.night_darkness,
        moon_phase: snap.moon_phase,
        world_time: snap.world_time,
        sun_elevation: snap.sun_elevation,
    }
}
