pub mod extract;
pub mod hud;
pub mod input;
pub mod menu;
pub mod interpolation;
pub mod net;
pub mod particles;
pub mod play_mode;
pub mod present;
pub mod spectator;

use engine_assets::poll_assets_system;
use engine_core::Stage;

pub use extract::{
    extract_render_world_system, queue_initial_world_meshes_system, sync_block_changes_system,
};
pub use interpolation::commit_player_transform_snapshot_system;
pub use input::{apply_local_look_system, sync_local_input_system};
pub use menu::{extract_pause_gui_system, pause_menu_input_system};
pub use net::client_net_system;
pub use particles::particle_extract_system;
pub use present::{present_frame_system, ClientRenderer};
pub use play_mode::toggle_play_mode_system;
pub use spectator::spectator_camera_system;

pub fn register_client_schedule(app: &mut engine_core::App) {
    app.add_system(Stage::PreUpdate, poll_assets_system);
    app.add_system(Stage::PreUpdate, toggle_play_mode_system);
    app.add_system(Stage::PreUpdate, pause_menu_input_system);
    app.add_system(Stage::PreUpdate, sync_local_input_system);
    app.add_system(Stage::PreUpdate, apply_local_look_system);
    app.add_system(Stage::PreUpdate, spectator_camera_system);
    app.add_system(Stage::PreUpdate, client_net_system);
    app.add_system(Stage::PostUpdate, sync_block_changes_system);
    app.add_system(Stage::Extract, queue_initial_world_meshes_system);
    app.add_system(Stage::Extract, extract_render_world_system);
    app.add_system(Stage::Extract, particle_extract_system);
    app.add_system(Stage::Extract, extract_pause_gui_system);
    app.add_system(Stage::Extract, commit_player_transform_snapshot_system);
    app.add_system(Stage::Render, present_frame_system);
}
