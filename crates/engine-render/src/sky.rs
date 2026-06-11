use bytemuck::{Pod, Zeroable};
use engine_assets::UvRect;
use glam::{Mat4, Vec3};

use crate::lighting::LightingSnapshot;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SkyGpuUniform {
    pub inv_view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub sun_dir: [f32; 4],
    pub moon_dir: [f32; 4],
    pub horizon_color: [f32; 4],
    pub sun_color: [f32; 4],
    pub sun_strength: f32,
    pub moon_strength: f32,
    pub star_visibility: f32,
    pub sky_rotation_rad: f32,
    pub moon_phase: u32,
    pub moon_phase_count: u32,
    pub sun_rect_min: [f32; 2],
    pub sun_rect_max: [f32; 2],
    pub moon_rect_min: [f32; 2],
    pub moon_rect_max: [f32; 2],
    pub _pad: [f32; 2],
}

pub fn build_sky_uniform(
    inv_view_proj: Mat4,
    camera_pos: Vec3,
    snapshot: &LightingSnapshot,
    sun_rect: UvRect,
    moon_rect: UvRect,
    moon_phase_count: u32,
) -> SkyGpuUniform {
    SkyGpuUniform {
        inv_view_proj: inv_view_proj.to_cols_array_2d(),
        camera_pos: vec4(camera_pos),
        sun_dir: vec4(-snapshot.sun_dir),
        moon_dir: vec4(-snapshot.moon_dir),
        horizon_color: vec4(snapshot.horizon_color),
        sun_color: vec4(snapshot.sun_color),
        sun_strength: snapshot.sun_strength,
        moon_strength: snapshot.moon_strength,
        star_visibility: snapshot.star_visibility,
        sky_rotation_rad: snapshot.world_time / 24_000.0 * std::f32::consts::TAU,
        moon_phase: snapshot.moon_phase as u32,
        moon_phase_count,
        sun_rect_min: sun_rect.min,
        sun_rect_max: sun_rect.max,
        moon_rect_min: moon_rect.min,
        moon_rect_max: moon_rect.max,
        _pad: [0.0, 0.0],
    }
}

fn vec4(v: Vec3) -> [f32; 4] {
    [v.x, v.y, v.z, 0.0]
}

pub struct SkyPipeline {
    pub pipeline: wgpu::RenderPipeline,
    _bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

impl SkyPipeline {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        env_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sky_uniform_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sky.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout, env_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sky_buffer"),
            size: std::mem::size_of::<SkyGpuUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            _bind_group_layout: bind_group_layout,
            bind_group,
            buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, uniform: &SkyGpuUniform) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(uniform));
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        env_bind_group: &'a wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_bind_group(1, env_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_gpu_uniform_matches_wgsl_layout() {
        assert_eq!(std::mem::size_of::<SkyGpuUniform>(), 208);
    }
}
