use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::mesh::SolidMesh;
use crate::renderer::Renderer;
use crate::world_mesh::RenderScene;

pub struct RenderFrame {
    pub scene: RenderScene,
    pub meshes: Vec<SolidMesh>,
}

enum RenderCommand {
    Init(Arc<Window>),
    Resize(PhysicalSize<u32>),
    Frame(RenderFrame),
    Shutdown,
}

pub struct RenderThread {
    tx: Sender<RenderCommand>,
    join: Option<JoinHandle<()>>,
}

impl RenderThread {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        let join = thread::Builder::new()
            .name("render".into())
            .spawn(move || render_loop(rx))
            .expect("spawn render thread");

        Self {
            tx,
            join: Some(join),
        }
    }

    pub fn init(&self, window: Arc<Window>) {
        let _ = self.tx.send(RenderCommand::Init(window));
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        let _ = self.tx.send(RenderCommand::Resize(size));
    }

    pub fn submit(&self, frame: RenderFrame) {
        let _ = self.tx.send(RenderCommand::Frame(frame));
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        let _ = self.tx.send(RenderCommand::Shutdown);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn render_loop(rx: Receiver<RenderCommand>) {
    let mut renderer: Option<Renderer> = None;

    while let Ok(command) = rx.recv() {
        match command {
            RenderCommand::Init(window) => {
                renderer = Some(Renderer::new(window));
            }
            RenderCommand::Resize(size) => {
                if let Some(renderer) = renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            RenderCommand::Frame(frame) => {
                if let Some(renderer) = renderer.as_mut() {
                    renderer.upload_meshes(&frame.meshes);
                    if let Err(error) = renderer.render(&frame.scene) {
                        log::warn!("render error: {error:?}");
                    }
                }
            }
            RenderCommand::Shutdown => break,
        }
    }
}
