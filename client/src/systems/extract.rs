use engine_assets::BlockRegistry;
use engine_core::SystemContext;
use engine_render::{
    cube_mesh, Camera, RenderExtractState, RenderSurfaceInfo, RenderWorld,
};
use engine_world::{BlockChanged, SparseVoxelOctree};
use game::{local_player_entity, Renderable, Transform, WorldInitialized};
use glam::Vec3;

use crate::mesh_pipeline::{
    enqueue_mesh_batch, queue_initial_world_chunks, rebuild_chunk_meshes,
};

pub fn sync_block_changes_system(ctx: &mut SystemContext<'_>) {
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    let changes: Vec<BlockChanged> = ctx.events.drain::<BlockChanged>();
    for change in changes {
        state.mesh_cache.mark_dirty_neighbors(change.position);
    }
}

pub fn queue_initial_world_meshes_system(ctx: &mut SystemContext<'_>) {
    let initialized = ctx
        .resources
        .get::<WorldInitialized>()
        .map(|flag| flag.0)
        .unwrap_or(false);
    if !initialized {
        return;
    }
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    queue_initial_world_chunks(state);
}

pub fn enqueue_world_mesh_batch_system(ctx: &mut SystemContext<'_>) {
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    enqueue_mesh_batch(state);
}

pub fn extract_render_world_system(ctx: &mut SystemContext<'_>) {
    let aspect = ctx
        .resources
        .get::<RenderSurfaceInfo>()
        .map(|info| info.aspect)
        .unwrap_or(16.0 / 9.0);
    let camera = extract_camera(ctx, aspect);

    let entity_meshes: Vec<_> = ctx
        .world
        .query::<(&Transform, &Renderable)>()
        .iter()
        .map(|(_, (transform, renderable))| {
            translate_mesh(
                cube_mesh(glam::IVec3::ZERO, renderable.size, renderable.color),
                transform.position - Vec3::splat(renderable.size * 0.5),
            )
        })
        .collect();

    let mut meshes = ctx
        .resources
        .with_triple::<SparseVoxelOctree, BlockRegistry, RenderExtractState, _>(|world, registry, state| {
            let _ = rebuild_chunk_meshes(state, world, registry, camera.position);
            state.mesh_cache.all_meshes()
        })
        .unwrap_or_default();
    meshes.extend(entity_meshes);

    if let Some(render_world) = ctx.resources.get_mut::<RenderWorld>() {
        render_world.camera = camera;
        render_world.meshes = meshes;
        render_world.ready = true;
    }
}

fn extract_camera(ctx: &SystemContext<'_>, aspect: f32) -> Camera {
    let mut camera = Camera::default();
    camera.aspect = aspect;

    if let Some(entity) = local_player_entity(ctx) {
        if let Ok(transform) = ctx.world.get::<&Transform>(entity) {
            camera.position = transform.position + Vec3::new(0.0, 1.6, 0.0);
            camera.yaw = transform.yaw;
            camera.pitch = transform.pitch;
        }
    }

    camera
}

fn translate_mesh(
    mut mesh: engine_render::SolidMesh,
    offset: Vec3,
) -> engine_render::SolidMesh {
    for vertex in &mut mesh.vertices {
        let position = Vec3::from_array(vertex.position) + offset;
        vertex.position = position.to_array();
    }
    mesh
}
