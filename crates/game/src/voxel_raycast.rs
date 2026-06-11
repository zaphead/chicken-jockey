use engine_assets::BlockRegistry;
use engine_core::SystemContext;
use engine_world::{BlockPos, SparseVoxelOctree};
use glam::Vec3;

use crate::axes::{player_view_position, view_forward};
use crate::components::{DisplayedPlayerView, Transform};
use crate::input::LocalPlayerId;

pub const BLOCK_REACH: f32 = 6.0;

#[derive(Debug, Clone, Copy)]
pub struct VoxelRayHit {
    pub block_pos: BlockPos,
    pub normal: glam::IVec3,
}

pub fn camera_interaction_ray(camera_position: Vec3, yaw: f32, pitch: f32) -> (Vec3, Vec3) {
    (
        camera_position,
        view_forward(yaw, pitch),
    )
}

pub fn player_interaction_ray(transform: &Transform) -> (Vec3, Vec3) {
    let origin = player_view_position(transform.position, transform.yaw);
    (origin, view_forward(transform.yaw, transform.pitch))
}

pub fn authoritative_interaction_ray(
    ctx: &SystemContext<'_>,
    net_id: Option<u32>,
    transform: &Transform,
) -> (Vec3, Vec3) {
    let local_id = ctx.resources.get::<LocalPlayerId>().and_then(|local| local.id);
    if net_id == local_id {
        if let Some(view) = ctx.resources.get::<DisplayedPlayerView>() {
            if view.valid {
                return (view.eye, view_forward(view.yaw, view.pitch));
            }
        }
    }
    player_interaction_ray(transform)
}

pub fn raycast_voxel(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<VoxelRayHit> {
    let direction = direction.normalize_or_zero();
    if direction.length_squared() == 0.0 {
        return None;
    }

    // Nudge origin off voxel boundaries so floor() picks the containing cell correctly.
    const ORIGIN_EPSILON: f32 = 1e-4;
    let origin = origin + direction * ORIGIN_EPSILON;

    let mut t = 0.0;
    let mut current = origin.floor().as_ivec3();
    let step = direction.signum().as_ivec3();
    let mut t_max = Vec3::ZERO;
    let mut t_delta = Vec3::splat(f32::INFINITY);

    for axis in 0..3 {
        if direction[axis] != 0.0 {
            let boundary = if step[axis] > 0 {
                current[axis] as f32 + 1.0
            } else {
                current[axis] as f32
            };
            t_max[axis] = (boundary - origin[axis]) / direction[axis];
            t_delta[axis] = (step[axis] as f32 / direction[axis]).abs();
        }
    }

    let mut last_step = glam::IVec3::ZERO;

    while t <= max_distance {
        let pos = BlockPos::new(current.x, current.y, current.z);
        if registry.is_solid(world.get_block(pos)) {
            return Some(VoxelRayHit {
                block_pos: pos,
                normal: -last_step,
            });
        }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            t = t_max.x;
            t_max.x += t_delta.x;
            last_step = glam::IVec3::X * step.x;
            current.x += step.x;
        } else if t_max.y < t_max.z {
            t = t_max.y;
            t_max.y += t_delta.y;
            last_step = glam::IVec3::Y * step.y;
            current.y += step.y;
        } else {
            t = t_max.z;
            t_max.z += t_delta.z;
            last_step = glam::IVec3::Z * step.z;
            current.z += step.z;
        }
    }

    None
}

pub fn block_overlaps_player(
    player_position: Vec3,
    half_extents: Vec3,
    block_pos: BlockPos,
) -> bool {
    let block_min = block_pos.0.as_vec3();
    let block_max = block_min + Vec3::ONE;
    let player_min = player_position - half_extents;
    let player_max = player_position + half_extents;
    player_min.x < block_max.x
        && player_max.x > block_min.x
        && player_min.y < block_max.y
        && player_max.y > block_min.y
        && player_min.z < block_max.z
        && player_max.z > block_min.z
}
