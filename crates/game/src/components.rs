use engine_assets::{ItemStack, ToolId};
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
    pub slots: [Option<ItemStack>; INVENTORY_SLOTS],
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
        slots[1] = Some(ItemStack::tool(pickaxe_id));
        Self {
            slots,
            selected_hotbar: 0,
        }
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.active_stack().and_then(|stack| match stack.kind {
            engine_assets::ItemKind::Tool(id) => Some(id),
            engine_assets::ItemKind::Block { .. } => None,
        })
    }

    pub fn active_block(&self) -> Option<(BlockId, engine_world::BlockState)> {
        self.active_stack().and_then(|stack| match stack.kind {
            engine_assets::ItemKind::Block { id, state } if stack.count > 0 => Some((id, state)),
            _ => None,
        })
    }

    pub fn active_stack(&self) -> Option<ItemStack> {
        self.slots[self.selected_hotbar as usize]
    }

    pub fn slot(&self, index: usize) -> Option<ItemStack> {
        self.slots.get(index).copied().flatten()
    }

    pub fn set_slot(&mut self, index: usize, item: Option<ItemStack>) {
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

#[derive(Debug, Clone, Copy)]
pub struct DroppedItem {
    pub stack: ItemStack,
    pub pickup_delay_ticks: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldItemId(pub u32);

#[derive(Debug, Clone, Copy, Default)]
pub struct InventoryDirty;

#[derive(Debug, Clone, Copy, Default)]
pub struct BlockMiningState {
    pub target: Option<BlockPos>,
    pub target_block: BlockId,
    pub face_normal: IVec3,
    pub progress: f32,
}

/// Head can turn this far each way before the torso catches up (radians).
pub const PLAYER_MAX_HEAD_YAW: f32 = 20.0_f32.to_radians();

#[derive(Debug, Clone, Copy)]
pub struct PlayerAnimation {
    pub limb_swing: f32,
    pub limb_swing_amount: f32,
    /// Horizontal facing used for the torso (Z rotation); head turns ahead of this.
    pub body_yaw: f32,
    /// Blend weight for the right-arm digging pose (0 = walk, 1 = dig).
    pub dig_amount: f32,
    /// Phase of the digging oval swing (radians).
    pub dig_phase: f32,
    /// Blend weight for the right-arm place pose (0 = walk, 1 = place swing).
    pub place_amount: f32,
    /// Phase of the place oval swing (radians).
    pub place_phase: f32,
    /// Radians left in the current one-shot place swing (0 = idle).
    pub place_loop_remaining: f32,
}

impl Default for PlayerAnimation {
    fn default() -> Self {
        Self {
            limb_swing: 0.0,
            limb_swing_amount: 0.0,
            body_yaw: 0.0,
            dig_amount: 0.0,
            dig_phase: 0.0,
            place_amount: 0.0,
            place_phase: 0.0,
            place_loop_remaining: 0.0,
        }
    }
}

impl PlayerAnimation {
    pub fn trigger_place_swing(&mut self) {
        use std::f32::consts::TAU;
        self.place_phase = 0.0;
        self.place_loop_remaining = TAU;
    }
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
