pub mod net;

pub use net::server_net_system;
pub use net::ServerNet;

use engine_assets::poll_assets_system;

pub fn register_server_schedule(app: &mut engine_core::App) {
    use engine_core::Stage;

    app.add_system(Stage::PreUpdate, poll_assets_system);
    app.add_system(Stage::PreUpdate, server_net_system);
    app.add_system(Stage::PostUpdate, server_net_broadcast_system);
}

use net::server_net_broadcast_system;
