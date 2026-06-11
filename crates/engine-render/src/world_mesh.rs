use std::collections::HashMap;

use engine_assets::{face_from_normal, BlockRegistry, DrawCategory, ResolvedBlockMaterials};
use engine_world::{BiomeMap, BlockPos, CHUNK_SIZE, SparseVoxelOctree, VoxelCell};
use glam::{IVec3, Vec3};
use rayon::prelude::*;

use crate::ctm::neighbor_mask_for_face;
use crate::extract::MiningOverlay;
use crate::particles::ParticleMesh;
use crate::mesh::{append_face, tint_index_for, MeshBuckets, SolidMesh};

#[derive(Debug, Clone)]
pub struct RenderScene {
    pub camera: crate::camera::Camera,
    pub opaque: SolidMesh,
    pub cutout: SolidMesh,
    pub animation_tick: u32,
    pub entity_meshes: Vec<(glam::Vec3, SolidMesh)>,
    pub target_block: Option<BlockPos>,
    pub mining_overlay: Option<MiningOverlay>,
    pub particles: ParticleMesh,
    pub lighting: crate::lighting::LightingSnapshot,
}

impl Default for RenderScene {
    fn default() -> Self {
        Self {
            camera: crate::camera::Camera::default(),
            opaque: SolidMesh::default(),
            cutout: SolidMesh::default(),
            animation_tick: 0,
            entity_meshes: Vec::new(),
            target_block: None,
            mining_overlay: None,
            particles: ParticleMesh::default(),
            lighting: crate::lighting::LightingSnapshot::default(),
        }
    }
}

pub const MAX_CHUNK_REBUILDS_PER_FRAME: usize = 8;
pub const CTM_DIRTY_RADIUS_BLOCKS: i32 = 2;

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
    meshes: HashMap<IVec3, MeshBuckets>,
    dirty: HashMap<IVec3, ()>,
    generation: u64,
    merged: MeshBuckets,
    merged_generation: u64,
}

impl ChunkMeshCache {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn mark_dirty(&mut self, chunk: IVec3) {
        self.dirty.insert(chunk, ());
    }

    pub fn mark_dirty_neighbors(&mut self, position: BlockPos) {
        let block = position.0;
        let radius = CTM_DIRTY_RADIUS_BLOCKS;
        let min_cx = (block.x - radius).div_euclid(CHUNK_SIZE);
        let max_cx = (block.x + radius).div_euclid(CHUNK_SIZE);
        let min_cy = (block.y - radius).div_euclid(CHUNK_SIZE);
        let max_cy = (block.y + radius).div_euclid(CHUNK_SIZE);
        let min_cz = (block.z - radius).div_euclid(CHUNK_SIZE);
        let max_cz = (block.z + radius).div_euclid(CHUNK_SIZE);
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                for cz in min_cz..=max_cz {
                    self.mark_dirty(IVec3::new(cx, cy, cz));
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
        materials: &ResolvedBlockMaterials,
        biome: &BiomeMap,
        camera_position: Vec3,
        budget: RebuildBudget,
        top_faces_only: bool,
    ) -> usize {
        let mut dirty: Vec<IVec3> = self
            .dirty
            .keys()
            .copied()
            .filter(|chunk| {
                chunk_center(*chunk).distance_squared(camera_position) <= budget.max_distance_sq
            })
            .collect();
        dirty.sort_by_key(|chunk| {
            chunk_center(*chunk)
                .distance_squared(camera_position)
                .to_bits()
        });

        let count = dirty.len().min(budget.max_chunks);
        let rebuilt: Vec<(IVec3, MeshBuckets)> = dirty
            .par_iter()
            .take(count)
            .map(|&chunk| {
                (
                    chunk,
                    mesh_chunk(world, registry, materials, biome, chunk, top_faces_only),
                )
            })
            .collect();

        let mut changed = false;
        for (chunk, mesh) in rebuilt {
            self.dirty.remove(&chunk);
            if mesh.is_empty() {
                changed |= self.meshes.remove(&chunk).is_some();
            } else {
                self.meshes.insert(chunk, mesh);
                changed = true;
            }
        }

        if budget.max_distance_sq < f32::MAX {
            let before = self.meshes.len();
            self.meshes.retain(|chunk, _| {
                chunk_center(*chunk).distance_squared(camera_position) <= budget.max_distance_sq
            });
            changed |= self.meshes.len() != before;
        }

        if changed {
            self.generation += 1;
        }

        count
    }

    pub fn merged_buckets(&mut self) -> &MeshBuckets {
        if self.merged_generation != self.generation {
            self.merged = MeshBuckets::default();
            for buckets in self.meshes.values() {
                self.merged
                    .push(DrawCategory::Opaque, &buckets.opaque);
                self.merged
                    .push(DrawCategory::Cutout, &buckets.cutout);
            }
            self.merged_generation = self.generation;
        }
        &self.merged
    }
}

fn chunk_center(chunk: IVec3) -> Vec3 {
    (chunk * CHUNK_SIZE).as_vec3() + Vec3::splat(CHUNK_SIZE as f32 * 0.5)
}

pub fn mesh_chunk(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    materials: &ResolvedBlockMaterials,
    biome: &BiomeMap,
    chunk: IVec3,
    top_faces_only: bool,
) -> MeshBuckets {
    let origin = chunk * CHUNK_SIZE;
    let mut buckets = MeshBuckets::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let pos = BlockPos::new(origin.x + x, origin.y + y, origin.z + z);
                let cell = world.get_voxel(pos);
                if !registry.is_solid(cell.id) {
                    continue;
                }

                let px = pos.0;

                for (normal, offset) in FACE_OFFSETS {
                    if top_faces_only && normal != [0.0, 0.0, 1.0] {
                        continue;
                    }
                    let neighbor = BlockPos::new(px.x + offset.x, px.y + offset.y, px.z + offset.z);
                    if should_cull_face(world, registry, cell, neighbor) {
                        continue;
                    }

                    let face = face_from_normal(normal);
                    let neighbors = registry
                        .get(cell.id)
                        .filter(|def| def.ctm.is_some())
                        .map(|_| neighbor_mask_for_face(world, registry, pos, cell.id, face));

                    let resolved = materials.resolve_face(cell.id, cell.state, face, neighbors);
                    let tint_index = tint_index_for(resolved.tint, biome, pos);
                    let mut face_mesh = SolidMesh::default();
                    append_face(&mut face_mesh, px.as_vec3(), normal, resolved, tint_index);
                    buckets.push(resolved.draw_category, &face_mesh);
                }
            }
        }
    }

    buckets
}

