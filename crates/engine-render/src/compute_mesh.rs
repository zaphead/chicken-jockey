use std::sync::Arc;

use engine_assets::BlockRegistry;
use engine_world::{BlockPos, CHUNK_SIZE, SparseVoxelOctree};
use glam::IVec3;
use wgpu::util::DeviceExt;

use crate::mesh::SolidMesh;
use crate::world_mesh::{lod_step_for_chunk, mesh_chunk_with_lod};

/// GPU compute validation + CPU mesh output for terrain chunks.
pub struct ComputeMesher {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl ComputeMesher {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_mesh_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_mesh.wgsl").into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_mesh_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute_mesh_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_mesh_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn mesh_chunk(
        &self,
        world: &SparseVoxelOctree,
        registry: &BlockRegistry,
        chunk: IVec3,
        camera_position: glam::Vec3,
    ) -> SolidMesh {
        let step = lod_step_for_chunk(chunk, camera_position);
        let voxels = pack_chunk_voxels(world, registry, chunk, step);

        let voxel_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_voxels"),
            contents: bytemuck::cast_slice(&voxels),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let output = [0u32];
        let output_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("solid_count"),
            contents: bytemuck::cast_slice(&output),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_mesh_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: voxel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("compute_mesh_encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_mesh_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            let workgroups = ((voxels.len() as u32) + 63) / 64;
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
        self.queue.submit(Some(encoder.finish()));

        mesh_chunk_with_lod(world, registry, chunk, step, camera_position)
    }
}

fn pack_chunk_voxels(
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    chunk: IVec3,
    step: i32,
) -> Vec<u32> {
    let origin = chunk * CHUNK_SIZE;
    let mut voxels = Vec::new();
    let mut x = 0;
    while x < CHUNK_SIZE {
        let mut y = 0;
        while y < CHUNK_SIZE {
            let mut z = 0;
            while z < CHUNK_SIZE {
                let pos = BlockPos::new(origin.x + x, origin.y + y, origin.z + z);
                let block = world.get_block(pos);
                voxels.push(if registry.is_solid(block) { 1 } else { 0 });
                z += step;
            }
            y += step;
        }
        x += step;
    }
    voxels
}
