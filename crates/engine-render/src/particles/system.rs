use engine_assets::{BlockRegistry, CubeFace, ResolvedBlockMaterials, UvRect};
use engine_world::{BiomeMap, BlockPos, SparseVoxelOctree, VoxelCell};
use glam::Vec3;

use crate::camera::Camera;
use crate::mesh::tint_index_for;

const PARTICLES_PER_BREAK: usize = 60;
const PARTICLE_LIFETIME: f32 = 1.5;
const PARTICLE_GRAVITY: f32 = 18.0;
const MAX_PARTICLES: usize = 2560;

#[derive(Debug, Clone, Copy)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    age: f32,
    lifetime: f32,
    uv_rect: UvRect,
    tint_index: u32,
    size: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub tint_index: u32,
    pub alpha: f32,
}

#[derive(Debug, Default, Clone)]
pub struct ParticleMesh {
    pub vertices: Vec<ParticleVertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug, Default)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn spawn_block_break(
        &mut self,
        block_pos: BlockPos,
        cell: VoxelCell,
        registry: &BlockRegistry,
        materials: &ResolvedBlockMaterials,
        biome: &BiomeMap,
    ) {
        if cell.id == 0 || !registry.is_breakable(cell.id) {
            return;
        }

        let center = block_pos.0.as_vec3() + Vec3::splat(0.5);
        let seed_base = block_pos.0.x as u32
            ^ (block_pos.0.y as u32).wrapping_mul(374761393)
            ^ (block_pos.0.z as u32).wrapping_mul(668265263);

        for i in 0..PARTICLES_PER_BREAK {
            let i = i as u32;
            if self.particles.len() >= MAX_PARTICLES {
                return;
            }

            let face = CubeFace::ALL[hash_u32(seed_base.wrapping_add(i * 97)) as usize % 6];
            let resolved = materials.resolve_face(cell.id, cell.state, face, None);
            let tint_index = tint_index_for(resolved.tint, biome, block_pos);
            let uv_rect = random_sub_rect(
                resolved.atlas_rect,
                seed_base.wrapping_add(i * 131),
            );

            let jitter = Vec3::new(
                hash_f32(seed_base, i * 3) - 0.5,
                hash_f32(seed_base, i * 3 + 1) - 0.5,
                hash_f32(seed_base, i * 3 + 2) - 0.5,
            ) * 0.35;
            let dir = jitter.normalize_or_zero();
            let speed = 1.5 + hash_f32(seed_base, i * 5) * 3.5;
            let upward = 1.0 + hash_f32(seed_base, i * 7) * 2.5;

            self.particles.push(Particle {
                position: center + jitter * 0.2,
                velocity: dir * speed + Vec3::Z * upward,
                age: 0.0,
                lifetime: PARTICLE_LIFETIME * (0.75 + hash_f32(seed_base, i * 11) * 0.5),
                uv_rect,
                tint_index,
                size: 0.04 + hash_f32(seed_base, i * 13) * 0.05,
            });
        }
    }

    pub fn tick(&mut self, dt: f32, world: &SparseVoxelOctree) {
        self.particles.retain_mut(|particle| {
            particle.age += dt;
            particle.velocity.z -= PARTICLE_GRAVITY * dt;
            particle.position += particle.velocity * dt;
            if particle.velocity.z < 0.0 {
                resolve_ground(particle, world);
            }
            particle.age < particle.lifetime
        });
    }

    pub fn build_mesh(&self, camera: &Camera) -> ParticleMesh {
        if self.particles.is_empty() {
            return ParticleMesh::default();
        }

        let mut mesh = ParticleMesh {
            vertices: Vec::with_capacity(self.particles.len() * 4),
            indices: Vec::with_capacity(self.particles.len() * 6),
        };

        let camera_right = camera.right();
        let camera_up = camera.up();

        for particle in &self.particles {
            let t = particle.age / particle.lifetime;
            let alpha = (1.0 - t * t).clamp(0.0, 1.0);
            let right = camera_right * particle.size;
            let up = camera_up * particle.size;
            let center = particle.position;
            let corners = [
                center - right - up,
                center + right - up,
                center + right + up,
                center - right + up,
            ];
            let uvs = rect_uvs(particle.uv_rect);
            let base = mesh.vertices.len() as u16;

            for (corner, uv) in corners.iter().zip(uvs) {
                mesh.vertices.push(ParticleVertex {
                    position: corner.to_array(),
                    uv,
                    tint_index: particle.tint_index,
                    alpha,
                });
            }

            mesh.indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base,
                base + 2,
                base + 3,
            ]);
        }

        mesh
    }

    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }
}

fn resolve_ground(particle: &mut Particle, world: &SparseVoxelOctree) {
    let bottom_z = particle.position.z - particle.size;
    let below = BlockPos::new(
        particle.position.x.floor() as i32,
        particle.position.y.floor() as i32,
        bottom_z.floor() as i32,
    );
    if !world.is_solid(below) {
        return;
    }
    let floor_z = below.0.z as f32 + 1.0;
    if bottom_z >= floor_z {
        return;
    }
    particle.position.z = floor_z + particle.size;
    particle.velocity.z = 0.0;
    particle.velocity.x *= 0.55;
    particle.velocity.y *= 0.55;
}

fn rect_uvs(rect: UvRect) -> [[f32; 2]; 4] {
    let [u0, v0] = rect.min;
    let [u1, v1] = rect.max;
    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}

fn random_sub_rect(tile: UvRect, seed: u32) -> UvRect {
    let span_u = tile.max[0] - tile.min[0];
    let span_v = tile.max[1] - tile.min[1];
    let quarter_u = span_u * 0.25;
    let quarter_v = span_v * 0.25;
    let offset_u = hash_f32(seed, 0) * (span_u - quarter_u);
    let offset_v = hash_f32(seed, 1) * (span_v - quarter_v);
    UvRect {
        min: [tile.min[0] + offset_u, tile.min[1] + offset_v],
        max: [
            tile.min[0] + offset_u + quarter_u,
            tile.min[1] + offset_v + quarter_v,
        ],
    }
}

fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

fn hash_f32(seed: u32, index: u32) -> f32 {
    hash_u32(seed.wrapping_add(index.wrapping_mul(2654435761))) as f32 / u32::MAX as f32
}
