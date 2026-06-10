//! Shared gameplay logic for client and server.

mod components;
mod events;
mod input;
mod mode;
mod plugin;
pub mod systems;

pub use components::{TerrainGeneration, *};
pub use events::{BlockChangeIntent, PlayerStateChanged};
pub use input::{local_player_entity, GameplayInput, LocalPlayerId, PlayerInputs};
pub use mode::{AuthoritativeServer, NetworkClient};
pub use plugin::{
    register_authoritative_block_system, register_client_systems, register_game_systems,
    register_local_client_systems, register_network_client_systems, register_player_systems,
    register_server_systems, register_shared_systems,
};
pub use systems::terrain::WORLD_RADIUS;
pub use systems::spawn_net_player;
