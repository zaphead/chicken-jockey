use std::path::Path;

use crate::atlas::{TextureAtlas, DEFAULT_TILE_SIZE};

pub const DESTROY_STAGE_COUNT: u32 = 10;

pub fn mining_textures_dir(manifest_dir: &str) -> std::path::PathBuf {
    crate::textures_asset_path(manifest_dir).join("mining")
}

pub fn load_destroy_stage_atlas(mining_dir: &Path) -> TextureAtlas {
    let tile = DEFAULT_TILE_SIZE;
    let width = tile * DESTROY_STAGE_COUNT;
    let height = tile;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for stage in 0..DESTROY_STAGE_COUNT {
        let path = mining_dir.join(format!("destroy_stage_{stage}.png"));
        let image = image::open(&path).unwrap_or_else(|error| {
            panic!("load destroy stage texture {}: {error}", path.display())
        });
        let rgba = image.to_rgba8();
        assert_eq!(
            rgba.dimensions(),
            (tile, tile),
            "destroy stage {} must be {}x{}",
            stage,
            tile,
            tile
        );
        for y in 0..tile {
            for x in 0..tile {
                let src = rgba.get_pixel(x, y).0;
                let dst_x = stage * tile + x;
                let index = ((y * width + dst_x) * 4) as usize;
                pixels[index..index + 4].copy_from_slice(&src);
            }
        }
    }

    TextureAtlas {
        tile_size: tile,
        width,
        height,
        pixels,
    }
}
