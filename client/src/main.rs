mod net;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use engine_assets::load_block_registry;
use engine_core::{App, Time};
use engine_input::{apply_winit_event, InputState};
use engine_net::NetClient;
use engine_render::{
    cube_mesh, extract_render_scene, Camera, ChunkMeshCache, Renderer, CHUNK_MESH_LOD_DISTANCE,
};
use engine_world::{BlockChanged, SparseVoxelOctree, WorldMutationQueue};
use game::{
    register_client_systems, LocalPlayer, Player, Renderable, SimulationMode, TerrainGeneration,
    Transform, WorldInitialized, WORLD_RADIUS,
};
use glam::{IVec3, Vec3};
use net::client_net_pre_update;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

struct ClientApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    net: Option<NetClient>,
    ecs: App,
    input: InputState,
    mesh_cache: ChunkMeshCache,
    last_frame: Instant,
    world_mesh_queue: Vec<IVec3>,
    window_centered: bool,
}

impl ClientApp {
    fn new() -> Self {
        let mut ecs = App::new();
        ecs.insert_resource(Time::new(1.0 / 60.0));
        ecs.insert_resource(InputState::default());
        ecs.insert_resource(SparseVoxelOctree::default());
        ecs.insert_resource(WorldMutationQueue::default());
        ecs.insert_resource(WorldInitialized::default());
        ecs.insert_resource(TerrainGeneration::default());
        ecs.insert_resource(LocalPlayer::default());

        let net = std::env::var("CJ_SERVER")
            .ok()
            .and_then(|value| value.parse::<SocketAddr>().ok())
            .map(|addr| {
                log::info!("connecting to server at {addr}");
                ecs.insert_resource(SimulationMode::NetworkClient);
                NetClient::connect(addr)
            });

        if net.is_none() {
            ecs.insert_resource(SimulationMode::Local);
        }

        let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets")
            .join("blocks");
        ecs.insert_resource(load_block_registry(&assets));

        register_client_systems(&mut ecs);

        Self {
            window: None,
            renderer: None,
            net,
            ecs,
            input: InputState::default(),
            mesh_cache: ChunkMeshCache::default(),
            last_frame: Instant::now(),
            world_mesh_queue: Vec::new(),
            window_centered: false,
        }
    }

    fn tick(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;

        if let Some(time) = self.ecs.resource_mut::<Time>() {
            time.advance_variable(delta);
        }
        if let Some(input) = self.ecs.resource_mut::<InputState>() {
            *input = self.input.clone();
        }

        if let Some(net) = &self.net {
            client_net_pre_update(&mut self.ecs, net, &self.input);
        }

        self.ecs.tick_with_render();

        for change in self.ecs.drain_events::<BlockChanged>() {
            self.mesh_cache.mark_dirty_neighbors(change.position);
        }
        self.ecs.end_frame();

        if self.world_mesh_queue.is_empty()
            && self
                .ecs
                .resource::<WorldInitialized>()
                .map(|flag| flag.0)
                .unwrap_or(false)
        {
            self.queue_world_mesh_chunks();
        }
        self.enqueue_world_mesh_batch();

        self.extract_and_render();
        self.input.clear_frame_state();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.last_frame + Duration::from_secs_f32(1.0 / 60.0),
        ));
    }

    fn queue_world_mesh_chunks(&mut self) {
        let chunk_radius = WORLD_RADIUS / engine_world::CHUNK_SIZE + 1;
        for cx in -chunk_radius..chunk_radius {
            for cz in -chunk_radius..chunk_radius {
                for cy in 0..2 {
                    self.world_mesh_queue.push(IVec3::new(cx, cy, cz));
                }
            }
        }
    }

    fn enqueue_world_mesh_batch(&mut self) {
        const BATCH: usize = 16;
        for chunk in self.world_mesh_queue.drain(..self.world_mesh_queue.len().min(BATCH)) {
            self.mesh_cache.mark_dirty(chunk);
        }
    }

    fn try_create_renderer(&mut self, size: PhysicalSize<u32>) {
        if self.renderer.is_some() || size.width == 0 || size.height == 0 {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        log::info!("creating renderer at {}x{}", size.width, size.height);
        self.renderer = Some(Renderer::new(window));
        log::info!("renderer ready");
    }

    fn extract_and_render(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let world = self.ecs.resource::<SparseVoxelOctree>().expect("world");
        let registry = self
            .ecs
            .resource::<engine_assets::BlockRegistry>()
            .expect("registry");

        let camera = player_camera(&self.ecs, renderer.aspect());
        let _rebuilt = self.mesh_cache.rebuild_dirty_near(
            world,
            registry,
            camera.position,
            CHUNK_MESH_LOD_DISTANCE,
        );
        let mut meshes = self.mesh_cache.all_meshes();

        for (_, (transform, renderable)) in self.ecs.world.query::<(&Transform, &Renderable)>().iter()
        {
            meshes.push(translate_mesh(
                cube_mesh(IVec3::ZERO, renderable.size, renderable.color),
                transform.position - Vec3::splat(renderable.size * 0.5),
            ));
        }

        renderer.upload_meshes(&meshes);
        let scene = extract_render_scene(camera, meshes, Vec::new());
        if let Err(error) = renderer.render(&scene) {
            log::warn!("render error: {error:?}");
        }
    }
}

