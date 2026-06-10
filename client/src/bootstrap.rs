use engine_assets::{blocks_asset_path, load_block_registry, AssetServer};
use engine_core::App;
use engine_input::InputState;
use engine_render::{RenderExtractState, RenderSurfaceInfo, RenderWorld};
use engine_world::{SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_local_client_systems, TerrainGeneration, WorldInitialized, LocalPlayerId, PlayerInputs,
};

use crate::systems::input::PendingWinitInput;
use crate::systems::register_client_schedule;

/// Shared client ECS bootstrap for the game binary, diagnostics, and tests.
pub fn bootstrap_local_app(time: engine_core::Time) -> App {
    let mut app = App::new();
    app.insert_resource(time);
    app.insert_resource(PlayerInputs::default());
    app.insert_resource(PendingWinitInput(InputState::default()));
    app.insert_resource(SparseVoxelOctree::default());
    app.insert_resource(WorldMutationQueue::default());
    app.insert_resource(WorldInitialized::default());
    app.insert_resource(TerrainGeneration::default());
    app.insert_resource(LocalPlayerId::default());
    app.insert_resource(RenderExtractState::default());
    app.insert_resource(RenderWorld::default());
    app.insert_resource(RenderSurfaceInfo::default());

    let blocks_path = blocks_asset_path(env!("CARGO_MANIFEST_DIR"));
    let registry = load_block_registry(&blocks_path);
    let mut assets = AssetServer::default();
    assets.insert_blocks(registry.clone());
    app.insert_resource(assets);
    app.insert_resource(registry);

    register_local_client_systems(&mut app);
    register_client_schedule(&mut app);
    app
}
