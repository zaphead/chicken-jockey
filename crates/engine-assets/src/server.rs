use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{Receiver, TryRecvError};

use crate::blocks::{load_block_registry, BlockRegistry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadState {
    Loading,
    Ready,
    Failed,
}

#[derive(Clone)]
pub struct Handle<T> {
    state: Arc<std::sync::Mutex<HandleInner<T>>>,
}

struct HandleInner<T> {
    value: Option<T>,
    state: LoadState,
}

impl<T> Handle<T> {
    pub fn state(&self) -> LoadState {
        self.state
            .lock()
            .expect("handle lock")
            .state
    }

    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.state
            .lock()
            .expect("handle lock")
            .value
            .clone()
    }
}

pub struct AssetServer {
    blocks: Handle<BlockRegistry>,
    inbox: Option<Receiver<Result<BlockRegistry, String>>>,
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            blocks: Handle {
                state: Arc::new(std::sync::Mutex::new(HandleInner {
                    value: None,
                    state: LoadState::Loading,
                })),
            },
            inbox: None,
        }
    }
}

impl AssetServer {
    pub fn load_blocks(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        let (tx, rx) = crossbeam_channel::unbounded();
        self.inbox = Some(rx);
        thread::Builder::new()
            .name("asset-io".into())
            .spawn(move || {
                let registry = load_block_registry(&path);
                let _ = tx.send(Ok(registry));
            })
            .expect("spawn asset io thread");
    }

    pub fn insert_blocks(&mut self, registry: BlockRegistry) {
        let mut inner = self.blocks.state.lock().expect("handle lock");
        inner.value = Some(registry);
        inner.state = LoadState::Ready;
        self.inbox = None;
    }

    pub fn poll(&mut self) {
        let Some(rx) = &self.inbox else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(registry)) => {
                let mut inner = self.blocks.state.lock().expect("handle lock");
                inner.value = Some(registry);
                inner.state = LoadState::Ready;
                self.inbox = None;
            }
            Ok(Err(_)) => {
                let mut inner = self.blocks.state.lock().expect("handle lock");
                inner.state = LoadState::Failed;
                self.inbox = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                let mut inner = self.blocks.state.lock().expect("handle lock");
                inner.state = LoadState::Failed;
                self.inbox = None;
            }
        }
    }

    pub fn blocks(&self) -> Option<BlockRegistry> {
        self.blocks.get()
    }

    pub fn blocks_handle(&self) -> Handle<BlockRegistry> {
        self.blocks.clone()
    }

    pub fn blocks_ready(&self) -> bool {
        self.blocks.state() == LoadState::Ready
    }
}

pub fn blocks_asset_path(manifest_dir: &str) -> PathBuf {
    PathBuf::from(manifest_dir)
        .join("..")
        .join("assets")
        .join("blocks")
}