impl ApplicationHandler for ClientApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(window_attributes())
                .expect("create window"),
        );

        let _ = window.request_inner_size(PhysicalSize::new(1280, 720));
        center_window_on_monitor(&window);
        let size = window.inner_size();
        log::info!("window created, inner_size={size:?}, outer_position={:?}", window.outer_position());

        self.window = Some(window.clone());

        window.focus_window();
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if !self.window_centered {
                    if let Some(window) = &self.window {
                        center_window_on_monitor(window);
                        self.window_centered = true;
                    }
                }
                self.try_create_renderer(*size);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(*size);
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if self.renderer.is_some() {
                    self.tick(event_loop);
                }
            }
            WindowEvent::Occluded(false) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(window) = &self.window {
                    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                    window.set_cursor_visible(false);
                    self.input.cursor_locked = true;
                }
            }
            _ => {}
        }

        apply_winit_event(&mut self.input, &event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.input.cursor_locked {
                self.input.look_delta.x += delta.0 as f32;
                self.input.look_delta.y += delta.1 as f32;
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let size = self.window.as_ref().map(|window| window.inner_size());
        if self.renderer.is_none() {
            if let Some(size) = size {
                self.try_create_renderer(size);
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs_f32(1.0 / 60.0),
        ));
    }
}

fn center_window_on_monitor(window: &Window) {
    let monitor = window
        .current_monitor()
        .or_else(|| window.primary_monitor());
    let Some(monitor) = monitor else {
        return;
    };

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let window_size = window.outer_size();
    let x = monitor_pos.x + (monitor_size.width.saturating_sub(window_size.width) as i32) / 2;
    let y = monitor_pos.y + (monitor_size.height.saturating_sub(window_size.height) as i32) / 2;
    window.set_outer_position(PhysicalPosition::new(x, y));
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_title("Chicken Jockey")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_min_inner_size(PhysicalSize::new(640, 480))
        .with_visible(true)
}

fn player_camera(ecs: &App, aspect: f32) -> Camera {
    let mut camera = Camera::default();
    camera.aspect = aspect;

    if let Some((_, (_, transform))) = ecs.world.query::<(&Player, &Transform)>().iter().next() {
        camera.position = transform.position + Vec3::new(0.0, 1.6, 0.0);
        camera.yaw = transform.yaw;
        camera.pitch = transform.pitch;
    }

    camera
}

fn translate_mesh(
    mut mesh: engine_render::SolidMesh,
    offset: Vec3,
) -> engine_render::SolidMesh {
    for vertex in &mut mesh.vertices {
        let position = Vec3::from_array(vertex.position) + offset;
        vertex.position = position.to_array();
    }
    mesh
}

fn main() {
    env_logger::init();

    let event_loop = {
        let mut builder = EventLoop::builder();
        #[cfg(target_os = "macos")]
        {
            builder.with_activation_policy(ActivationPolicy::Regular);
            builder.with_activate_ignoring_other_apps(true);
        }
        builder.build().expect("event loop")
    };

    let mut app = ClientApp::new();
    event_loop.run_app(&mut app).expect("run app");
}