fn should_cull_face(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    cell: VoxelCell,
    neighbor: BlockPos,
) -> bool {
    let neighbor_cell = world.get_voxel(neighbor);
    if !registry.is_solid(neighbor_cell.id) {
        return false;
    }
    let self_cutout = registry
        .get(cell.id)
        .map(|def| def.draw == DrawCategory::Cutout)
        .unwrap_or(false);
    let neighbor_cutout = registry
        .get(neighbor_cell.id)
        .map(|def| def.draw == DrawCategory::Cutout)
        .unwrap_or(false);
    if self_cutout || neighbor_cutout {
        return false;
    }
    true
}

const FACE_OFFSETS: [([f32; 3], IVec3); 6] = [
    ([1.0, 0.0, 0.0], IVec3::new(1, 0, 0)),
    ([-1.0, 0.0, 0.0], IVec3::new(-1, 0, 0)),
    ([0.0, 1.0, 0.0], IVec3::new(0, 1, 0)),
    ([0.0, -1.0, 0.0], IVec3::new(0, -1, 0)),
    ([0.0, 0.0, 1.0], IVec3::new(0, 0, 1)),
    ([0.0, 0.0, -1.0], IVec3::new(0, 0, -1)),
];

pub fn extract_render_scene(
    camera: crate::camera::Camera,
    opaque: SolidMesh,
    cutout: SolidMesh,
    animation_tick: u32,
    entity_meshes: Vec<(glam::Vec3, SolidMesh)>,
    target_block: Option<BlockPos>,
    mining_overlay: Option<MiningOverlay>,
    particles: ParticleMesh,
    lighting: crate::lighting::LightingSnapshot,
) -> RenderScene {
    RenderScene {
        camera,
        opaque,
        cutout,
        animation_tick,
        entity_meshes,
        target_block,
        mining_overlay,
        particles,
        lighting,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_assets::{
        blocks_asset_path, load_block_registry, pack_block_materials, textures_asset_path,
    };

    fn grass_fixtures() -> (
        SparseVoxelOctree,
        BlockRegistry,
        ResolvedBlockMaterials,
        BiomeMap,
    ) {
        let client = concat!(env!("CARGO_MANIFEST_DIR"), "/../../client");
        let registry = load_block_registry(&blocks_asset_path(client));
        let materials = pack_block_materials(&textures_asset_path(client), &registry).expect("pack");
        (SparseVoxelOctree::default(), registry, materials, BiomeMap::default())
    }

    #[test]
    fn top_faces_wind_counterclockwise_from_above() {
        let (mut world, registry, materials, biome) = grass_fixtures();
        let grass = registry.id_by_name("grass").expect("grass");
        world.set_block(BlockPos::new(0, 0, 0), grass);
        let mesh = mesh_chunk(&world, &registry, &materials, &biome, IVec3::ZERO, true);
        assert!(!mesh.opaque.vertices.is_empty());
        let v = &mesh.opaque.vertices[0];
        let p0 = glam::Vec3::from_array(v.position);
        let p1 = glam::Vec3::from_array(mesh.opaque.vertices[1].position);
        let p2 = glam::Vec3::from_array(mesh.opaque.vertices[2].position);
        let cross = (p1 - p0).cross(p2 - p0);
        assert!(cross.z > 0.0, "top face should wind CCW from +Z");
    }

    #[test]
    fn leaves_mesh_routes_to_cutout_bucket() {
        let (mut world, registry, materials, biome) = grass_fixtures();
        let leaves = registry.id_by_name("leaves").expect("leaves");
        world.set_block(BlockPos::new(2, 2, 0), leaves);
        let mesh = mesh_chunk(&world, &registry, &materials, &biome, IVec3::ZERO, false);
        assert!(mesh.opaque.vertices.is_empty() || mesh.cutout.vertices.len() > 0);
        assert!(!mesh.cutout.vertices.is_empty());
    }

    #[test]
    fn mesh_chunk_covers_negative_quadrant() {
        let (mut world, registry, materials, biome) = grass_fixtures();
        let grass = registry.id_by_name("grass").expect("grass");
        world.set_block(BlockPos::new(-1, -1, 0), grass);
        let chunk = IVec3::new(-1, -1, 0);
        let mesh = mesh_chunk(&world, &registry, &materials, &biome, chunk, true);
        assert!(!mesh.opaque.vertices.is_empty());
    }

    #[test]
    fn top_faces_wind_counterclockwise_in_clip_space() {
        use crate::camera::Camera;
        use crate::mesh::face_corners;
        use glam::Vec4;

        let camera = Camera {
            position: glam::Vec3::new(0.5, 0.5, 5.9),
            yaw: 0.0,
            pitch: -0.35,
            aspect: 16.0 / 9.0,
            ..Camera::default()
        };
        let vp = camera.view_projection();
        let corners = face_corners(glam::Vec3::ZERO, [0.0, 0.0, 1.0]);
        let ndc: Vec<glam::Vec3> = corners
            .iter()
            .map(|p| {
                let clip = vp * Vec4::new(p.x, p.y, p.z, 1.0);
                clip.truncate() / clip.w
            })
            .collect();
        let cross = (ndc[1] - ndc[0]).cross(ndc[2] - ndc[0]);
        assert!(
            cross.z > 0.0,
            "top face should be CCW in clip space for FrontFace::Ccw, got cross.z={}",
            cross.z
        );
    }

    #[test]
    fn dirt_debug_view_winds_visible_faces_ccw() {
        use crate::camera::Camera;
        use crate::mesh::face_corners;
        use glam::Vec4;

        let camera = Camera {
            position: glam::Vec3::new(6.4, -2.8, 2.1),
            yaw: 16.3_f32.to_radians(),
            pitch: -23.4_f32.to_radians(),
            aspect: 16.0 / 9.0,
            ..Camera::default()
        };
        let block_center = glam::Vec3::new(7.5, 0.5, 0.5);
        let to_cam = (camera.position - block_center).normalize();

        let normals: [[f32; 3]; 6] = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];

        let vp = camera.view_projection();
        for normal in normals {
            let outward = Vec3::from_array(normal);
            if outward.dot(to_cam) <= 0.0 {
                continue;
            }
            let corners = face_corners(glam::Vec3::new(7.0, 0.0, 0.0), normal);
            let ndc: Vec<glam::Vec3> = corners
                .iter()
                .map(|p| {
                    let clip = vp * Vec4::new(p.x, p.y, p.z, 1.0);
                    clip.truncate() / clip.w
                })
                .collect();
            let cross = (ndc[1] - ndc[0]).cross(ndc[2] - ndc[0]);
            assert!(
                cross.z > 0.0,
                "visible face {:?} should be CCW in clip space, cross.z={}",
                normal,
                cross.z
            );
        }
    }

    #[test]
    fn dirt_bottom_view_winds_visible_faces_ccw() {
        use crate::camera::Camera;
        use crate::mesh::face_corners;
        use glam::Vec4;

        let camera = Camera {
            position: glam::Vec3::new(8.8, 2.4, -1.2),
            yaw: 203.4_f32.to_radians(),
            pitch: 23.3_f32.to_radians(),
            aspect: 16.0 / 9.0,
            ..Camera::default()
        };
        let block_center = glam::Vec3::new(7.5, 0.5, 0.5);
        let to_cam = (camera.position - block_center).normalize();

        let normals: [[f32; 3]; 6] = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];

        let vp = camera.view_projection();
        for normal in normals {
            let outward = Vec3::from_array(normal);
            if outward.dot(to_cam) <= 0.0 {
                continue;
            }
            let corners = face_corners(glam::Vec3::new(7.0, 0.0, 0.0), normal);
            let ndc: Vec<glam::Vec3> = corners
                .iter()
                .map(|p| {
                    let clip = vp * Vec4::new(p.x, p.y, p.z, 1.0);
                    clip.truncate() / clip.w
                })
                .collect();
            let cross = (ndc[1] - ndc[0]).cross(ndc[2] - ndc[0]);
            assert!(
                cross.z > 0.0,
                "visible face {:?} should be CCW in clip space, cross.z={}",
                normal,
                cross.z
            );
        }
    }

    #[test]
    fn side_face_uvs_keep_grass_fringe_on_top_z() {
        use crate::mesh::{face_corners, face_uvs, side_face_grass_fringe_on_top_z};
        use engine_assets::{pack_block_materials, CubeFace};
        use engine_assets::{blocks_asset_path, load_block_registry, textures_asset_path};

        let client = concat!(env!("CARGO_MANIFEST_DIR"), "/../../client");
        let registry = load_block_registry(&blocks_asset_path(client));
        let materials = pack_block_materials(&textures_asset_path(client), &registry).expect("pack");
        let grass = registry.id_by_name("grass").expect("grass");
        let rect = materials
            .tables()
            .default_faces
            .get(grass, CubeFace::Front)
            .expect("grass side")
            .atlas_rect;

        for face in [
            CubeFace::Right,
            CubeFace::Left,
            CubeFace::Front,
            CubeFace::Back,
        ] {
            let normal = match face {
                CubeFace::Right => [1.0, 0.0, 0.0],
                CubeFace::Left => [-1.0, 0.0, 0.0],
                CubeFace::Front => [0.0, 1.0, 0.0],
                CubeFace::Back => [0.0, -1.0, 0.0],
                _ => unreachable!(),
            };
            let corners = face_corners(glam::Vec3::ZERO, normal);
            let uvs = face_uvs(face, rect);
            assert!(
                side_face_grass_fringe_on_top_z(corners, uvs),
                "grass fringe should map to +Z on {face:?}"
            );
        }
    }

    #[test]
    fn grass_side_faces_carry_biome_tint_index() {
        let (mut world, registry, materials, biome) = grass_fixtures();
        let grass = registry.id_by_name("grass").expect("grass");
        world.set_block(BlockPos::new(0, 0, 0), grass);
        let mesh = mesh_chunk(&world, &registry, &materials, &biome, IVec3::ZERO, false);
        let tinted: Vec<_> = mesh
            .opaque
            .vertices
            .iter()
            .filter(|v| v.tint_index > 0)
            .collect();
        assert!(!tinted.is_empty(), "grass sides should carry biome tint index");
    }

    #[test]
    fn grass_bottom_face_has_no_biome_tint_index() {
        let (mut world, registry, materials, biome) = grass_fixtures();
        let grass = registry.id_by_name("grass").expect("grass");
        world.set_block(BlockPos::new(0, 0, 0), grass);
        let mesh = mesh_chunk(&world, &registry, &materials, &biome, IVec3::ZERO, false);
        let bottom: Vec<_> = mesh
            .opaque
            .vertices
            .iter()
            .filter(|v| v.normal[2] < -0.5)
            .collect();
        assert!(!bottom.is_empty(), "grass block should emit a bottom face");
        assert!(
            bottom.iter().all(|v| v.tint_index == 0),
            "grass bottom should not carry biome tint"
        );
    }
}
