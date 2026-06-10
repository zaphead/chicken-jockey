use std::collections::HashMap;

use engine_assets::{BlockMaterialMap, BlockRegistry};
use engine_world::{BlockPos, CHUNK_SIZE, SparseVoxelOctree};
use glam::{IVec3, Vec3};
use rayon::prelude::*;

use crate::mesh::{append_face, SolidMesh};

#[derive(Debug, Default, Clone)]
pub struct RenderScene {
    pub camera: crate::camera::Camera,
    pub chunk_meshes: Vec<SolidMesh>,
    pub entity_meshes: Vec<(glam::Vec3, SolidMesh)>,
}

/// Max chunk meshes rebuilt per frame to keep the main thread responsive.
pub const MAX_CHUNK_REBUILDS_PER_FRAME: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct RebuildBudget {
    pub max_chunks: usize,
    pub max_distance_sq: f32,
}

impl RebuildBudget {
    pub fn all() -> Self {
        Self {
            max_chunks: usize::MAX,
            max_distance_sq: f32::MAX,
        }
    }

    pub fn near(max_distance: f32) -> Self {
        Self {
            max_chunks: MAX_CHUNK_REBUILDS_PER_FRAME,
            max_distance_sq: max_distance * max_distance,
        }
    }
}

#[derive(Debug, Default)]
pub struct ChunkMeshCache {
    meshes: HashMap<IVec3, SolidMesh>,
    dirty: HashMap<IVec3, ()>,
}

impl ChunkMeshCache {
    pub fn mark_dirty(&mut self, chunk: IVec3) {
        self.dirty.insert(chunk, ());
    }

    pub fn mark_dirty_neighbors(&mut self, position: BlockPos) {
        let chunk = position.chunk_key();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    self.mark_dirty(chunk + IVec3::new(dx, dy, dz));
                }
            }
        }
    }

    pub fn has_dirty_chunks(&self) -> bool {
        !self.dirty.is_empty()
    }

    pub fn rebuild(
        &mut self,
        world: &SparseVoxelOctree,
        registry: &BlockRegistry,
        materials: &BlockMaterialMap,
        camera_position: Vec3,
        budget: RebuildBudget,
        top_faces_only: bool,
    ) -> usize {
        let mut dirty: Vec<IVec3> = self
            .dirty
            .keys()
            .copied()
            .filter(|chunk| {
                let center = chunk_center(*chunk);
                center.distance_squared(camera_position) <= budget.max_distance_sq
            })
            .collect();

        if budget.max_chunks != usize::MAX {
            dirty.sort_by(|a, b| {
                chunk_center(*a)
                    .distance_squared(camera_position)
                    .partial_cmp(&chunk_center(*b).distance_squared(camera_position))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            dirty.truncate(budget.max_chunks);
        }

        for chunk in &dirty {
            self.dirty.remove(chunk);
        }

        let rebuilt: Vec<(IVec3, SolidMesh)> = dirty
            .par_iter()
            .map(|chunk| {
                (
                    *chunk,
                    mesh_chunk(world, registry, materials, *chunk, top_faces_only),
                )
            })
            .collect();

        let count = rebuilt.len();
        for (chunk, mesh) in rebuilt {
            if mesh.vertices.is_empty() {
                self.meshes.remove(&chunk);
            } else {
                self.meshes.insert(chunk, mesh);
            }
        }

        if budget.max_distance_sq < f32::MAX {
            self.meshes.retain(|chunk, _| {
                chunk_center(*chunk).distance_squared(camera_position) <= budget.max_distance_sq
            });
        }

        count
    }

    pub fn all_meshes(&self) -> Vec<SolidMesh> {
        self.meshes.values().cloned().collect()
    }
}

fn chunk_center(chunk: IVec3) -> Vec3 {
    (chunk * CHUNK_SIZE).as_vec3() + Vec3::splat(CHUNK_SIZE as f32 * 0.5)
}

pub fn mesh_chunk(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    materials: &BlockMaterialMap,
    chunk: IVec3,
    top_faces_only: bool,
) -> SolidMesh {
    let origin = chunk * CHUNK_SIZE;
    let mut mesh = SolidMesh::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let pos = BlockPos::new(origin.x + x, origin.y + y, origin.z + z);
                let block = world.get_block(pos);
                if !registry.is_solid(block) {
                    continue;
                }

                let px = pos.0;

                for (normal, offset) in FACE_OFFSETS {
                    if top_faces_only && normal != [0.0, 0.0, 1.0] {
                        continue;
                    }
                    let neighbor =
                        BlockPos::new(px.x + offset.x, px.y + offset.y, px.z + offset.z);
                    if registry.is_solid(world.get_block(neighbor)) {
                        continue;
                    }
                    let uv = materials.face_uv(block, normal);
                    append_face(&mut mesh, px.as_vec3(), normal, uv);
                }
            }
        }
    }

    mesh
}

