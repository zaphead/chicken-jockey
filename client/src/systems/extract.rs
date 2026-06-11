use std::sync::Arc;

use engine_assets::{BlockRegistry, ResolvedBlockMaterials};
use engine_core::{SystemContext, Time};
use engine_render::{
    build_mining_overlay_mesh, Camera, MiningOverlay, ParticleSystem, RenderExtractState,
    RenderSurfaceInfo, RenderWorld,
};
use engine_world::{BiomeMap, SparseVoxelOctree, VoxelCell, VoxelChanged};
use game::{
    build_lighting_snapshot, destroy_stage, local_player_entity, raycast_voxel, ActiveDebugWorld,
    ActivePlayMode, BlockMiningState, DayNightCycle, DebugWorldKind, DisplayedPlayerView, PlayMode,
    Transform, WorldInitialized, BLOCK_REACH, PLAYER_EYE_OFFSET_Z,
};

use crate::lighting::render_lighting;
use crate::mesh_pipeline::{bootstrap_terrain_meshes, rebuild_budget_for_extract, rebuild_chunk_meshes};
use crate::systems::interpolation::{
    lerp_transform_snapshot, PreviousPlayerTransform, TransformSnapshot,
};
use crate::systems::spectator::SpectatorCamera;

pub fn sync_block_changes_system(ctx: &mut SystemContext<'_>) {
    let changes: Vec<VoxelChanged> = ctx.events.drain::<VoxelChanged>();
    if changes.is_empty() {
        return;
    }

    let registry = ctx.resources.get::<BlockRegistry>().cloned();
    let materials = ctx.resources.get::<Arc<ResolvedBlockMaterials>>().cloned();
    let biome = ctx
        .resources
        .get::<BiomeMap>()
        .cloned()
        .unwrap_or_default();

    if let (Some(registry), Some(materials), Some(particles)) = (
        registry,
        materials,
        ctx.resources.get_mut::<ParticleSystem>(),
    ) {
        for change in &changes {
            if change.old_cell.id != 0 && change.new_cell == VoxelCell::AIR {
                particles.spawn_block_break(
                    change.position,
                    change.old_cell,
                    &registry,
                    &materials,
                    &biome,
                );
            }
        }
    }

    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
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
    let world_kind = ctx
        .resources
        .get::<ActiveDebugWorld>()
        .map(|active| active.0)
        .unwrap_or(DebugWorldKind::Flat);
    let Some(state) = ctx.resources.get_mut::<RenderExtractState>() else {
        return;
    };
    bootstrap_terrain_meshes(state, world_kind);
}

pub fn extract_render_world_system(ctx: &mut SystemContext<'_>) {
    let aspect = ctx
        .resources
        .get::<RenderSurfaceInfo>()
        .map(|info| info.aspect)
        .unwrap_or(16.0 / 9.0);
    let camera = extract_camera(ctx, aspect);
    if let Some(view) = ctx.resources.get_mut::<DisplayedPlayerView>() {
        *view = DisplayedPlayerView {
            eye: camera.position,
            yaw: camera.yaw,
            pitch: camera.pitch,
            valid: true,
        };
    }
    let animation_tick = ctx
        .resources
        .get::<Time>()
        .map(|time| time.sim_tick as u32)
        .unwrap_or(0);

    let Some(materials) = ctx.resources.get::<Arc<ResolvedBlockMaterials>>().cloned() else {
        return;
    };

    let biome = ctx
        .resources
        .get::<BiomeMap>()
        .cloned()
        .unwrap_or_default();
    let mesh_generation = ctx
        .resources
        .get::<RenderWorld>()
        .map(|world| world.mesh_generation)
        .unwrap_or(0);

    let extract = ctx
        .resources
        .with_triple::<SparseVoxelOctree, BlockRegistry, RenderExtractState, _>(
            |world, registry, state| {
                if state.mesh_cache.has_dirty_chunks() {
                    let budget = rebuild_budget_for_extract(state);
                    rebuild_chunk_meshes(
                        state,
                        world,
                        registry,
                        &materials,
                        &biome,
                        camera.position,
                        budget,
                    );
                }
                let generation = state.mesh_cache.generation();
                let buckets = state.mesh_cache.merged_buckets();
                let mesh_update = if generation != mesh_generation {
                    Some((buckets.opaque.clone(), buckets.cutout.clone()))
                } else {
                    None
                };
                let target_block = raycast_voxel(
                    world,
                    registry,
                    camera.position,
                    camera.forward(),
                    BLOCK_REACH,
                )
                .map(|hit| hit.block_pos);
                (generation, mesh_update, target_block)
            },
        );

    let Some((generation, mesh_update, target_block)) = extract else {
        return;
    };

    let mining_overlay = match (
        ctx.resources.get::<engine_world::SparseVoxelOctree>(),
        ctx.resources.get::<engine_assets::BlockRegistry>(),
        local_player_entity(ctx),
    ) {
        (Some(world), Some(registry), Some(entity)) => ctx
            .world
            .get::<&BlockMiningState>(entity)
            .ok()
            .and_then(|mining| {
                mining.target.map(|block_pos| {
                    let cell = world.get_voxel(block_pos);
                    MiningOverlay {
                        mesh: build_mining_overlay_mesh(
                            block_pos,
                            destroy_stage(mining.progress),
                            cell,
                            world,
                            registry,
                            &materials,
                            &biome,
                        ),
                    }
                })
            }),
        _ => None,
    };

    let lighting = ctx
        .resources
        .get::<DayNightCycle>()
        .map(|cycle| render_lighting(build_lighting_snapshot(cycle.world_time)))
        .unwrap_or_default();

    if let Some(render_world) = ctx.resources.get_mut::<RenderWorld>() {
        render_world.camera = camera;
        if let Some((opaque, cutout)) = mesh_update {
            render_world.opaque = opaque;
            render_world.cutout = cutout;
            render_world.mesh_generation = generation;
        }
        render_world.animation_tick = animation_tick;
        render_world.lighting = lighting;
        render_world.target_block = target_block;
        render_world.mining_overlay = mining_overlay;
        render_world.ready = true;
    }
}

fn extract_camera(ctx: &SystemContext<'_>, aspect: f32) -> Camera {
    let survival = ctx
        .resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.0 == PlayMode::Survival);

    if survival {
        if let Some(entity) = local_player_entity(ctx) {
            if let Ok(transform) = ctx.world.get::<&Transform>(entity) {
                let alpha = ctx
                    .resources
                    .get::<Time>()
                    .map(|time| time.interpolation_alpha)
                    .unwrap_or(0.0);
                let previous = ctx
                    .resources
                    .get::<PreviousPlayerTransform>()
                    .and_then(|prev| prev.0)
                    .unwrap_or_else(|| TransformSnapshot::from(&*transform));
                let rendered = lerp_transform_snapshot(previous, &transform, alpha);
                return Camera {
                    position: rendered.position
                        + glam::Vec3::new(0.0, 0.0, PLAYER_EYE_OFFSET_Z),
                    yaw: rendered.yaw,
                    pitch: rendered.pitch,
                    aspect,
                    ..Camera::default()
                };
            }
        }
    }

    let spectator = ctx
        .resources
        .get::<SpectatorCamera>()
        .expect("SpectatorCamera must be registered");
    Camera {
        position: spectator.position,
        yaw: spectator.yaw,
        pitch: spectator.pitch,
        aspect,
        ..Camera::default()
    }
}
