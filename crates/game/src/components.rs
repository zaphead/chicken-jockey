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

#[derive(Debug, Clone, Copy)]
pub struct HeldTool {
    pub slots: [Option<ToolId>; 9],
    pub selected: u8,
}

impl Default for HeldTool {
    fn default() -> Self {
        Self {
            slots: [None; 9],
            selected: 0,
        }
    }
}

impl HeldTool {
    pub fn starter_loadout(pickaxe_id: ToolId) -> Self {
        let mut slots = [None; 9];
        slots[1] = Some(pickaxe_id);
        Self { slots, selected: 0 }
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.slots[self.selected as usize]
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
