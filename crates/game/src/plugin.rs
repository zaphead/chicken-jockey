use engine_core::{App, Stage};

use crate::systems::{
    block_interaction_system, chicken_spawn_system, chicken_wander_system, dismount_system,
    flush_world_mutations_system, generate_terrain_system, mount_system, mounted_movement_system,
    mounted_physics_system, player_look_system, player_movement_system, player_physics_system,
    spawn_player_system,
};

pub fn register_shared_systems(app: &mut App) {
    app.add_system(Stage::Update, generate_terrain_system);
    app.add_system(Stage::Update, chicken_spawn_system);
    app.add_system(Stage::Update, mount_system);
    app.add_system(Stage::Update, dismount_system);
    app.add_system(Stage::Update, mounted_movement_system);
    app.add_system(Stage::Physics, chicken_wander_system);
    app.add_system(Stage::Physics, mounted_physics_system);
    app.add_system(Stage::PostUpdate, flush_world_mutations_system);
}

pub fn register_player_systems(app: &mut App) {
    app.add_system(Stage::Update, spawn_player_system);
    app.add_system(Stage::Update, player_look_system);
    app.add_system(Stage::Update, player_movement_system);
    app.add_system(Stage::Update, block_interaction_system);
    app.add_system(Stage::Physics, player_physics_system);
}

pub fn register_game_systems(app: &mut App) {
    register_shared_systems(app);
    register_player_systems(app);
}

pub fn register_server_systems(app: &mut App) {
    register_shared_systems(app);
    register_player_systems(app);
}

pub fn register_client_systems(app: &mut App) {
    register_game_systems(app);
}
