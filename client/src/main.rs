use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use engine_assets::{
    load_destroy_stage_atlas, mining_textures_dir, EnvironmentTextures, GuiTextures, PlayerSkin,
    ResolvedBlockMaterials,
};
use engine_core::{App, Time, SIM_DT};
use engine_input::{apply_mouse_motion, apply_winit_event, InputState};
use engine_net::NetClient;
use engine_render::{RenderSurfaceInfo, RenderWorld, Renderer};
use game::{register_local_client_systems, register_network_client_systems, NetworkClient};

use client::bootstrap::{bootstrap_client_resources, bootstrap_client_shell};
use client::frame::run_client_frame;
use client::systems::input::PendingWinitInput;
use client::diagnostics::ClientDiagnostics;
use client::systems::net::ClientNet;
use client::systems::menu::CursorGrabRequest;
use client::systems::present::ClientRenderer;
use client::systems::register_client_schedule;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

struct ClientApp {
    window: Option<Arc<Window>>,
    ecs: App,
    input: InputState,
    last_frame: Instant,
    frame: u64,
    diagnostic_mode: bool,
}

impl ClientApp {
    fn new() -> Self {
        let mut ecs = App::new();
        ecs.insert_resource(Time::new(SIM_DT));
        bootstrap_client_shell(&mut ecs);
        bootstrap_client_resources(&mut ecs, env!("CARGO_MANIFEST_DIR"));

        if let Some(addr) = std::env::var("OC_SERVER")
            .ok()
            .and_then(|value| value.parse::<SocketAddr>().ok())
        {
            log::info!("connecting to server at {addr}");
            ecs.insert_resource(NetworkClient);
            ecs.insert_resource(ClientNet(NetClient::connect(addr)));
            ecs.insert_resource(client::systems::pending_inventory::PendingInventoryActions::default());
            register_network_client_systems(&mut ecs);
        } else {
            register_local_client_systems(&mut ecs);
        }

        register_client_schedule(&mut ecs);

        Self {
            window: None,
            ecs,
            input: InputState::default(),
            last_frame: Instant::now(),
            frame: 0,
            diagnostic_mode: std::env::var("OC_DIAGNOSTIC").is_ok(),
        }
    }

    fn tick(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let delta = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        if let Some(pending) = self.ecs.resource_mut::<PendingWinitInput>() {
            pending.0 = self.input.clone();
        }

        run_client_frame(&mut self.ecs, delta);
        self.apply_cursor_grab();
        self.input.clear_frame_state();
        self.frame += 1;

        if self.diagnostic_mode || self.frame == 1 || self.frame % 60 == 0 {
            let presented = self
                .ecs
                .resource::<RenderWorld>()
                .map(|world| world.meshes().len())
                .unwrap_or(0);
            let mut diag = ClientDiagnostics::sample(
                &self.ecs,
                self.ecs.resource::<ClientRenderer>().is_some(),
                presented,
            );
            diag.frame = self.frame;
            log::info!("oc diag: {}", diag.log_line());
        }

        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            self.last_frame + Duration::from_secs_f32(1.0 / 60.0),
        ));
    }

    fn apply_cursor_grab(&mut self) {
        let locked = self
            .ecs
            .resource::<CursorGrabRequest>()
            .map(|request| request.locked)
            .unwrap_or(true);
        if locked {
            lock_cursor(&self.window, &mut self.input);
        } else {
            unlock_cursor(&self.window, &mut self.input);
        }
    }

    fn try_create_renderer(&mut self, size: PhysicalSize<u32>) {
        if self.ecs.resource::<ClientRenderer>().is_some() || size.width == 0 || size.height == 0 {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        log::info!("creating renderer at {}x{}", size.width, size.height);
        let Some(materials) = self.ecs.resource::<Arc<ResolvedBlockMaterials>>().cloned() else {
            log::error!("renderer init failed: ResolvedBlockMaterials resource missing");
            return;
        };
        let destroy_atlas =
            load_destroy_stage_atlas(&mining_textures_dir(env!("CARGO_MANIFEST_DIR")));
        let environment = self
            .ecs
            .resource::<Arc<EnvironmentTextures>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(engine_assets::load_environment_textures(
                env!("CARGO_MANIFEST_DIR"),
            )));
        let gui = self
            .ecs
            .resource::<Arc<GuiTextures>>()
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(engine_assets::load_gui_textures(env!("CARGO_MANIFEST_DIR")))
            });
        let player_skin = self
            .ecs
            .resource::<Arc<PlayerSkin>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(engine_assets::load_player_skin(env!("CARGO_MANIFEST_DIR"))));
        let renderer = Renderer::new(
            window,
            &materials,
            &destroy_atlas,
            &environment,
            &gui,
            &player_skin,
        );
        self.ecs.insert_resource(ClientRenderer(renderer));
        if let Some(info) = self.ecs.resource_mut::<RenderSurfaceInfo>() {
            info.width = size.width;
            info.height = size.height;
            info.aspect = size.width as f32 / size.height.max(1) as f32;
        }
        log::info!("renderer ready");
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

        window.focus_window();
        window.request_redraw();
        self.window = Some(window);
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
                self.try_create_renderer(*size);
                if let Some(renderer) = self.ecs.resource_mut::<ClientRenderer>() {
                    renderer.0.resize(*size);
                }
                if let Some(info) = self.ecs.resource_mut::<RenderSurfaceInfo>() {
                    info.width = size.width;
                    info.height = size.height;
                    info.aspect = size.width as f32 / size.height.max(1) as f32;
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.tick(event_loop);
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
                let grab = self
                    .ecs
                    .resource::<CursorGrabRequest>()
                    .map(|request| request.locked)
                    .unwrap_or(true);
                if grab {
                    lock_cursor(&self.window, &mut self.input);
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
            apply_mouse_motion(&mut self.input, delta);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(size) = self.window.as_ref().map(|w| w.inner_size()) {
            self.try_create_renderer(size);
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs_f32(1.0 / 60.0),
        ));
    }
}

fn lock_cursor(window: &Option<Arc<Window>>, input: &mut InputState) {
    let Some(window) = window else {
        return;
    };
    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
    window.set_cursor_visible(false);
    input.cursor_locked = true;
}

fn unlock_cursor(window: &Option<Arc<Window>>, input: &mut InputState) {
    let Some(window) = window else {
        return;
    };
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);
    input.cursor_locked = false;
}

fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_title("OpenCraft")
        .with_maximized(true)
        .with_min_inner_size(PhysicalSize::new(640, 480))
        .with_visible(true)
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
