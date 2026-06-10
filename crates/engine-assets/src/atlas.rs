use std::path::PathBuf;

pub const DEFAULT_TILE_SIZE: u32 = 16;
pub const DEFAULT_GRID: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl UvRect {
    pub const BLACK: Self = Self {
        min: [0.0, 0.0],
        max: [0.0, 0.0],
    };
}

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub tile_size: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn textures_asset_path(manifest_dir: &str) -> PathBuf {
    crate::server::assets_dir(manifest_dir).join("textures")
}

pub fn tile_uv_rect(col: u32, row: u32, tile_size: u32, width: u32, height: u32) -> UvRect {
    let tile = tile_size as f32;
    let w = width as f32;
    let h = height as f32;
    let inset_u = 0.5 / w;
    let inset_v = 0.5 / h;
    let u0 = col as f32 * tile / w + inset_u;
    let u1 = (col as f32 + 1.0) * tile / w - inset_u;
    let v0 = row as f32 * tile / h + inset_v;
    let v1 = (row as f32 + 1.0) * tile / h - inset_v;
    UvRect {
        min: [u0, v0],
        max: [u1, v1],
    }
}
