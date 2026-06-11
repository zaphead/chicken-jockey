use std::sync::Arc;

use engine_assets::{
    blocks_asset_path, load_block_registry, load_environment_textures, load_gui_textures,
    load_music_manifest, load_player_skin, load_sound_registry, load_tool_registry,
    music_asset_path, music_manifest_path, pack_block_materials, runtime_asset_root,
    textures_asset_path, tools_asset_path, AssetServer,
};
use engine_audio::AudioEngine;
use engine_core::App;
use engine_input::InputState;
use engine_render::{ParticleSystem, RenderExtractState, RenderSurfaceInfo, RenderWorld};
use engine_world::{BiomeMap, SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_local_client_systems, ActiveDebugWorld, ActivePlayMode, DayNightCycle, DebugWorldKind,
    DisplayedPlayerView, InventoryCommandQueue, LocalPlayerId, PlayerInputs, TerrainGeneration,
    WorldInitialized, WorldItemBook, WorldSeed,
};

use crate::systems::audio::{ClientAudio, SoundBank};
use crate::systems::music::{MusicBank, MusicPlaybackState};
use crate::systems::input::PendingWinitInput;
use crate::systems::menu::{ClientSettings, CursorGrabRequest};
use crate::systems::ui_state::ClientUiState;
use crate::systems::interpolation::PreviousPlayerTransform;
use crate::systems::register_client_schedule;
use crate::systems::spectator::reset_spectator_for_world;
use crate::systems::zoom::CameraZoom;

pub fn asset_root() -> String {
    runtime_asset_root().to_string_lossy().into_owned()
}

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
    app.insert_resource(ParticleSystem::default());
    app.insert_resource(RenderSurfaceInfo::default());
    app.insert_resource(reset_spectator_for_world(DebugWorldKind::Flat, seed_value));
    app.insert_resource(ActivePlayMode::default());
    app.insert_resource(DisplayedPlayerView::default());
    app.insert_resource(ActiveDebugWorld::default());
    app.insert_resource(DayNightCycle::default());
    app.insert_resource(ClientUiState::default());
    app.insert_resource(ClientSettings::default());
    app.insert_resource(CameraZoom::default());
    app.insert_resource(CursorGrabRequest {
        locked: true,
    });
    app.insert_resource(WorldItemBook::default());
    app.insert_resource(InventoryCommandQueue::default());
    app.insert_resource(MusicPlaybackState::default());
}

/// Loads block registry and packs block textures into a single resource.
pub fn bootstrap_client_resources(app: &mut App, manifest_dir: &str) {
    let blocks_path = blocks_asset_path(manifest_dir);
    let textures_path = textures_asset_path(manifest_dir);
    let registry = load_block_registry(&blocks_path);
    let tools = load_tool_registry(&tools_asset_path(manifest_dir));
    let packed = pack_block_materials(&textures_path, &registry).expect("pack block materials");
    let mut assets = AssetServer::default();
    assets.insert_blocks(registry.clone());
    app.insert_resource(assets);
    app.insert_resource(registry);
    app.insert_resource(tools);
    app.insert_resource(Arc::new(packed));
    app.insert_resource(Arc::new(load_environment_textures(manifest_dir)));
    app.insert_resource(Arc::new(load_gui_textures(manifest_dir)));
    app.insert_resource(Arc::new(load_player_skin(manifest_dir)));

    match load_sound_registry(manifest_dir) {
        Ok(sound_registry) => {
            match AudioEngine::new() {
                Ok(engine) => {
                    log::info!("client audio ready");
                    app.insert_resource(ClientAudio {
                        engine,
                        bank: SoundBank::default(),
                    });
                }
                Err(error) => {
                    log::warn!("client audio disabled: {error}");
                }
            }
            app.insert_resource(sound_registry);
        }
        Err(error) => {
            log::warn!("sound registry unavailable; client audio disabled: {error}");
        }
    }

    match load_music_manifest(&music_manifest_path(manifest_dir)) {
        Ok(music_manifest) => {
            match MusicBank::from_manifest(&music_manifest, &music_asset_path(manifest_dir)) {
                Ok(mut bank) => {
                    bank.start_background_preload();
                    log::info!(
                        "ambient music ready ({} tracks, background preload)",
                        music_manifest.tracks.len()
                    );
                    app.insert_resource(bank);
                }
                Err(error) => {
                    log::warn!("ambient music disabled: {error}");
                }
            }
        }
        Err(error) => {
            log::warn!("music manifest unavailable; ambient music disabled: {error}");
        }
    }
}

/// Shared client ECS bootstrap for the game binary, diagnostics, and tests.
pub fn bootstrap_local_app(time: engine_core::Time) -> engine_core::App {
    let mut app = App::new();
    app.insert_resource(time);
    bootstrap_client_shell(&mut app);
    bootstrap_client_resources(&mut app, &asset_root());
    register_local_client_systems(&mut app);
    register_client_schedule(&mut app);
    app
}
