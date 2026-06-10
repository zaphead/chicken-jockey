use std::sync::Arc;

use engine_assets::{
    blocks_asset_path, load_block_registry, pack_block_materials, textures_asset_path, AssetServer,
};
use engine_core::App;
use engine_input::InputState;
use engine_render::{RenderExtractState, RenderSurfaceInfo, RenderWorld};
use engine_world::{BiomeMap, SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_local_client_systems, ActiveDebugWorld, ActivePlayMode, DebugWorldKind,
    DisplayedPlayerView, LocalPlayerId, PlayerInputs, TerrainGeneration, WorldInitialized,
    WorldSeed,
};

use crate::systems::input::PendingWinitInput;
use crate::systems::interpolation::PreviousPlayerTransform;
use crate::systems::register_client_schedule;
use crate::systems::spectator::reset_spectator_for_world;

/// Shared ECS resources for client, tests, and diagnostics (no block registry or textures).
pub fn bootstrap_client_shell(app: &mut App) {
    app.insert_resource(PlayerInputs::default());
    app.insert_resource(PreviousPlayerTransform::default());
    app.insert_resource(PendingWinitInput(InputState::default()));
    app.insert_resource(SparseVoxelOctree::default());
    app.insert_resource(BiomeMap::default());
    app.insert_resource(WorldMutationQueue::default());
    app.insert_resource(WorldInitialized::default());
    app.insert_resource(TerrainGeneration::default());
    let seed = WorldSeed::random();
    let seed_value = seed.0;
    app.insert_resource(seed);
    app.insert_resource(LocalPlayerId::default());
    app.insert_resource(RenderExtractState::default());
    app.insert_resource(RenderWorld::default());
    app.insert_resource(RenderSurfaceInfo::default());
    app.insert_resource(reset_spectator_for_world(DebugWorldKind::Flat, seed_value));
    app.insert_resource(ActivePlayMode::default());
    app.insert_resource(DisplayedPlayerView::default());
    app.insert_resource(ActiveDebugWorld::default());
}

/// Loads block registry and packs block textures into a single resource.
pub fn bootstrap_client_resources(app: &mut App, manifest_dir: &str) {
    let blocks_path = blocks_asset_path(manifest_dir);
    let textures_path = textures_asset_path(manifest_dir);
    let registry = load_block_registry(&blocks_path);
    let packed = pack_block_materials(&textures_path, &registry).expect("pack block materials");
    let mut assets = AssetServer::default();
    assets.insert_blocks(registry.clone());
    app.insert_resource(assets);
    app.insert_resource(registry);
    app.insert_resource(Arc::new(packed));
}

/// Shared client ECS bootstrap for the game binary, diagnostics, and tests.
pub fn bootstrap_local_app(time: engine_core::Time) -> engine_core::App {
    let mut app = App::new();
    app.insert_resource(time);
    bootstrap_client_shell(&mut app);
    bootstrap_client_resources(&mut app, env!("CARGO_MANIFEST_DIR"));
    register_local_client_systems(&mut app);
    register_client_schedule(&mut app);
    app
}
