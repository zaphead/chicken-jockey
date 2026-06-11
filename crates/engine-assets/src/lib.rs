//! Asset handles and synchronous block definition loading.

mod atlas;
mod blocks;
mod items;
mod environment;
mod gui;
mod tools;
pub mod import;
mod layouts;
mod material;
mod mining_atlas;
mod poll;
mod server;
mod skin;

pub use atlas::{textures_asset_path, tile_uv_rect, TextureAtlas, UvRect, DEFAULT_GRID, DEFAULT_TILE_SIZE};
pub use environment::{environment_asset_path, load_environment_textures, EnvironmentTextures};
pub use gui::{gui_asset_path, load_gui_textures, GuiSprite, GuiTextures, NineSliceSprite};
pub use blocks::{load_block_registry, BlockDefinition, BlockRegistry, DropSpec};
pub use items::{
    clamp_stack_count, item_kind_registry_name, item_name_short_label, max_stack, stacks_merge,
    ItemKind, ItemStack, BLOCK_MAX_STACK, TOOL_MAX_STACK,
};
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
pub use skin::{load_player_skin, player_skin_path, PlayerSkin};
pub use server::{assets_dir, blocks_asset_path, runtime_asset_root, AssetServer, Handle, LoadState};
