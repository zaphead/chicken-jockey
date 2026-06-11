use engine_world::BlockPos;
use glam::Vec3;

pub fn block_center(position: BlockPos) -> Vec3 {
    position.0.as_vec3() + Vec3::splat(0.5)
}
