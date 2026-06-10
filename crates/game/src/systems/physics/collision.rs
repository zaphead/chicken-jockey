use engine_assets::BlockRegistry;
use engine_world::{BlockPos, SparseVoxelOctree};
use glam::Vec3;

pub fn collides_aabb(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    position: Vec3,
    half_extents: Vec3,
) -> bool {
    let min = (position - half_extents).floor().as_ivec3();
    let max = (position + half_extents).ceil().as_ivec3();

    for x in min.x..=max.x {
        for y in min.y..=max.y {
            for z in min.z..=max.z {
                let block = world.get_block(BlockPos::new(x, y, z));
                if registry.is_solid(block) {
                    return true;
                }
            }
        }
    }
    false
}
