mod frame;

use bytemuck::{Pod, Zeroable};
use engine_assets::{GuiTextures, NineSliceSprite, UvRect};

pub use frame::{GuiButton, GuiFrame, GuiLabel, GuiPanel, GuiRect};

const MAX_GUI_VERTICES: usize = 4096;
const GUI_VERTEX_BUFFER_SIZE: u64 =
    (MAX_GUI_VERTICES * std::mem::size_of::<GuiVertex>()) as u64;

const DIMMER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.55];
const TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GuiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

pub struct GuiPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl GuiPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        textures: &GuiTextures,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gui_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("gui.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gui_bind_group_layout"),
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

        let bind_group = upload_gui_atlas(device, queue, &bind_group_layout, textures);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gui_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gui_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<GuiVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
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
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gui_vertex_buffer"),
            size: GUI_VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            vertex_buffer,
            vertex_count: 0,
        }
    }

    pub fn set_frame(&mut self, queue: &wgpu::Queue, frame: &GuiFrame, textures: &GuiTextures) {
        let mut vertices = Vec::new();
        if frame.dim_background {
            push_solid_quad(
                &mut vertices,
                0.0,
                0.0,
                frame.width as f32,
                frame.height as f32,
                frame.width,
                frame.height,
                textures.solid_uv,
                DIMMER_COLOR,
            );
        }

        for panel in &frame.panels {
            append_nine_slice(
                &mut vertices,
                &panel.rect,
                frame.width,
                frame.height,
                &textures.panel,
                WHITE,
            );
        }

        for button in &frame.buttons {
            let sprite = if button.highlighted {
                &textures.button_highlighted
            } else {
                &textures.button
            };
            append_nine_slice(
                &mut vertices,
                &button.rect,
                frame.width,
                frame.height,
                sprite,
                WHITE,
            );
        }

        for label in &frame.labels {
            append_bitmap_label(
                &mut vertices,
                label.x,
                label.y,
                &label.text,
                frame.width,
                frame.height,
                frame.scale,
                textures.solid_uv,
                TEXT_COLOR,
            );
        }

        vertices.truncate(MAX_GUI_VERTICES);
        self.vertex_count = vertices.len() as u32;
        if !vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

fn upload_gui_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    textures: &GuiTextures,
) -> wgpu::BindGroup {
    let atlas = &textures.atlas;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("gui_atlas"),
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
        label: Some("gui_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("gui_bind_group"),
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
    })
}

fn append_nine_slice(
    vertices: &mut Vec<GuiVertex>,
    rect: &GuiRect,
    width: u32,
    height: u32,
    sprite: &NineSliceSprite,
    color: [f32; 4],
) {
    let scale = rect.h / sprite.height as f32;
    let left = sprite.border_left as f32 * scale;
    let right = sprite.border_right as f32 * scale;
    let top = sprite.border_top as f32 * scale;
    let bottom = sprite.border_bottom as f32 * scale;

    let x0 = rect.x;
    let y0 = rect.y;
    let x1 = rect.x + rect.w;
    let y1 = rect.y + rect.h;
    let xc0 = x0 + left;
    let xc1 = x1 - right;
    let yc0 = y0 + top;
    let yc1 = y1 - bottom;

    let uv = sprite.uv;
    let du = uv.max[0] - uv.min[0];
    let dv = uv.max[1] - uv.min[1];
    let u_left = uv.min[0] + du * (sprite.border_left as f32 / sprite.width as f32);
    let u_right = uv.max[0] - du * (sprite.border_right as f32 / sprite.width as f32);
    let v_top = uv.min[1] + dv * (sprite.border_top as f32 / sprite.height as f32);
    let v_bottom = uv.max[1] - dv * (sprite.border_bottom as f32 / sprite.height as f32);

    let xs = [x0, xc0, xc1, x1];
    let ys = [y0, yc0, yc1, y1];
    let us = [uv.min[0], u_left, u_right, uv.max[0]];
    let vs = [uv.min[1], v_top, v_bottom, uv.max[1]];

    for row in 0..3 {
        for col in 0..3 {
            push_textured_quad(
                vertices,
                xs[col],
                ys[row],
                xs[col + 1],
                ys[row + 1],
                width,
                height,
                us[col],
                vs[row],
                us[col + 1],
                vs[row + 1],
                color,
            );
        }
    }
}

fn append_bitmap_label(
    vertices: &mut Vec<GuiVertex>,
    x: f32,
    y: f32,
    text: &str,
    width: u32,
    height: u32,
    scale: f32,
    solid_uv: UvRect,
    color: [f32; 4],
) {
    use crate::screen_text::{glyph_rows, widget_char_width, widget_glyph_pixel, CELL};

    let pixel = widget_glyph_pixel(scale);
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == ' ' {
            cursor_x += widget_char_width(scale);
            continue;
        }
        let Some(glyph) = glyph_rows(ch) else {
            continue;
        };
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..CELL as usize {
                if bits & (1 << (CELL as usize - 1 - col)) != 0 {
                    let x0 = cursor_x + col as f32 * pixel;
                    let y0 = y + row as f32 * pixel;
                    push_solid_quad(
                        vertices,
                        x0,
                        y0,
                        x0 + pixel,
                        y0 + pixel,
                        width,
                        height,
                        solid_uv,
                        color,
                    );
                }
            }
        }
        cursor_x += widget_char_width(scale);
    }
}

fn push_textured_quad(
    vertices: &mut Vec<GuiVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    width: u32,
    height: u32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    color: [f32; 4],
) {
    let vtx = |x: f32, y: f32, u: f32, v: f32| GuiVertex {
        pos: screen_to_ndc(x, y, width, height),
        uv: [u, v],
        color,
    };
    vertices.push(vtx(x0, y0, u0, v0));
    vertices.push(vtx(x1, y0, u1, v0));
    vertices.push(vtx(x0, y1, u0, v1));
    vertices.push(vtx(x1, y0, u1, v0));
    vertices.push(vtx(x1, y1, u1, v1));
    vertices.push(vtx(x0, y1, u0, v1));
}

fn push_solid_quad(
    vertices: &mut Vec<GuiVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    width: u32,
    height: u32,
    uv: UvRect,
    color: [f32; 4],
) {
    push_textured_quad(
        vertices, x0, y0, x1, y1, width, height, uv.min[0], uv.min[1], uv.max[0], uv.max[1], color,
    );
}

fn screen_to_ndc(x: f32, y: f32, width: u32, height: u32) -> [f32; 2] {
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    [(x / w) * 2.0 - 1.0, 1.0 - (y / h) * 2.0]
}
