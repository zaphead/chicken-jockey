use engine_world::{BlockId, BlockPos};

/// Player intent to change a block; authoritative systems apply via `WorldMutationQueue`.
#[derive(Debug, Clone, Copy)]
pub struct BlockChangeIntent {
    pub position: BlockPos,
    pub new_block: BlockId,
}

/// Emitted after player transform changes for net translation.
#[derive(Debug, Clone, Copy)]
pub struct PlayerStateChanged {
    pub player_id: u32,
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
}
