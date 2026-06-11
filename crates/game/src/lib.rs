//! Shared gameplay logic for client and server.

mod axes;
mod components;
pub mod day_night;
mod debug_world;
mod events;
mod input;
mod mode;
mod mining;
mod movement;
mod play_mode;
mod plugin;
pub mod systems;
mod voxel_raycast;

pub use axes::{
    grounded_probe_offset, horizontal_forward, horizontal_right, view_forward,
    PLAYER_EYE_OFFSET_Z, PLAYER_HALF_EXTENTS, UP,
};
pub use components::{DisplayedPlayerView, TerrainGeneration, *};
pub use events::{BlockChangeIntent, BlockMiningProgress, PlayerStateChanged};
pub use mining::{destroy_stage, tool_label_for_inventory};
pub use input::{
    local_player_entity, resolve_input, GameplayInput, LocalPlayerId, PlayerInputs,
};
pub use mode::{AuthoritativeServer, NetworkClient};
pub use movement::{
    accelerate_toward, apply_ice_drag, apply_look_delta, max_fly_speed, wish_direction_fly,
    wish_direction_horizontal, MOUSE_SENSITIVITY,
};
pub use day_night::{
    build_lighting_snapshot, format_time_of_day, time_of_day_label, DayNightCycle,
    DEFAULT_DAY_LENGTH_SECS, LightingSnapshot,
};
pub use debug_world::{iter_mesh_chunks, ActiveDebugWorld, DebugWorldKind};
pub use play_mode::{ActivePlayMode, PlayMode};
pub use plugin::{
    register_authoritative_block_system, register_local_client_systems,
    register_network_client_systems, register_player_systems, register_player_spawn_systems,
    register_server_systems, register_world_systems,
};
pub use systems::terrain::{
    player_ground_center_z_at, player_spawn_center_z, player_spawn_center_z_at,
    terrain_surface_z, FLAT_SURFACE_Z, FLAT_WORLD_RADIUS, GRASS_PLANE_Z,
    PLAYER_SPAWN_PITCH, WORLD_RADIUS,
};
pub use systems::spawn_net_player;
pub use voxel_raycast::{
    block_overlaps_player, camera_interaction_ray, player_interaction_ray, raycast_voxel,
    VoxelRayHit, BLOCK_REACH,
};
