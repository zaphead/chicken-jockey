use std::sync::Arc;

use engine_assets::{EnvironmentTextures, GuiTextures, ResolvedBlockMaterials, UvRect};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::gui::{GuiFrame, GuiPipeline};
use crate::hud::HudPipeline;
use crate::lighting::{compute_light_view_proj, LightingResources};
use crate::render_passes;
use crate::sky::SkyPipeline;
use crate::mesh::SolidMesh;
use crate::pipeline::{GpuMesh, RenderPipelines};
use crate::post::{PostGpuUniform, PostPipeline};
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
    lighting: LightingResources,
    sky: SkyPipeline,
    post: PostPipeline,
    colormap_rect: Option<UvRect>,
    opaque_meshes: Vec<GpuMesh>,
    cutout_meshes: Vec<GpuMesh>,
    uploaded_mesh_generation: u64,
    hud: HudPipeline,
    gui: GuiPipeline,
    gui_textures: GuiTextures,
}

impl Renderer {
    pub fn new(
        window: Arc<Window>,
        materials: &ResolvedBlockMaterials,
        destroy_atlas: &engine_assets::TextureAtlas,
        environment: &EnvironmentTextures,
        gui_textures: &GuiTextures,
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

        let mut limits = wgpu::Limits::default();
        limits.max_bind_groups = 5;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
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

        let hdr_format = wgpu::TextureFormat::Rgba16Float;
        let colormap_rect = materials.colormap_atlas_rect;
        let lighting = LightingResources::new(&device, &queue, environment);
        let mut post = PostPipeline::new(&device, surface_format, config.width, config.height);
        let sky = SkyPipeline::new(&device, hdr_format, &lighting.env_bind_group_layout);
        post.recreate_bind_group(&device, &depth_view);

        let pipelines = RenderPipelines::new(
            &device,
            &queue,
            hdr_format,
            surface_format,
            &materials.atlas,
            destroy_atlas,
            colormap_rect,
            &lighting,
        );

        let hud = HudPipeline::new(&device, surface_format);
        let gui = GuiPipeline::new(&device, &queue, surface_format, gui_textures);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            pipelines,
            lighting,
            sky,
            post,
            colormap_rect,
            opaque_meshes: Vec::new(),
            cutout_meshes: Vec::new(),
            uploaded_mesh_generation: u64::MAX,
            hud,
            gui,
            gui_textures: gui_textures.clone(),
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
        self.post.resize(&self.device, size.width, size.height);
        self.post.recreate_bind_group(&self.device, &self.depth_view);
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

    pub fn render(
        &mut self,
        scene: &RenderScene,
        hud_label: Option<&str>,
        gui_scale: f32,
        gui: Option<&GuiFrame>,
    ) -> Result<(), SurfaceError> {
        let lighting = scene.lighting;
        let camera = scene.camera;
        let view_proj = camera.view_projection();
        let inv_view_proj = view_proj.inverse();
        let light_view_proj = compute_light_view_proj(lighting.sun_dir, camera.position);

        self.lighting.update(
            &self.queue,
            &lighting,
            camera.position,
            light_view_proj,
        );
        let sky_uniform = self
            .lighting
            .sky_uniform(inv_view_proj, camera.position, &lighting);
        self.sky.update(&self.queue, &sky_uniform);

        self.pipelines.update_scene(
            &self.queue,
            view_proj,
            scene.animation_tick,
            self.colormap_rect,
        );

        self.post.update_uniforms(
            &self.queue,
            &PostGpuUniform {
                fog_color: [
                    lighting.horizon_color.x,
                    lighting.horizon_color.y,
                    lighting.horizon_color.z,
                    1.0,
                ],
                fog_density: 0.0022,
                near: camera.near,
                far: camera.far,
                _align_pad: 0.0,
                _pad: [0.0; 4],
            },
        );

        let menu_open = gui.is_some_and(|frame| !frame.is_empty());
        let scale = gui_scale.max(0.25);
        self.hud.set_text(
            &self.queue,
            hud_label.unwrap_or(""),
            self.config.width,
            self.config.height,
            scale,
            !menu_open,
        );
        if let Some(frame) = gui.filter(|gui| !gui.is_empty()) {
            self.gui
                .set_frame(&self.queue, frame, &self.gui_textures);
        } else {
            self.gui.set_frame(
                &self.queue,
                &GuiFrame::default(),
                &self.gui_textures,
            );
        }

        let particle_mesh = if scene.particles.vertices.is_empty() {
            None
        } else {
            Some(&scene.particles)
        };
        self.pipelines
            .particles
            .sync_mesh(&self.queue, particle_mesh);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        render_passes::record_sky_pass(
            &mut encoder,
            &self.post.hdr_view,
            &self.depth_view,
            &self.sky,
            &self.lighting.env_bind_group,
        );
        render_passes::record_shadow_pass(
            &mut encoder,
            &self.lighting,
            &self.pipelines,
            &self.opaque_meshes,
            &self.cutout_meshes,
        );
        render_passes::record_depth_pass(
            &mut encoder,
            &self.depth_view,
            &self.lighting,
            &self.pipelines,
            &self.opaque_meshes,
            &self.cutout_meshes,
        );
        render_passes::record_opaque_pass(
            &mut encoder,
            &self.post.hdr_view,
            &self.depth_view,
            &self.lighting,
            &self.pipelines,
            &self.opaque_meshes,
        );
        render_passes::record_cutout_pass(
            &mut encoder,
            &self.post.hdr_view,
            &self.depth_view,
            &self.lighting,
            &self.pipelines,
            &self.cutout_meshes,
        );
        render_passes::record_particle_pass(
            &mut encoder,
            &self.post.hdr_view,
            &self.depth_view,
            &self.lighting,
            &self.pipelines,
        );
        render_passes::record_post_pass(&mut encoder, &view, &self.post);

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
                &self.lighting.uniform_bind_group,
                &self.lighting.shadow_bind_group,
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

        if gui.is_some_and(|frame| !frame.is_empty()) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gui_pass"),
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
            self.gui.draw(&mut pass);
        }

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        output.present();
        Ok(())
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
