use glam::Vec3;

/// Minecraft-style day length in ticks (one full cycle).
pub const DAY_TICKS: f32 = 24_000.0;
/// Default real-time duration for one full day (10 minutes).
pub const DEFAULT_DAY_LENGTH_SECS: f32 = 600.0;
pub const MOON_PHASE_COUNT: u8 = 8;

/// Authoritative world clock; advances on client and server.
#[derive(Debug, Clone)]
pub struct DayNightCycle {
    pub world_time: f32,
    pub day_length_secs: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            world_time: 6_000.0,
            day_length_secs: DEFAULT_DAY_LENGTH_SECS,
        }
    }
}

/// Snapshot consumed by the renderer after extract.
#[derive(Debug, Clone, Copy)]
pub struct LightingSnapshot {
    pub sun_dir: Vec3,
    pub moon_dir: Vec3,
    pub sun_color: Vec3,
    pub moon_color: Vec3,
    pub ambient_color: Vec3,
    pub horizon_color: Vec3,
    pub sun_strength: f32,
    pub moon_strength: f32,
    pub star_visibility: f32,
    pub night_darkness: f32,
    pub moon_phase: u8,
    pub world_time: f32,
    pub sun_elevation: f32,
}

struct LightingPalette {
    sunrise_sun: Vec3,
    day_sun: Vec3,
    sunset_sun: Vec3,
    sunrise_amb: Vec3,
    day_amb: Vec3,
    sunset_amb: Vec3,
    night_amb: Vec3,
    sunrise_horizon: Vec3,
    day_horizon: Vec3,
    sunset_horizon: Vec3,
    night_horizon: Vec3,
    moon_color: Vec3,
}

impl Default for LightingPalette {
    fn default() -> Self {
        Self {
            sunrise_sun: Vec3::new(0.95, 0.5, 0.22),
            day_sun: Vec3::new(0.82, 0.78, 0.68),
            sunset_sun: Vec3::new(0.9, 0.38, 0.12),
            sunrise_amb: Vec3::new(0.22, 0.2, 0.28),
            day_amb: Vec3::new(0.32, 0.36, 0.42),
            sunset_amb: Vec3::new(0.28, 0.22, 0.26),
            // Baked former GPU night_dim (0.36 floor) into CPU ambient.
            night_amb: Vec3::new(0.019, 0.021, 0.030),
            sunrise_horizon: Vec3::new(0.95, 0.55, 0.32),
            day_horizon: Vec3::new(0.55, 0.72, 0.92),
            sunset_horizon: Vec3::new(0.92, 0.38, 0.18),
            night_horizon: Vec3::new(0.06, 0.075, 0.14),
            moon_color: Vec3::new(0.35, 0.42, 0.55),
        }
    }
}

struct PhaseWeights {
    dawn: f32,
    day: f32,
    dusk: f32,
    night: f32,
    sun_strength: f32,
    night_darkness: f32,
    star_visibility: f32,
}

