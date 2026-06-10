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
    pub next_column: i32,
    pub complete: bool,
}
