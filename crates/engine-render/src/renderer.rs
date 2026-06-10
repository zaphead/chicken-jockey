use std::sync::Arc;

use engine_assets::{ResolvedBlockMaterials, UvRect};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::hud::HudPipeline;
use crate::mesh::SolidMesh;
use crate::pipeline::{GpuMesh, RenderPipelines};
use crate::world_mesh::RenderScene;

pub struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    pipelines: RenderPipelines,
    colormap_rect: Option<UvRect>,
    opaque_meshes: Vec<GpuMesh>,
    cutout_meshes: Vec<GpuMesh>,
    uploaded_mesh_generation: u64,
    hud: HudPipeline,
}

impl Renderer {
    pub fn new(
        window: Arc<Window>,
        materials: &ResolvedBlockMaterials,
        destroy_atlas: &engine_assets::TextureAtlas,
    ) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .find(|adapter| adapter.is_surface_supported(&surface))
            .expect("compatible adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        ))
        .expect("request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (depth_texture, depth_view) =
            create_depth_texture(&device, config.width, config.height);

        let colormap_rect = materials.colormap_atlas_rect;
        let pipelines = RenderPipelines::new(
            &device,
            &queue,
            surface_format,
            &materials.atlas,
            destroy_atlas,
            colormap_rect,
        );

        let hud = HudPipeline::new(&device, surface_format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            pipelines,
            colormap_rect,
            opaque_meshes: Vec::new(),
            cutout_meshes: Vec::new(),
            uploaded_mesh_generation: u64::MAX,
            hud,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        let (depth_texture, depth_view) =
            create_depth_texture(&self.device, size.width, size.height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
    }

    pub fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn sync_meshes(
        &mut self,
        mesh_generation: u64,
        opaque: &SolidMesh,
        cutout: &SolidMesh,
    ) {
        if self.uploaded_mesh_generation == mesh_generation {
            return;
        }
        self.uploaded_mesh_generation = mesh_generation;
        self.opaque_meshes = if opaque.vertices.is_empty() {
            Vec::new()
        } else {
            vec![GpuMesh::from_mesh(&self.device, opaque)]
        };
        self.cutout_meshes = if cutout.vertices.is_empty() {
            Vec::new()
        } else {
            vec![GpuMesh::from_mesh(&self.device, cutout)]
        };
    }

    pub fn render(&mut self, scene: &RenderScene, hud_label: Option<&str>) -> Result<(), SurfaceError> {
        self.hud.set_text(
            &self.queue,
            hud_label.unwrap_or(""),
            self.config.width,
            self.config.height,
        );
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.pipelines.update_scene(
            &self.queue,
            scene.camera.view_projection(),
            scene.animation_tick,
            self.colormap_rect,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("depth_pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipelines.depth);
            pass.set_bind_group(0, &self.pipelines.scene_bind_group, &[]);
            pass.set_bind_group(1, &self.pipelines.atlas_bind_group, &[]);
            draw_meshes(&mut pass, &self.opaque_meshes);
            draw_meshes(&mut pass, &self.cutout_meshes);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.98,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipelines.opaque);
            pass.set_bind_group(0, &self.pipelines.scene_bind_group, &[]);
            pass.set_bind_group(1, &self.pipelines.atlas_bind_group, &[]);
            draw_meshes(&mut pass, &self.opaque_meshes);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cutout_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipelines.cutout);
            pass.set_bind_group(0, &self.pipelines.scene_bind_group, &[]);
            pass.set_bind_group(1, &self.pipelines.atlas_bind_group, &[]);
            draw_meshes(&mut pass, &self.cutout_meshes);
        }

        let mining_mesh = scene.mining_overlay.as_ref().map(|overlay| &overlay.mesh);
        self.pipelines
            .mining_overlay
            .sync_overlay(&self.queue, mining_mesh);

        self.pipelines
            .outline
            .sync_block(&self.queue, scene.target_block);
        if scene.target_block.is_some() {
            let outline_count = crate::outline::OutlinePipeline::index_count_for_block();
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("outline_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.pipelines.outline.draw(
                &mut pass,
                &self.pipelines.scene_bind_group,
                outline_count,
            );
        }

        if mining_mesh.is_some() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mining_overlay_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.pipelines.mining_overlay.draw(
                &mut pass,
                &self.pipelines.scene_bind_group,
                &self.pipelines.atlas_bind_group,
            );
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            self.hud.draw(&mut pass);
        }

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        output.present();
        Ok(())
    }
}

fn draw_meshes(pass: &mut wgpu::RenderPass<'_>, meshes: &[GpuMesh]) {
    for mesh in meshes {
        pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