impl PhaseWeights {
    fn from_sun_elevation(sun_elevation: f32) -> Self {
        let sun_strength = smoothstep(-0.18, 0.22, sun_elevation);
        let night_darkness = smoothstep(0.10, -0.14, sun_elevation);
        let star_visibility = smoothstep(0.05, -0.20, sun_elevation);

        let golden = (1.0 - smoothstep(0.0, 0.22, sun_elevation.abs()))
            * smoothstep(-0.18, 0.08, sun_elevation);
        let dawn = golden * smoothstep(0.0, 0.12, sun_elevation);
        let dusk = golden * smoothstep(0.12, 0.0, sun_elevation);
        let day = sun_strength * (1.0 - dawn - dusk).max(0.0);
        let night = night_darkness;

        Self {
            dawn,
            day,
            dusk,
            night,
            sun_strength,
            night_darkness,
            star_visibility,
        }
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Human-readable phase label for debug HUD.
pub fn time_of_day_label(world_time: f32) -> &'static str {
    let t = (world_time / DAY_TICKS).rem_euclid(1.0);
    match t {
        t if t < 0.04 || t > 0.96 => "dawn",
        t if t < 0.20 => "morning",
        t if t < 0.42 => "day",
        t if t < 0.52 => "dusk",
        t if t < 0.68 => "evening",
        _ => "night",
    }
}

pub fn format_time_of_day(world_time: f32) -> String {
    format!(
        "TIME {:>5.0} {}",
        world_time,
        time_of_day_label(world_time)
    )
}

/// World-time ticks where ambient music may trigger (morning, noon, evening, midnight).
pub const MUSIC_WORLD_ANCHORS: [f32; 4] = [0.0, 6_000.0, 12_000.0, 18_000.0];

fn anchor_crossed(previous: f32, current: f32, anchor: f32) -> bool {
    if previous <= current {
        previous < anchor && current >= anchor
    } else {
        previous < anchor || current >= anchor
    }
}

/// Anchors crossed when advancing from `previous` to `current` (handles day wrap).
pub fn world_time_crossed_anchors(previous: f32, current: f32) -> Vec<f32> {
    MUSIC_WORLD_ANCHORS
        .into_iter()
        .filter(|&anchor| anchor_crossed(previous, current, anchor))
        .collect()
}

/// Unit vector from world origin toward the sun (Z-up).
pub fn sun_position(world_time: f32) -> Vec3 {
    let phase = (world_time / DAY_TICKS) * std::f32::consts::TAU;
    let horizontal = phase.cos();
    let elevation = phase.sin();
    Vec3::new(0.0, horizontal, elevation).normalize_or_zero()
}

pub fn moon_position(world_time: f32) -> Vec3 {
    let phase = (world_time / DAY_TICKS) * std::f32::consts::TAU + std::f32::consts::PI;
    let horizontal = phase.cos();
    let elevation = phase.sin();
    Vec3::new(0.0, horizontal, elevation).normalize_or_zero()
}

pub fn moon_phase_index(world_time: f32) -> u8 {
    ((world_time / DAY_TICKS) * MOON_PHASE_COUNT as f32) as u8 % MOON_PHASE_COUNT
}

/// Direction light travels (toward surfaces).
pub fn sun_light_dir(world_time: f32) -> Vec3 {
    -sun_position(world_time)
}

pub fn moon_light_dir(world_time: f32) -> Vec3 {
    -moon_position(world_time)
}

pub fn build_lighting_snapshot(world_time: f32) -> LightingSnapshot {
    let sun_pos = sun_position(world_time);
    let moon_pos = moon_position(world_time);
    let sun_elevation = sun_pos.z;
    let weights = PhaseWeights::from_sun_elevation(sun_elevation);
    let palette = LightingPalette::default();

    let mut sun_color =
        palette.sunrise_sun * weights.dawn + palette.day_sun * weights.day + palette.sunset_sun * weights.dusk;
    if sun_color.length_squared() > 0.0 {
        sun_color = sun_color.normalize() * sun_color.length().min(0.85);
    } else {
        sun_color = palette.day_sun * 0.0;
    }

    let ambient_color = palette.sunrise_amb * weights.dawn
        + palette.day_amb * weights.day
        + palette.sunset_amb * weights.dusk
        + palette.night_amb * weights.night;

    let horizon_color = palette.sunrise_horizon * weights.dawn
        + palette.day_horizon * weights.day
        + palette.sunset_horizon * weights.dusk
        + palette.night_horizon * weights.night;

    let moon_strength = smoothstep(-0.08, 0.18, moon_pos.z)
        * (1.0 - weights.sun_strength * 0.95)
        * (0.144 + (0.086 - 0.144) * weights.night_darkness);

    LightingSnapshot {
        sun_dir: sun_light_dir(world_time),
        moon_dir: moon_light_dir(world_time),
        sun_color,
        moon_color: palette.moon_color,
        ambient_color,
        horizon_color,
        sun_strength: weights.sun_strength,
        moon_strength,
        star_visibility: weights.star_visibility,
        night_darkness: weights.night_darkness,
        moon_phase: moon_phase_index(world_time),
        world_time,
        sun_elevation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noon_sun_is_overhead() {
        let pos = sun_position(6_000.0);
        assert!(pos.z > 0.9);
    }

    #[test]
    fn midnight_sun_is_below_horizon() {
        let pos = sun_position(18_000.0);
        assert!(pos.z < -0.5);
    }

    #[test]
    fn midnight_is_dark() {
        let snap = build_lighting_snapshot(18_000.0);
        assert!(snap.night_darkness > 0.8);
        assert!(snap.sun_strength < 0.1);
    }

    #[test]
    fn midnight_ambient_in_expected_band() {
        let snap = build_lighting_snapshot(18_000.0);
        let amb = snap.ambient_color.length();
        assert!(amb > 0.02 && amb < 0.06, "ambient magnitude {amb}");
    }

    #[test]
    fn crosses_noon_advancing_forward() {
        let crossed = world_time_crossed_anchors(5_990.0, 6_010.0);
        assert_eq!(crossed, vec![6_000.0]);
    }

    #[test]
    fn crosses_morning_on_day_wrap() {
        let crossed = world_time_crossed_anchors(23_990.0, 10.0);
        assert!(crossed.contains(&0.0));
    }

    #[test]
    fn no_cross_when_stationary() {
        assert!(world_time_crossed_anchors(6_000.0, 6_000.0).is_empty());
    }

    #[test]
    fn fast_forward_crosses_multiple_anchors() {
        let crossed = world_time_crossed_anchors(0.0, 18_500.0);
        assert_eq!(crossed, vec![6_000.0, 12_000.0, 18_000.0]);
    }
}
