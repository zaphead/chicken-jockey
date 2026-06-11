use engine_assets::ToolId;
use glam::{IVec3, Vec3};
use hecs::Entity;
use engine_world::{BlockId, BlockPos};

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

pub const HOTBAR_SLOTS: usize = 9;
pub const MAIN_INVENTORY_SLOTS: usize = 27;
pub const INVENTORY_SLOTS: usize = HOTBAR_SLOTS + MAIN_INVENTORY_SLOTS;

#[derive(Debug, Clone, Copy)]
pub struct PlayerInventory {
    pub slots: [Option<ToolId>; INVENTORY_SLOTS],
    pub selected_hotbar: u8,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            slots: [None; INVENTORY_SLOTS],
            selected_hotbar: 0,
        }
    }
}

impl PlayerInventory {
    pub fn starter_loadout(pickaxe_id: ToolId) -> Self {
        let mut slots = [None; INVENTORY_SLOTS];
        slots[1] = Some(pickaxe_id);
        Self {
            slots,
            selected_hotbar: 0,
        }
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.slots[self.selected_hotbar as usize]
    }

    pub fn slot(&self, index: usize) -> Option<ToolId> {
        self.slots.get(index).copied().flatten()
    }

    pub fn set_slot(&mut self, index: usize, item: Option<ToolId>) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = item;
        }
    }

    pub fn hotbar_slot_index(hotbar: usize) -> usize {
        hotbar
    }

    pub fn main_slot_index(main: usize) -> usize {
        HOTBAR_SLOTS + main
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BlockMiningState {
    pub target: Option<BlockPos>,
    pub target_block: BlockId,
    pub face_normal: IVec3,
    pub progress: f32,
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
