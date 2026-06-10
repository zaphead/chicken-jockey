pub mod extract;
pub mod input;
pub mod net;
pub mod present;

use engine_assets::poll_assets_system;

pub use extract::{
    enqueue_world_mesh_batch_system, extract_render_world_system, queue_initial_world_meshes_system,
    sync_block_changes_system,
};
pub use input::sync_local_input_system;
pub use net::client_net_system;
pub use present::{present_frame_system, ClientRenderer};

pub fn register_client_schedule(app: &mut engine_core::App) {
    use engine_core::Stage;

    app.add_system(Stage::PreUpdate, poll_assets_system);
    app.add_system(Stage::PreUpdate, sync_local_input_system);
    app.add_system(Stage::PreUpdate, client_net_system);
    app.add_system(Stage::Extract, sync_block_changes_system);
    app.add_system(Stage::Extract, queue_initial_world_meshes_system);
    app.add_system(Stage::Extract, enqueue_world_mesh_batch_system);
    app.add_system(Stage::Extract, extract_render_world_system);
    app.add_system(Stage::Render, present_frame_system);
}
