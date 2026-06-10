use std::path::Path;

use image::RgbaImage;

use crate::atlas::{tile_uv_rect, TextureAtlas, DEFAULT_GRID, DEFAULT_TILE_SIZE};
use crate::blocks::BlockRegistry;
use crate::layouts::{
    face_region, CubeFace, UvLayoutId, ALBEDO_HEIGHT, ALBEDO_WIDTH, FACE_SIZE,
};
use crate::material_map::{BlockFaceUvs, BlockMaterialMap};
use crate::packed_textures::PackedBlockTextures;

pub fn pack_block_textures(
    textures_dir: &Path,
    registry: &BlockRegistry,
) -> Result<PackedBlockTextures, String> {
    let tile_size = DEFAULT_TILE_SIZE;
    let grid = DEFAULT_GRID;
    let atlas_size = tile_size * grid;
    let mut pixels = vec![0u8; (atlas_size * atlas_size * 4) as usize];
    let mut next_slot = 0u32;

    let fallback_slot = grid * grid - 1;
    fill_black_tile(&mut pixels, atlas_size, fallback_slot, tile_size);
    let fallback_uv = tile_uv_rect(
        fallback_slot % grid,
        fallback_slot / grid,
        tile_size,
        atlas_size,
        atlas_size,
    );
    let mut material_map = BlockMaterialMap::new(fallback_uv);

    for definition in registry.definitions() {
        if !definition.solid {
            continue;
        }

        let material_path = definition.material_path();
        let albedo_path = textures_dir.join(&material_path).join("albedo.png");

        match definition.layout {
            UvLayoutId::CubeV1 => {
                if !albedo_path.is_file() {
                    return Err(format!(
                        "solid block '{}' missing albedo at {}",
                        definition.name,
                        albedo_path.display()
                    ));
                }
                let faces = pack_cube_v1_albedo(
                    &mut pixels,
                    atlas_size,
                    tile_size,
                    grid,
                    &mut next_slot,
                    &albedo_path,
                )?;
                material_map.insert(definition.id, faces);
            }
        }
    }

    Ok(PackedBlockTextures {
        atlas: TextureAtlas {
            tile_size,
            width: atlas_size,
            height: atlas_size,
            pixels,
        },
        materials: material_map,
    })
}

fn pack_cube_v1_albedo(
    pixels: &mut [u8],
    atlas_size: u32,
    tile_size: u32,
    grid: u32,
    next_slot: &mut u32,
    albedo_path: &Path,
) -> Result<BlockFaceUvs, String> {
    let image = image::open(albedo_path)
        .map_err(|error| format!("read {}: {error}", albedo_path.display()))?
        .into_rgba8();

    if image.width() != ALBEDO_WIDTH || image.height() != ALBEDO_HEIGHT {
        return Err(format!(
            "albedo {} must be {ALBEDO_WIDTH}x{ALBEDO_HEIGHT}, got {}x{}",
            albedo_path.display(),
            image.width(),
            image.height()
        ));
    }

    let mut faces = BlockFaceUvs::default();
    for face in [
        CubeFace::Top,
        CubeFace::Bottom,
        CubeFace::Left,
        CubeFace::Front,
        CubeFace::Right,
        CubeFace::Back,
    ] {
        if *next_slot >= grid * grid - 1 {
            return Err("block texture atlas is full".into());
        }
        let slot = *next_slot;
        *next_slot += 1;
        let col = slot % grid;
        let row = slot / grid;
        blit_face(pixels, atlas_size, tile_size, col, row, &image, face_region(face));
        faces.set(
            face,
            tile_uv_rect(col, row, tile_size, atlas_size, atlas_size),
        );
    }

    Ok(faces)
}

fn blit_face(
    atlas: &mut [u8],
    atlas_size: u32,
    tile_size: u32,
    col: u32,
    row: u32,
    source: &RgbaImage,
    region: crate::layouts::PixelRect,
) {
    let dst_x = col * tile_size;
    let dst_y = row * tile_size;
    for py in 0..FACE_SIZE.min(tile_size) {
        for px in 0..FACE_SIZE.min(tile_size) {
            let pixel = source.get_pixel(region.x + px, region.y + py);
            let atlas_x = dst_x + px;
            let atlas_y = dst_y + py;
            let idx = ((atlas_y * atlas_size + atlas_x) * 4) as usize;
            atlas[idx..idx + 4].copy_from_slice(&pixel.0);
        }
    }
}

fn fill_black_tile(pixels: &mut [u8], atlas_size: u32, slot: u32, tile_size: u32) {
    let grid = atlas_size / tile_size;
    let col = slot % grid;
    let row = slot / grid;
    let dst_x = col * tile_size;
    let dst_y = row * tile_size;
    for py in 0..tile_size {
        for px in 0..tile_size {
            let idx = (((dst_y + py) * atlas_size + dst_x + px) * 4) as usize;
            pixels[idx..idx + 4].copy_from_slice(&[0, 0, 0, 255]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::load_block_registry;
    use crate::server::blocks_asset_path;

    #[test]
    fn packs_grass_with_distinct_top_and_side_uvs() {
        let client = concat!(env!("CARGO_MANIFEST_DIR"), "/../../client");
        let registry = load_block_registry(&blocks_asset_path(client));
        let textures = crate::atlas::textures_asset_path(client);
        let packed = pack_block_textures(&textures, &registry).expect("pack");
        let grass = registry.id_by_name("grass").expect("grass");
        let top = packed.materials.face_uv(grass, [0.0, 0.0, 1.0]);
        let side = packed.materials.face_uv(grass, [0.0, 1.0, 0.0]);
        assert_ne!(top, side);
        assert_ne!(top, packed.materials.fallback());
    }
}
