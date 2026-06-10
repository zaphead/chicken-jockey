use std::collections::HashMap;

use engine_assets::BlockRegistry;
use engine_world::{BlockPos, CHUNK_SIZE, SparseVoxelOctree};
use glam::{IVec3, Vec3};
use rayon::prelude::*;

use crate::mesh::{MeshVertex, SolidMesh};

#[derive(Debug, Default, Clone)]
pub struct RenderScene {
    pub camera: crate::camera::Camera,
    pub chunk_meshes: Vec<SolidMesh>,
    pub entity_meshes: Vec<(glam::Vec3, SolidMesh)>,
}

/// Max chunk meshes rebuilt per frame to keep the main thread responsive.
pub const MAX_CHUNK_REBUILDS_PER_FRAME: usize = 8;

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

    pub fn rebuild_dirty(
        &mut self,
        world: &SparseVoxelOctree,
        registry: &BlockRegistry,
    ) -> usize {
        self.rebuild_dirty_near(world, registry, Vec3::ZERO, f32::MAX)
    }

    pub fn has_dirty_chunks(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Rebuilds up to [`MAX_CHUNK_REBUILDS_PER_FRAME`] nearest dirty chunks. Returns how many were rebuilt.
    pub fn rebuild_dirty_near(
        &mut self,
        world: &SparseVoxelOctree,
        registry: &BlockRegistry,
        camera_position: Vec3,
        max_distance: f32,
    ) -> usize {
        let max_distance_sq = max_distance * max_distance;
        let mut dirty: Vec<IVec3> = self
            .dirty
            .keys()
            .copied()
            .filter(|chunk| {
                let center = (*chunk * CHUNK_SIZE).as_vec3() + Vec3::splat(CHUNK_SIZE as f32 * 0.5);
                center.distance_squared(camera_position) <= max_distance_sq
            })
            .collect();
        dirty.sort_by(|a, b| {
            let center = |chunk: IVec3| {
                (chunk * CHUNK_SIZE).as_vec3() + Vec3::splat(CHUNK_SIZE as f32 * 0.5)
            };
            center(*a)
                .distance_squared(camera_position)
                .partial_cmp(&center(*b).distance_squared(camera_position))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        dirty.truncate(MAX_CHUNK_REBUILDS_PER_FRAME);
        for chunk in &dirty {
            self.dirty.remove(chunk);
        }

        let rebuilt: Vec<(IVec3, SolidMesh)> = dirty
            .par_iter()
            .map(|chunk| (*chunk, mesh_chunk(world, registry, *chunk)))
            .collect();

        let count = rebuilt.len();
        for (chunk, mesh) in rebuilt {
            if mesh.vertices.is_empty() {
                self.meshes.remove(&chunk);
            } else {
                self.meshes.insert(chunk, mesh);
            }
        }
        count
    }

    pub fn all_meshes(&self) -> Vec<SolidMesh> {
        self.meshes.values().cloned().collect()
    }
}

pub fn mesh_chunk(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    chunk: IVec3,
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

                let color = registry.color(block);
                let px = pos.0;

                for (normal, offset) in FACE_OFFSETS {
                    let neighbor = BlockPos::new(px.x + offset.x, px.y + offset.y, px.z + offset.z);
                    if registry.is_solid(world.get_block(neighbor)) {
                        continue;
                    }
                    push_face(&mut mesh, px.as_vec3(), normal, color);
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

const FACE_OFFSETS: [( [f32; 3], IVec3); 6] = [
    ([1.0, 0.0, 0.0], IVec3::new(1, 0, 0)),
    ([-1.0, 0.0, 0.0], IVec3::new(-1, 0, 0)),
    ([0.0, 1.0, 0.0], IVec3::new(0, 1, 0)),
    ([0.0, -1.0, 0.0], IVec3::new(0, -1, 0)),
    ([0.0, 0.0, 1.0], IVec3::new(0, 0, 1)),
    ([0.0, 0.0, -1.0], IVec3::new(0, 0, -1)),
];

fn push_face(mesh: &mut SolidMesh, origin: glam::Vec3, normal: [f32; 3], color: [f32; 3]) {
    let base = mesh.vertices.len() as u32;
    let [nx, ny, nz] = normal;

    let corners = if nx > 0.0 {
        [
            origin + glam::Vec3::new(1.0, 0.0, 0.0),
            origin + glam::Vec3::new(1.0, 1.0, 0.0),
            origin + glam::Vec3::new(1.0, 1.0, 1.0),
            origin + glam::Vec3::new(1.0, 0.0, 1.0),
        ]
    } else if nx < 0.0 {
        [
            origin + glam::Vec3::new(0.0, 0.0, 1.0),
            origin + glam::Vec3::new(0.0, 1.0, 1.0),
            origin + glam::Vec3::new(0.0, 1.0, 0.0),
            origin + glam::Vec3::new(0.0, 0.0, 0.0),
        ]
    } else if ny > 0.0 {
        [
            origin + glam::Vec3::new(0.0, 1.0, 0.0),
            origin + glam::Vec3::new(1.0, 1.0, 0.0),
            origin + glam::Vec3::new(1.0, 1.0, 1.0),
            origin + glam::Vec3::new(0.0, 1.0, 1.0),
        ]
    } else if ny < 0.0 {
        [
            origin + glam::Vec3::new(0.0, 0.0, 1.0),
            origin + glam::Vec3::new(1.0, 0.0, 1.0),
            origin + glam::Vec3::new(1.0, 0.0, 0.0),
            origin + glam::Vec3::new(0.0, 0.0, 0.0),
        ]
    } else if nz > 0.0 {
        [
            origin + glam::Vec3::new(0.0, 0.0, 1.0),
            origin + glam::Vec3::new(0.0, 1.0, 1.0),
            origin + glam::Vec3::new(1.0, 1.0, 1.0),
            origin + glam::Vec3::new(1.0, 0.0, 1.0),
        ]
    } else {
        [
            origin + glam::Vec3::new(1.0, 0.0, 0.0),
            origin + glam::Vec3::new(1.0, 1.0, 0.0),
            origin + glam::Vec3::new(0.0, 1.0, 0.0),
            origin + glam::Vec3::new(0.0, 0.0, 0.0),
        ]
    };

    for corner in corners {
        mesh.vertices.push(MeshVertex {
            position: corner.to_array(),
            normal,
            color,
        });
    }

    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
