use glam::Vec2;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 4242;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDelta {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u16,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub jump: bool,
    pub interact: bool,
    pub break_block: bool,
    pub place_block: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub player_id: u32,
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    Welcome { player_id: u32 },
    BlockDeltas(Vec<BlockDelta>),
    EntitySnapshots(Vec<EntitySnapshot>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Join,
    Input(PlayerInput),
}
