mod systems;

use std::thread;
use std::time::{Duration, Instant};

use engine_assets::{blocks_asset_path, load_block_registry, AssetServer};
use engine_core::{App, Time, SIM_DT, SIM_HZ};
use engine_net::NetServer;
use engine_world::{SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_server_systems, AuthoritativeServer, PlayerInputs, TerrainGeneration,
    WorldInitialized, WorldSeed,
};
use systems::{register_server_schedule, ServerNet};

fn main() {
    env_logger::init();

    let mut app = App::new();
    app.insert_resource(Time::new(SIM_DT));
    app.insert_resource(AuthoritativeServer);
    app.insert_resource(PlayerInputs::default());
    app.insert_resource(SparseVoxelOctree::default());
    app.insert_resource(WorldMutationQueue::default());
    app.insert_resource(WorldInitialized::default());
    app.insert_resource(TerrainGeneration::default());
    app.insert_resource(WorldSeed::random());

    let blocks_path = blocks_asset_path(env!("CARGO_MANIFEST_DIR"));
    let registry = load_block_registry(&blocks_path);
    let mut assets = AssetServer::default();
    assets.insert_blocks(registry.clone());
    app.insert_resource(assets);
    app.insert_resource(registry);

    register_server_systems(&mut app);
    register_server_schedule(&mut app);

    let addr = NetServer::default_addr();
    let net = NetServer::bind(addr);
    app.insert_resource(ServerNet(net));
    log::info!("Chicken Jockey server listening on {addr}");

    let tick_duration = Duration::from_secs_f32(SIM_DT);
    log::info!("Chicken Jockey server sim rate: {SIM_HZ} Hz");
    loop {
        let frame_start = Instant::now();

        if let Some(time) = app.resource_mut::<Time>() {
            time.advance_fixed();
        }
        app.tick();
        app.end_frame();

        let elapsed = frame_start.elapsed();
        if elapsed < tick_duration {
            thread::sleep(tick_duration - elapsed);
        }
    }
}
