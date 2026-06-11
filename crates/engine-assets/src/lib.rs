//! Asset handles and synchronous block definition loading.

mod atlas;
mod blocks;
mod environment;
mod tools;
pub mod import;
mod layouts;
mod material;
mod mining_atlas;
mod poll;
mod server;

pub use atlas::{textures_asset_path, tile_uv_rect, TextureAtlas, UvRect, DEFAULT_GRID, DEFAULT_TILE_SIZE};
pub use environment::{environment_asset_path, load_environment_textures, EnvironmentTextures};
pub use blocks::{load_block_registry, BlockDefinition, BlockRegistry};
pub use tools::{
    load_tool_registry, tools_asset_path, ToolClass, ToolDefinition, ToolId, ToolRegistry,
};
pub use import::{import_texture_pack, load_manifest, ImportManifest, ImportReport};
pub use layouts::{
    face_from_normal, face_region, CubeFace, PixelRect, UvLayoutId, ALBEDO_HEIGHT, ALBEDO_WIDTH,
    FACE_SIZE,
};
pub use mining_atlas::{
    load_destroy_stage_atlas, mining_textures_dir, DESTROY_STAGE_COUNT,
};
pub use material::{
    pack_block_materials, AnimIndex, DrawCategory, NeighborMask, ResolvedBlockMaterials,
    ResolvedFace, TintMode,
};
pub use poll::poll_assets_system;
pub use server::{assets_dir, blocks_asset_path, AssetServer, Handle, LoadState};
