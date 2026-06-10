mod net;

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use engine_assets::load_block_registry;
use engine_core::{App, Time};
use engine_net::NetServer;
use engine_world::{SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_server_systems, RemoteInputs, SimulationMode, TerrainGeneration, WorldInitialized,
};
use net::{server_net_post_update, server_net_pre_update};

const TICK_RATE: f32 = 60.0;

fn main() {
    env_logger::init();

    let mut app = App::new();
    app.insert_resource(Time::new(1.0 / TICK_RATE));
    app.insert_resource(SimulationMode::AuthoritativeServer);
    app.insert_resource(RemoteInputs::default());
    app.insert_resource(SparseVoxelOctree::default());
    app.insert_resource(WorldMutationQueue::default());
    app.insert_resource(WorldInitialized::default());
    app.insert_resource(TerrainGeneration::default());

    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("assets")
        .join("blocks");
    app.insert_resource(load_block_registry(&assets));

    register_server_systems(&mut app);

    let addr = NetServer::default_addr();
    let net = NetServer::bind(addr);
    log::info!("Chicken Jockey server listening on {addr}");

    let tick_duration = Duration::from_secs_f32(1.0 / TICK_RATE);
    loop {
        let frame_start = Instant::now();

        server_net_pre_update(&mut app, &net);
        app.tick();
        server_net_post_update(&mut app, &net);
        app.end_frame();

        let elapsed = frame_start.elapsed();
        if elapsed < tick_duration {
            thread::sleep(tick_duration - elapsed);
        }
    }
}
