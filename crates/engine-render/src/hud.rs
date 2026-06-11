use bytemuck::{Pod, Zeroable};

use crate::screen_text::{glyph_rows, widget_char_width, widget_glyph_pixel, widget_line_height, CELL};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HudVertex {
    pos: [f32; 2],
    color: [f32; 4],
}

const MAX_HUD_LINES: usize = 18;
const MAX_HUD_CHARS_PER_LINE: usize = 24;
const MAX_HUD_VERTICES: usize = MAX_HUD_LINES * MAX_HUD_CHARS_PER_LINE * 5 * 7 * 6 + 6 * 8;
const HUD_VERTEX_BUFFER_SIZE: u64 =
    (MAX_HUD_VERTICES * std::mem::size_of::<HudVertex>()) as u64;

const BG_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.72];
const TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
const CROSSHAIR_OUTLINE: [f32; 4] = [0.0, 0.0, 0.0, 0.82];
const CROSSHAIR_CORE: [f32; 4] = [1.0, 1.0, 1.0, 0.92];

const BASE_PADDING: f32 = 16.0;
const BASE_BG_PAD: f32 = 8.0;
const BASE_CROSSHAIR_ARM: f32 = 11.0;
const BASE_CROSSHAIR_GAP: f32 = 3.0;
const BASE_CROSSHAIR_CORE: f32 = 1.25;
const BASE_CROSSHAIR_OUTLINE: f32 = 2.75;

pub struct HudPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    crosshair_vertices: Vec<HudVertex>,
    crosshair_key: (u32, u32, u32),
}

impl HudPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hud.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<HudVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
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
            label: Some("hud_vertex_buffer"),
            size: HUD_VERTEX_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_count: 0,
            crosshair_vertices: Vec::new(),
            crosshair_key: (0, 0, 0),
        }
    }

    pub fn set_text(
        &mut self,
        queue: &wgpu::Queue,
        text: &str,
        width: u32,
        height: u32,
        scale: f32,
        show_crosshair: bool,
    ) {
        let scale_key = scale.to_bits();
        if self.crosshair_key != (width, height, scale_key) {
            self.crosshair_vertices.clear();
            append_crosshair(&mut self.crosshair_vertices, width, height, scale);
            self.crosshair_key = (width, height, scale_key);
        }
        let mut vertices = build_hud_vertices(text, width, height, scale);
        if show_crosshair {
            vertices.extend_from_slice(&self.crosshair_vertices);
        }
        vertices.truncate(MAX_HUD_VERTICES);
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
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

fn build_hud_vertices(text: &str, width: u32, height: u32, scale: f32) -> Vec<HudVertex> {
    let padding = BASE_PADDING * scale;
    let bg_pad = BASE_BG_PAD * scale;
    let row_height = widget_line_height(scale) + 2.0 * scale;
    let pixel = widget_glyph_pixel(scale);

    let lines: Vec<&str> = text.lines().take(MAX_HUD_LINES).collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let max_line_chars = lines
        .iter()
        .map(|line| line.chars().count().min(MAX_HUD_CHARS_PER_LINE))
        .max()
        .unwrap_or(0) as f32;
    let panel_w = max_line_chars * widget_char_width(scale) + bg_pad * 2.0;
    let panel_h = lines.len() as f32 * row_height - 2.0 * scale + bg_pad * 2.0;

    let mut vertices = Vec::new();
    push_quad(
        &mut vertices,
        padding - bg_pad,
        padding - bg_pad,
        padding - bg_pad + panel_w,
        padding - bg_pad + panel_h,
        width,
        height,
        BG_COLOR,
    );

    for (line_index, line) in lines.iter().enumerate() {
        let py = padding + line_index as f32 * row_height;
        let mut cursor_x = padding;

        for ch in line.chars().take(MAX_HUD_CHARS_PER_LINE) {
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
                        let y0 = py + row as f32 * pixel;
                        push_quad(
                            &mut vertices,
                            x0,
                            y0,
                            x0 + pixel,
                            y0 + pixel,
                            width,
                            height,
                            TEXT_COLOR,
                        );
                    }
                }
            }
            cursor_x += widget_char_width(scale);
        }
    }

    vertices
}

fn push_quad(
    vertices: &mut Vec<HudVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    width: u32,
    height: u32,
    color: [f32; 4],
) {
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    let vtx = |x: f32, y: f32| HudVertex {
        pos: [(x / w) * 2.0 - 1.0, 1.0 - (y / h) * 2.0],
        color,
    };
    vertices.push(vtx(x0, y0));
    vertices.push(vtx(x1, y0));
    vertices.push(vtx(x0, y1));
    vertices.push(vtx(x1, y0));
    vertices.push(vtx(x1, y1));
    vertices.push(vtx(x0, y1));
}

fn append_crosshair(vertices: &mut Vec<HudVertex>, width: u32, height: u32, scale: f32) {
    let cx = width.max(1) as f32 * 0.5;
    let cy = height.max(1) as f32 * 0.5;
    let arm = BASE_CROSSHAIR_ARM * scale;
    let gap = BASE_CROSSHAIR_GAP * scale;
    let core = BASE_CROSSHAIR_CORE * scale;
    let outline = BASE_CROSSHAIR_OUTLINE * scale;

    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - arm - scale,
        cy - outline,
        cx - gap + scale,
        cy + outline,
        CROSSHAIR_OUTLINE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx + gap - scale,
        cy - outline,
        cx + arm + scale,
        cy + outline,
        CROSSHAIR_OUTLINE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - outline,
        cy - arm - scale,
        cx + outline,
        cy - gap + scale,
        CROSSHAIR_OUTLINE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - outline,
        cy + gap - scale,
        cx + outline,
        cy + arm + scale,
        CROSSHAIR_OUTLINE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - arm,
        cy - core,
        cx - gap,
        cy + core,
        CROSSHAIR_CORE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx + gap,
        cy - core,
        cx + arm,
        cy + core,
        CROSSHAIR_CORE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - core,
        cy - arm,
        cx + core,
        cy - gap,
        CROSSHAIR_CORE,
    );
    push_crosshair_bar(
        vertices,
        width,
        height,
        cx - core,
        cy + gap,
        cx + core,
        cy + arm,
        CROSSHAIR_CORE,
    );
}

fn push_crosshair_bar(
    vertices: &mut Vec<HudVertex>,
    width: u32,
    height: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: [f32; 4],
) {
    push_quad(vertices, x0, y0, x1, y1, width, height, color);
}
