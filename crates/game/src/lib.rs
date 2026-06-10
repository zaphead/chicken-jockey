//! Shared gameplay logic for client and server.

mod components;
mod plugin;
pub mod simulation;
pub mod systems;

pub use components::{TerrainGeneration, *};
pub use plugin::{
    register_client_systems, register_game_systems, register_player_systems,
    register_server_systems, register_shared_systems,
};
pub use simulation::{LocalPlayer, RemoteInputs, SimulationMode};
pub use systems::terrain::WORLD_RADIUS;
pub use systems::spawn_net_player;
