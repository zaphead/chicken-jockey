use glam::Vec3;

#[derive(Debug, Clone, Copy, Default)]
pub struct Listener {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialMix {
    pub volume: f32,
    /// 0.0 = hard left, 1.0 = hard right (Kira maps to centered pan internally).
    pub pan: f32,
}

pub fn mix_for_listener(
    listener: &Listener,
    sound_position: Vec3,
    attenuation: super::Attenuation,
) -> SpatialMix {
    let to_sound = sound_position - listener.position;
    let distance = to_sound.length();

    let volume = match attenuation {
        super::Attenuation::None => 1.0,
        super::Attenuation::Linear { max_distance } => {
            if max_distance <= 0.0 {
                1.0
            } else {
                (1.0 - distance / max_distance).clamp(0.0, 1.0)
            }
        }
    };

    let forward = Vec3::new(listener.forward.x, listener.forward.y, 0.0).normalize_or_zero();
    if forward.length_squared() < 1e-6 {
        return SpatialMix { volume, pan: 0.5 };
    }
    let right = Vec3::new(forward.y, -forward.x, 0.0);
    let flat = Vec3::new(to_sound.x, to_sound.y, 0.0);
    if flat.length_squared() < 1e-6 {
        return SpatialMix { volume, pan: 0.5 };
    }
    let pan = (flat.normalize().dot(right) * 0.5 + 0.5).clamp(0.0, 1.0);
    SpatialMix { volume, pan }
}
