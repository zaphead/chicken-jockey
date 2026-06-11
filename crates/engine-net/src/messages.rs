use glam::Vec2;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 4242;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDelta {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: u16,
    #[serde(default)]
    pub state: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DropAmountWire {
    One,
    Half,
    All,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub jump: bool,
    pub interact: bool,
    pub break_block: bool,
    pub place_block: bool,
    #[serde(default)]
    pub tool_slot: u8,
    #[serde(default)]
    pub drop_hotbar: Option<DropAmountWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemStackWire {
    Block {
        id: u16,
        state: u8,
        count: u16,
    },
    Tool {
        id: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldItemSnapshot {
    pub id: u32,
    pub position: [f32; 3],
    pub stack: ItemStackWire,
}

pub const INVENTORY_SLOT_COUNT: usize = 36;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySync {
    pub player_id: u32,
    pub slots: Vec<Option<ItemStackWire>>,
    pub selected: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryAction {
    MoveSlot { from: u8, to: u8 },
    QuickMove { slot: u8 },
    SwapWithCarried {
        slot: u8,
        carried: Option<ItemStackWire>,
    },
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
    WorldItems(Vec<WorldItemSnapshot>),
    InventorySync(InventorySync),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    Join,
    Input(PlayerInput),
    InventoryActions(Vec<InventoryAction>),
}
