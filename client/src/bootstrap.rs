use std::sync::Arc;

use engine_assets::{
    blocks_asset_path, load_block_registry, pack_block_textures, textures_asset_path, AssetServer,
};
use engine_core::App;
use engine_input::InputState;
use engine_render::{RenderExtractState, RenderSurfaceInfo, RenderWorld};
use engine_world::{SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_local_client_systems, LocalPlayerId, PlayerInputs, TerrainGeneration, WorldInitialized,
};

use crate::systems::input::PendingWinitInput;
use crate::systems::register_client_schedule;
use crate::systems::spectator::SpectatorCamera;

/// Shared ECS resources for client, tests, and diagnostics (no block registry or textures).
pub fn bootstrap_client_shell(app: &mut App) {
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
    app.insert_resource(SpectatorCamera::default());
}

/// Loads block registry and packs block textures into a single resource.
pub fn bootstrap_client_resources(app: &mut App, manifest_dir: &str) {
    let blocks_path = blocks_asset_path(manifest_dir);
    let textures_path = textures_asset_path(manifest_dir);
    let registry = load_block_registry(&blocks_path);
    let packed = pack_block_textures(&textures_path, &registry).expect("pack block textures");
    let mut assets = AssetServer::default();
    assets.insert_blocks(registry.clone());
    app.insert_resource(assets);
    app.insert_resource(registry);
    app.insert_resource(Arc::new(packed));
}

/// Shared client ECS bootstrap for the game binary, diagnostics, and tests.
pub fn bootstrap_local_app(time: engine_core::Time) -> App {
    let mut app = App::new();
    app.insert_resource(time);
    bootstrap_client_shell(&mut app);
    bootstrap_client_resources(&mut app, env!("CARGO_MANIFEST_DIR"));
    register_local_client_systems(&mut app);
    register_client_schedule(&mut app);
    app
}
