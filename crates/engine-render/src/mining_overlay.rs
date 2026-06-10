use bytemuck::{Pod, Zeroable};
use engine_assets::{
    face_from_normal, BlockRegistry, DESTROY_STAGE_COUNT, ResolvedBlockMaterials, TextureAtlas,
    UvRect,
};
use engine_world::{BiomeMap, BlockPos, SparseVoxelOctree, VoxelCell};
use glam::{IVec3, Vec3};
use crate::ctm::neighbor_mask_for_face;
use crate::mesh::{
    face_corners, face_uvs, pack_vertex_anim, tint_index_for, VERTEX_FLAG_OVERLAY,
};

const FACE_DATA: [([f32; 3], IVec3); 6] = [
    ([1.0, 0.0, 0.0], IVec3::new(1, 0, 0)),
    ([-1.0, 0.0, 0.0], IVec3::new(-1, 0, 0)),
    ([0.0, 1.0, 0.0], IVec3::new(0, 1, 0)),
    ([0.0, -1.0, 0.0], IVec3::new(0, -1, 0)),
    ([0.0, 0.0, 1.0], IVec3::new(0, 0, 1)),
    ([0.0, 0.0, -1.0], IVec3::new(0, 0, -1)),
];

const VERTICES_PER_BLOCK: usize = 24;
const INDICES_PER_BLOCK: usize = 36;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MiningOverlayVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub mask_uv: [f32; 2],
    pub block_uv: [f32; 2],
    pub block_uv2: [f32; 2],
    pub tint_index: u32,
    pub flags: u32,
    pub anim_packed: u32,
}

#[derive(Debug, Clone)]
pub struct MiningOverlayMesh {
    pub block_pos: BlockPos,
    pub stage: u8,
    pub vertices: Vec<MiningOverlayVertex>,
    pub indices: Vec<u16>,
}

pub fn build_mining_overlay_mesh(
    block_pos: BlockPos,
    stage: u8,
    cell: VoxelCell,
    world: &SparseVoxelOctree,
    registry: &BlockRegistry,
    materials: &ResolvedBlockMaterials,
    biome: &BiomeMap,
) -> MiningOverlayMesh {
    let mut vertices = Vec::with_capacity(VERTICES_PER_BLOCK);
    let mut indices = Vec::with_capacity(INDICES_PER_BLOCK);
    let origin = block_pos.0.as_vec3();
    let mask_uvs = destroy_stage_uvs(stage);
    let black_uv2 = face_uvs(face_from_normal([0.0, 0.0, 1.0]), UvRect::BLACK);

    for (normal, _offset) in FACE_DATA {
        let face = face_from_normal(normal);
        let neighbors = registry
            .get(cell.id)
            .filter(|def| def.ctm.is_some())
            .map(|_| neighbor_mask_for_face(world, registry, block_pos, cell.id, face));

        let resolved = materials.resolve_face(cell.id, cell.state, face, neighbors);
        let tint_index = tint_index_for(resolved.tint, biome, block_pos);
        let block_uvs = face_uvs(face, resolved.atlas_rect);
        let block_uv2 = resolved
            .uv2
            .map(|rect| face_uvs(face, rect))
            .unwrap_or(black_uv2);
        let flags = if resolved.has_overlay() {
            VERTEX_FLAG_OVERLAY
        } else {
            0
        };
        let anim_packed = pack_vertex_anim(resolved.anim, resolved.atlas_rect);

        let epsilon = Vec3::from_array(normal) * 0.002;
        let corners = face_corners(origin, normal).map(|corner| corner + epsilon);
        let base = vertices.len() as u16;

        for (i, (corner, mask_uv)) in corners.iter().zip(mask_uvs).enumerate() {
            vertices.push(MiningOverlayVertex {
                position: corner.to_array(),
                normal,
                mask_uv,
                block_uv: block_uvs[i],
                block_uv2: block_uv2[i],
                tint_index,
                flags,
                anim_packed,
            });
        }

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    MiningOverlayMesh {
        block_pos,
        stage,
        vertices,
        indices,
    }
}

fn destroy_stage_uvs(stage: u8) -> [[f32; 2]; 4] {
    let stage = stage.min((DESTROY_STAGE_COUNT - 1) as u8) as f32;
    let u0 = stage / DESTROY_STAGE_COUNT as f32;
    let u1 = (stage + 1.0) / DESTROY_STAGE_COUNT as f32;
    [[u0, 1.0], [u1, 1.0], [u1, 0.0], [u0, 0.0]]
}

pub struct MiningOverlayPipeline {
    pipeline: wgpu::RenderPipeline,
    destroy_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uploaded: Option<(BlockPos, u8)>,
}

impl MiningOverlayPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        block_atlas_bind_group_layout: &wgpu::BindGroupLayout,
        destroy_atlas: &TextureAtlas,
    ) -> Self {
        let destroy_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("destroy_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let destroy_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("destroy_stage_texture"),
            size: wgpu::Extent3d {
                width: destroy_atlas.width,
                height: destroy_atlas.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &destroy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &destroy_atlas.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(destroy_atlas.width * 4),
                rows_per_image: Some(destroy_atlas.height),
            },
            wgpu::Extent3d {
                width: destroy_atlas.width,
                height: destroy_atlas.height,
                depth_or_array_layers: 1,
            },
        );
        let destroy_view = destroy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let destroy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("destroy_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let destroy_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("destroy_bind_group"),
            layout: &destroy_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&destroy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&destroy_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mining_overlay_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("mining_overlay.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mining_overlay_pipeline_layout"),
            bind_group_layouts: &[
                scene_bind_group_layout,
                block_atlas_bind_group_layout,
                &destroy_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mining_overlay_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<MiningOverlayVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 12,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 32,
                            shader_location: 3,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 40,
                            shader_location: 4,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 48,
                            shader_location: 5,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 52,
                            shader_location: 6,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 56,
                            shader_location: 7,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -1,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mining_overlay_vertex_buffer"),
            size: (VERTICES_PER_BLOCK * std::mem::size_of::<MiningOverlayVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mining_overlay_index_buffer"),
            size: (INDICES_PER_BLOCK * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            destroy_bind_group,
            vertex_buffer,
            index_buffer,
            index_count: 0,
            uploaded: None,
        }
    }

    pub fn sync_overlay(&mut self, queue: &wgpu::Queue, mesh: Option<&MiningOverlayMesh>) {
        let key = mesh.map(|mesh| (mesh.block_pos, mesh.stage));
        if self.uploaded == key {
            return;
        }
        self.uploaded = key;
        let Some(mesh) = mesh else {
            self.index_count = 0;
            return;
        };
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh.vertices),
        );
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&mesh.indices));
        self.index_count = mesh.indices.len() as u32;
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        scene_bind_group: &'a wgpu::BindGroup,
        block_atlas_bind_group: &'a wgpu::BindGroup,
    ) {
        if self.index_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, block_atlas_bind_group, &[]);
        pass.set_bind_group(2, &self.destroy_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destroy_stage_uvs_span_one_tile() {
        let uvs = destroy_stage_uvs(3);
        assert!((uvs[0][0] - 0.3).abs() < 1e-5);
        assert!((uvs[1][0] - 0.4).abs() < 1e-5);
    }
}
