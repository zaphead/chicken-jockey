//! Asset handles and synchronous block definition loading.

mod blocks;
mod poll;
mod server;

pub use blocks::{load_block_registry, BlockDefinition, BlockRegistry};
pub use poll::poll_assets_system;
pub use server::{blocks_asset_path, AssetServer, Handle, LoadState};
