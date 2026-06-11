use engine_world::{BlockId, BlockPos};
use glam::{IVec3, Vec3};

/// Player intent to change a block; authoritative systems apply via `WorldMutationQueue`.
#[derive(Debug, Clone, Copy)]
pub struct BlockChangeIntent {
    pub position: BlockPos,
    pub new_block: BlockId,
}

/// Active mining progress for client overlay. `progress < 0` clears the overlay.
#[derive(Debug, Clone, Copy)]
pub struct BlockMiningProgress {
    pub position: BlockPos,
    pub face_normal: IVec3,
    pub progress: f32,
}

/// Emitted when a block is fully mined. Drop resolution uses `harvested`.
#[derive(Debug, Clone, Copy)]
pub struct BlockBroken {
    pub position: BlockPos,
    pub block_id: BlockId,
    pub harvested: bool,
}

/// Emitted after player transform changes for net translation.
#[derive(Debug, Clone, Copy)]
pub struct PlayerStateChanged {
    pub player_id: u32,
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundKind {
    BlockBreak,
    BlockPlace,
    BlockDigHit,
    PlayerFootstep,
    PlayerFall { big: bool },
}

impl SoundKind {
    pub fn manifest_key(self) -> &'static str {
        match self {
            Self::BlockBreak => "block_break",
            Self::BlockPlace => "block_place",
            Self::BlockDigHit => "block_dig_hit",
            Self::PlayerFootstep => "player_footstep",
            Self::PlayerFall { .. } => "player_fall",
        }
    }

    pub fn manifest_sound_group(self) -> Option<&'static str> {
        match self {
            Self::PlayerFall { big: true } => Some("big"),
            Self::PlayerFall { big: false } => Some("small"),
            _ => None,
        }
    }
}

/// Audio-agnostic cue consumed by the client sound system.
#[derive(Debug, Clone, Copy)]
pub struct SoundCue {
    pub kind: SoundKind,
    pub position: Vec3,
    pub block_id: Option<BlockId>,
}
