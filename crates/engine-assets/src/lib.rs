//! Asset handles and synchronous block definition loading.

mod atlas;
mod blocks;
mod layouts;
mod material_map;
mod packed_textures;
mod poll;
mod server;
mod texture_packer;

pub use atlas::{textures_asset_path, tile_uv_rect, TextureAtlas, UvRect, DEFAULT_GRID, DEFAULT_TILE_SIZE};
pub use blocks::{load_block_registry, BlockDefinition, BlockRegistry};
pub use layouts::{
    face_from_normal, face_region, CubeFace, PixelRect, UvLayoutId, ALBEDO_HEIGHT, ALBEDO_WIDTH,
    FACE_SIZE,
};
pub use material_map::{BlockFaceUvs, BlockMaterialMap};
pub use packed_textures::PackedBlockTextures;
pub use poll::poll_assets_system;
pub use server::{assets_dir, blocks_asset_path, AssetServer, Handle, LoadState};
pub use texture_packer::pack_block_textures;
