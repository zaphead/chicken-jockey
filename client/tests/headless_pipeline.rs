use client::bootstrap::bootstrap_local_app;
use client::frame::run_client_frame;
use engine_core::{Time, SIM_DT};
use engine_render::RenderWorld;
use engine_world::{BlockPos, SparseVoxelOctree};
use game::{
    terrain_surface_z, ActiveDebugWorld, DebugWorldKind, TerrainGeneration, WorldInitialized,
    WorldSeed,
};

#[test]
fn headless_pipeline_builds_terrain_and_meshes() {
    let mut app = bootstrap_local_app(Time::new(SIM_DT));

    for _ in 0..300 {
        run_client_frame(&mut app, SIM_DT);
    }

    let registry = app.resource::<engine_assets::BlockRegistry>().expect("registry");
    let world = app.resource::<SparseVoxelOctree>().expect("svo");
    let initialized = app
        .resource::<WorldInitialized>()
        .map(|w| w.0)
        .unwrap_or(false);
    let terrain_done = app
        .resource::<TerrainGeneration>()
        .map(|t| t.complete)
        .unwrap_or(false);

    let world_kind = app
        .resource::<ActiveDebugWorld>()
        .map(|active| active.0)
        .unwrap_or(DebugWorldKind::Flat);
    let seed = app.resource::<WorldSeed>().map(|seed| seed.0).unwrap_or(0);
    let origin = BlockPos::new(0, 0, terrain_surface_z(0, 0, world_kind, seed));
    let block = world.get_block(origin);
    let solid_origin = registry.is_solid(block);
    let render_world = app.resource::<RenderWorld>().expect("render world");
    let meshes = render_world.meshes();
    let vertex_count: usize = meshes.iter().map(|m| m.vertices.len()).sum();

    assert!(terrain_done);
    assert!(initialized);
    assert!(solid_origin);
    assert!(!meshes.is_empty(), "expected merged terrain meshes");
    assert!(vertex_count > 0);
}
