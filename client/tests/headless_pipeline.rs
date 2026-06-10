use client::bootstrap::bootstrap_local_app;
use engine_core::Time;
use engine_render::RenderWorld;
use engine_world::{BlockPos, SparseVoxelOctree};
use game::{TerrainGeneration, WorldInitialized};

#[test]
fn headless_pipeline_builds_terrain_and_meshes() {
    let mut app = bootstrap_local_app(Time::new(1.0 / 60.0));

    for _ in 0..300 {
        app.tick_with_render();
        app.end_frame();
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

    let origin = BlockPos::new(0, 10, 0);
    let block = world.get_block(origin);
    let solid_origin = registry.is_solid(block);
    let render_world = app.resource::<RenderWorld>().expect("render world");
    let meshes = &render_world.meshes;
    let vertex_count: usize = meshes.iter().map(|m| m.vertices.len()).sum();

    eprintln!("=== headless pipeline diagnostic ===");
    eprintln!("terrain_complete: {terrain_done}");
    eprintln!("world_initialized: {initialized}");
    eprintln!("origin solid: {solid_origin}");
    eprintln!("mesh_count: {}", meshes.len());
    eprintln!("vertex_count: {vertex_count}");

    assert!(terrain_done);
    assert!(initialized);
    assert!(solid_origin);
    assert!(!meshes.is_empty());
    assert!(vertex_count > 0);
}
