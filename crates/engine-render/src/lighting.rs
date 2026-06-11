use bytemuck::{Pod, Zeroable};
use engine_assets::{EnvironmentTextures, UvRect};
use glam::{Mat4, Vec3};
pub const SHADOW_MAP_SIZE: u32 = 4096;
pub const SHADOW_ORTHO_HALF_SIZE: f32 = 96.0;

#[derive(Debug, Clone, Copy)]
pub struct LightingSnapshot {
    pub sun_dir: Vec3,
    pub moon_dir: Vec3,
    pub sun_color: Vec3,
    pub moon_color: Vec3,
    pub ambient_color: Vec3,
    pub horizon_color: Vec3,
    pub sun_strength: f32,
    pub moon_strength: f32,
    pub star_visibility: f32,
    pub night_darkness: f32,
    pub moon_phase: u8,
    pub world_time: f32,
    pub sun_elevation: f32,
}

impl Default for LightingSnapshot {
    fn default() -> Self {
        Self {
            sun_dir: Vec3::new(0.0, 0.0, -1.0),
            moon_dir: Vec3::new(0.0, 0.0, 1.0),
            sun_color: Vec3::ONE,
            moon_color: Vec3::new(0.7, 0.8, 1.0),
            ambient_color: Vec3::new(0.45, 0.5, 0.55),
            horizon_color: Vec3::new(0.55, 0.75, 0.95),
            sun_strength: 1.0,
            moon_strength: 0.0,
            star_visibility: 0.0,
            night_darkness: 0.0,
            moon_phase: 0,
            world_time: 6_000.0,
            sun_elevation: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightingGpuUniform {
    pub sun_dir: [f32; 4],
    pub moon_dir: [f32; 4],
    pub sun_color: [f32; 4],
    pub ambient_color: [f32; 4],
    pub moon_color: [f32; 4],
    pub camera_pos: [f32; 4],
    pub horizon_color: [f32; 4],
    pub light_view_proj: [[f32; 4]; 4],
    pub sun_strength: f32,
    pub moon_strength: f32,
    pub star_visibility: f32,
    pub night_darkness: f32,
    pub specular_strength: f32,
    pub moon_phase: u32,
    pub moon_phase_count: u32,
    pub world_time: f32,
    pub fog_density: f32,
    pub _pad_before_rects: f32,
    pub sun_rect_min: [f32; 2],
    pub sun_rect_max: [f32; 2],
    pub moon_rect_min: [f32; 2],
    pub moon_rect_max: [f32; 2],
    pub sky_colormap_min: [f32; 2],
    pub sky_colormap_max: [f32; 2],
    pub fog_colormap_min: [f32; 2],
    pub fog_colormap_max: [f32; 2],
    pub _pad_end: [f32; 2],
}

pub fn compute_light_view_proj(sun_light_dir: Vec3, focus: Vec3) -> Mat4 {
    let toward_scene = sun_light_dir.normalize_or_zero();
    let light_pos = focus - toward_scene * 280.0;
    let forward = toward_scene;
    let up = if forward.z.abs() > 0.95 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let view = Mat4::look_to_rh(light_pos, forward, up);
    let h = SHADOW_ORTHO_HALF_SIZE;

    // Snap shadow map to a world-space texel grid so shadows don't swim with the camera.
    let focus_in_light = view.transform_point3(focus);
    let world_per_texel = (2.0 * h) / SHADOW_MAP_SIZE as f32;
    let snapped = Vec3::new(
        (focus_in_light.x / world_per_texel).round() * world_per_texel,
        (focus_in_light.y / world_per_texel).round() * world_per_texel,
        focus_in_light.z,
    );
    let view_snapped = Mat4::from_translation(snapped - focus_in_light) * view;

    let proj = Mat4::orthographic_rh(-h, h, -h, h, 1.0, 520.0);
    proj * view_snapped
}

pub struct LightingResources {
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group: wgpu::BindGroup,
    pub shadow_bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
    pub shadow_texture: wgpu::Texture,
    pub shadow_view: wgpu::TextureView,
    pub shadow_sampler: wgpu::Sampler,
    pub env_bind_group_layout: wgpu::BindGroupLayout,
    pub env_bind_group: wgpu::BindGroup,
    _env_texture: wgpu::Texture,
    sun_rect: UvRect,
    moon_rect: UvRect,
    sky_rect: UvRect,
    fog_rect: UvRect,
    moon_phase_count: u32,
}

impl LightingResources {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        env: &EnvironmentTextures,
    ) -> Self {
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("lighting_uniform_layout"),
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

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_sample_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_map"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_comparison_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lighting_buffer"),
            size: std::mem::size_of::<LightingGpuUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lighting_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_sample_bind_group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        let env_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("env_bind_group_layout"),
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

        let (env_texture, env_bind_group) =
            upload_env_atlas(device, queue, &env_bind_group_layout, env);

        Self {
            uniform_bind_group_layout,
            shadow_bind_group_layout,
            uniform_bind_group,
            shadow_bind_group,
            buffer,
            shadow_texture,
            shadow_view,
            shadow_sampler,
            env_bind_group_layout,
            env_bind_group,
            _env_texture: env_texture,
            sun_rect: env.sun_rect,
            moon_rect: env.moon_strip_rect,
            sky_rect: env.sky_colormap_rect,
            fog_rect: env.fog_colormap_rect,
            moon_phase_count: env.moon_phase_count,
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        snapshot: &LightingSnapshot,
        camera_pos: Vec3,
        light_view_proj: Mat4,
    ) {
        let uniform = LightingGpuUniform {
            sun_dir: vec4(snapshot.sun_dir),
            moon_dir: vec4(snapshot.moon_dir),
            sun_color: vec4(snapshot.sun_color),
            ambient_color: vec4(snapshot.ambient_color),
            moon_color: vec4(snapshot.moon_color),
            camera_pos: vec4(camera_pos),
            horizon_color: vec4(snapshot.horizon_color),
            light_view_proj: light_view_proj.to_cols_array_2d(),
            sun_strength: snapshot.sun_strength,
            moon_strength: snapshot.moon_strength,
            star_visibility: snapshot.star_visibility,
            night_darkness: snapshot.night_darkness,
            specular_strength: 0.07,
            moon_phase: snapshot.moon_phase as u32,
            moon_phase_count: self.moon_phase_count,
            world_time: snapshot.world_time,
            fog_density: 0.0022,
            _pad_before_rects: 0.0,
            sun_rect_min: self.sun_rect.min,
            sun_rect_max: self.sun_rect.max,
            moon_rect_min: self.moon_rect.min,
            moon_rect_max: self.moon_rect.max,
            sky_colormap_min: self.sky_rect.min,
            sky_colormap_max: self.sky_rect.max,
            fog_colormap_min: self.fog_rect.min,
            fog_colormap_max: self.fog_rect.max,
            _pad_end: [0.0, 0.0],
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn sky_uniform(
        &self,
        inv_view_proj: Mat4,
        camera_pos: Vec3,
        snapshot: &LightingSnapshot,
    ) -> crate::sky::SkyGpuUniform {
        crate::sky::build_sky_uniform(
            inv_view_proj,
            camera_pos,
            snapshot,
            self.sun_rect,
            self.moon_rect,
            self.moon_phase_count,
        )
    }
}

fn vec4(v: Vec3) -> [f32; 4] {
    [v.x, v.y, v.z, 0.0]
}

fn upload_env_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    env: &EnvironmentTextures,
) -> (wgpu::Texture, wgpu::BindGroup) {
    let atlas = &env.atlas;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("env_atlas"),
        size: wgpu::Extent3d {
            width: atlas.width.max(1),
            height: atlas.height.max(1),
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
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &atlas.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * atlas.width),
            rows_per_image: Some(atlas.height),
        },
        wgpu::Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("env_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("env_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (texture, bind_group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighting_gpu_uniform_matches_wgsl_layout() {
        assert_eq!(std::mem::size_of::<LightingGpuUniform>(), 288);
    }
}