pub fn extract_render_scene(
    camera: crate::camera::Camera,
    chunk_meshes: Vec<SolidMesh>,
    entity_meshes: Vec<(glam::Vec3, SolidMesh)>,
) -> RenderScene {
    RenderScene {
        camera,
        chunk_meshes,
        entity_meshes,
    }
}

const FACE_OFFSETS: [([f32; 3], IVec3); 6] = [
    ([1.0, 0.0, 0.0], IVec3::new(1, 0, 0)),
    ([-1.0, 0.0, 0.0], IVec3::new(-1, 0, 0)),
    ([0.0, 1.0, 0.0], IVec3::new(0, 1, 0)),
    ([0.0, -1.0, 0.0], IVec3::new(0, -1, 0)),
    ([0.0, 0.0, 1.0], IVec3::new(0, 0, 1)),
    ([0.0, 0.0, -1.0], IVec3::new(0, 0, -1)),
];

#[cfg(test)]
mod tests {
    use super::*;
    use engine_assets::{
        blocks_asset_path, load_block_registry, pack_block_textures, textures_asset_path,
    };
    use glam::IVec3;

    fn grass_fixtures() -> (SparseVoxelOctree, BlockRegistry, engine_assets::BlockMaterialMap) {
        let client = concat!(env!("CARGO_MANIFEST_DIR"), "/../../client");
        let registry = load_block_registry(&blocks_asset_path(client));
        let packed = pack_block_textures(&textures_asset_path(client), &registry).expect("pack");
        let grass = registry.id_by_name("grass").expect("grass block");
        let mut world = SparseVoxelOctree::default();
        for x in -64..64 {
            for y in -64..64 {
                world.set_block(BlockPos::new(x, y, 0), grass);
            }
        }
        (world, registry, packed.materials)
    }

    #[test]
    fn mesh_chunk_covers_negative_quadrant() {
        let (world, registry, materials) = grass_fixtures();
        let neg = mesh_chunk(&world, &registry, &materials, IVec3::new(-1, -1, 0), true);
        let pos = mesh_chunk(&world, &registry, &materials, IVec3::new(0, 0, 0), true);
        assert!(!neg.vertices.is_empty(), "negative chunk should mesh");
        assert_eq!(neg.vertices.len(), pos.vertices.len());
    }

    #[test]
    fn top_faces_wind_counterclockwise_from_above() {
        let (world, registry, materials) = grass_fixtures();
        let mesh = mesh_chunk(&world, &registry, &materials, IVec3::ZERO, true);
        for tri in mesh.indices.chunks_exact(3) {
            let a = glam::Vec3::from(mesh.vertices[tri[0] as usize].position);
            let b = glam::Vec3::from(mesh.vertices[tri[1] as usize].position);
            let c = glam::Vec3::from(mesh.vertices[tri[2] as usize].position);
            let winding = (b - a).cross(c - a).z;
            assert!(
                winding > 0.0,
                "top face triangle should be CCW from +Z, got winding {winding}"
            );
        }
    }
}
