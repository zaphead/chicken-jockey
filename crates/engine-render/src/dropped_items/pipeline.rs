use crate::particles::{ParticleMesh, ParticleVertex};

pub struct ItemToolPipeline {
    hdr_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pub index_count: u32,
    capacity_vertices: usize,
    capacity_indices: usize,
}

impl ItemToolPipeline {
    pub fn new(
        device: &wgpu::Device,
        hdr_format: wgpu::TextureFormat,
        scene_bind_group_layout: &wgpu::BindGroupLayout,
        gui_atlas_bind_group_layout: &wgpu::BindGroupLayout,
        lighting_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("item_tool_shader"),
            source: crate::shader_source::item_tool_shader_source(),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("item_tool_pipeline_layout"),
            bind_group_layouts: &[
                scene_bind_group_layout,
                gui_atlas_bind_group_layout,
                lighting_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ParticleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }];

        let depth_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let hdr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("item_tool_hdr_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_item"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: hdr_format,
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
            depth_stencil: Some(depth_stencil),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let capacity_vertices = 512;
        let capacity_indices = 768;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("item_tool_vertex_buffer"),
            size: (capacity_vertices * std::mem::size_of::<ParticleVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("item_tool_index_buffer"),
            size: (capacity_indices * std::mem::size_of::<u16>()) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            hdr_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: 0,
            capacity_vertices,
            capacity_indices,
        }
    }

    pub fn sync_mesh(&mut self, queue: &wgpu::Queue, mesh: Option<&ParticleMesh>) {
        let Some(mesh) = mesh.filter(|mesh| !mesh.vertices.is_empty()) else {
            self.index_count = 0;
            return;
        };

        let vertex_count = mesh.vertices.len().min(self.capacity_vertices);
        let index_count = mesh.indices.len().min(self.capacity_indices);
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&mesh.vertices[..vertex_count]),
        );
        queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&mesh.indices[..index_count]),
        );
        self.index_count = index_count as u32;
    }

    pub fn draw_hdr<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        scene_bind_group: &'a wgpu::BindGroup,
        gui_atlas_bind_group: &'a wgpu::BindGroup,
        lighting_bind_group: &'a wgpu::BindGroup,
    ) {
        if self.index_count == 0 {
            return;
        }
        pass.set_pipeline(&self.hdr_pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, gui_atlas_bind_group, &[]);
        pass.set_bind_group(2, lighting_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
