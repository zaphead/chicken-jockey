use engine_core::{App, Stage};

use crate::systems::{
    block_interaction_system, block_mining_system, flush_world_mutations_system,
    generate_terrain_system, held_tool_select_system, player_look_system,
    player_locomotion_system, spawn_local_player_system, spawn_network_player_system,
};

pub fn register_world_systems(app: &mut App) {
    app.add_system(Stage::Update, generate_terrain_system);
    app.add_system(Stage::PostUpdate, flush_world_mutations_system);
}

pub fn register_player_spawn_systems(app: &mut App) {
    app.add_system(Stage::PostUpdate, spawn_local_player_system);
    app.add_system(Stage::PostUpdate, spawn_network_player_system);
}

pub fn register_player_look_system(app: &mut App) {
    app.add_system(Stage::Update, player_look_system);
}

pub fn register_player_systems(app: &mut App) {
    app.add_system(Stage::Physics, player_locomotion_system);
}

pub fn register_authoritative_block_system(app: &mut App) {
    app.add_system(Stage::Update, held_tool_select_system);
    app.add_system(Stage::Update, block_mining_system);
    app.add_system(Stage::Update, block_interaction_system);
}

pub fn register_server_systems(app: &mut App) {
    register_world_systems(app);
    register_player_spawn_systems(app);
    register_player_look_system(app);
    register_player_systems(app);
    register_authoritative_block_system(app);
}

/// Local client: terrain + survival player sim (spectator camera gated by play mode).
pub fn register_local_client_systems(app: &mut App) {
    register_world_systems(app);
    register_player_spawn_systems(app);
    register_player_systems(app);
    register_authoritative_block_system(app);
}

pub fn register_network_client_systems(app: &mut App) {
    register_world_systems(app);
    register_player_spawn_systems(app);
    register_player_systems(app);
}
