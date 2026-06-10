use glam::Vec3;
use hecs::Entity;

#[derive(Debug, Clone, Copy)]
pub struct Player;

#[derive(Debug, Clone, Copy)]
pub struct NetPlayerId(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub Vec3);

/// Last rendered local-player eye pose (client Extract). Block interaction uses this so
/// clicks match the crosshair rather than the raw sim transform.
#[derive(Debug, Clone, Copy, Default)]
pub struct DisplayedPlayerView {
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LocomotionState {
    pub on_ground: bool,
    pub was_on_ground: bool,
    pub jump_cooldown: u8,
    pub horizontal_tick_accum: f32,
    pub place_cooldown: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub half_extents: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Chicken {
    pub wander_timer: f32,
    pub wander_direction: Vec3,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Mountable;

#[derive(Debug, Clone, Copy)]
pub struct Mounted {
    pub mount: Entity,
}

#[derive(Debug, Clone, Copy)]
pub struct Rider {
    pub rider: Entity,
}

#[derive(Debug, Clone, Copy)]
pub struct Renderable {
    pub color: [f32; 3],
    pub size: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WorldInitialized(pub bool);

#[derive(Debug, Clone, Copy, Default)]
pub struct TerrainGeneration {
    pub complete: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldSeed(pub u32);

impl WorldSeed {
    pub fn random() -> Self {
        Self(rand::random())
    }
}
