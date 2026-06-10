use std::sync::Arc;

use crossbeam_channel::bounded;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::mesh::SolidMesh;
use crate::pipeline::RenderPipeline;
use crate::render_submit::{RenderSubmitThread, RenderSubmitWork};
use crate::world_mesh::RenderScene;

pub struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    config: wgpu::SurfaceConfiguration,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    submit_thread: RenderSubmitThread,
}

impl Renderer {
    /// Synchronous renderer setup for use from `ApplicationHandler::resumed`.
    pub fn new(window: Arc<Window>) -> Self {
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

        let device = Arc::new(device);
        let queue = Arc::new(queue);

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

        let pipeline = RenderPipeline::new(&device, surface_format);
        let depth_pipeline = pipeline.depth_pipeline.clone();
        let submit_thread =
            RenderSubmitThread::spawn(device.clone(), queue.clone(), pipeline, depth_pipeline);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            submit_thread,
        }
    }

    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
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

    pub fn upload_meshes(&mut self, _meshes: &[SolidMesh]) {}

    pub fn render(&mut self, scene: &RenderScene) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let color_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut meshes = scene.chunk_meshes.clone();
        for (offset, mesh) in &scene.entity_meshes {
            let mut translated = mesh.clone();
            for vertex in &mut translated.vertices {
                let position = glam::Vec3::from_array(vertex.position) + *offset;
                vertex.position = position.to_array();
            }
            meshes.push(translated);
        }

        let (done_tx, done_rx) = bounded(1);
        self.submit_thread.submit(RenderSubmitWork {
            color_view,
            depth_view: self.depth_view.clone(),
            scene: scene.clone(),
            meshes,
            done: done_tx,
        });
        let _ = done_rx.recv();

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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
