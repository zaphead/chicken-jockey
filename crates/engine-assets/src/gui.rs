use std::path::{Path, PathBuf};

use crate::atlas::{TextureAtlas, UvRect};

#[derive(Debug, Clone, Copy)]
pub struct NineSliceSprite {
    pub uv: UvRect,
    pub width: u32,
    pub height: u32,
    pub border_left: u32,
    pub border_top: u32,
    pub border_right: u32,
    pub border_bottom: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct GuiSprite {
    pub uv: UvRect,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct GuiTextures {
    pub atlas: TextureAtlas,
    pub solid_uv: UvRect,
    pub button: NineSliceSprite,
    pub button_highlighted: NineSliceSprite,
    pub panel: NineSliceSprite,
    pub hotbar: GuiSprite,
    pub hotbar_selection: GuiSprite,
    pub inventory: GuiSprite,
    pub slot: GuiSprite,
}

pub fn gui_asset_path(manifest_dir: &str) -> PathBuf {
    crate::textures_asset_path(manifest_dir).join("gui")
}

pub fn load_gui_textures(manifest_dir: &str) -> GuiTextures {
    let dir = gui_asset_path(manifest_dir);
    match pack_gui(&dir) {
        Ok(textures) => textures,
        Err(error) => {
            log::warn!(
                "gui atlas pack failed ({}): using fallback — {error}",
                dir.display()
            );
            fallback_gui()
        }
    }
}

fn load_rgba(path: &Path) -> Result<image::RgbaImage, String> {
    image::open(path)
        .map(|image| image.into_rgba8())
        .map_err(|error| format!("load {}: {error}", path.display()))
}

fn pack_gui(dir: &Path) -> Result<GuiTextures, String> {
    let button = load_rgba(&dir.join("button.png"))?;
    let button_highlighted = load_rgba(&dir.join("button_highlighted.png"))?;
    let panel = load_rgba(&dir.join("panel.png"))?;
    let hotbar = load_rgba(&dir.join("hotbar.png"))?;
    let hotbar_selection = load_rgba(&dir.join("hotbar_selection.png"))?;
    let inventory = load_rgba(&dir.join("inventory.png"))?;
    let slot = load_rgba(&dir.join("slot.png"))?;

    let pad = 2u32;
    let row_w = button
        .width()
        .max(button_highlighted.width())
        .max(panel.width())
        .max(hotbar.width())
        .max(hotbar_selection.width())
        .max(inventory.width())
        .max(slot.width());
    let width = row_w + pad * 2;
    let height = button.height()
        + button_highlighted.height()
        + panel.height()
        + hotbar.height()
        + hotbar_selection.height()
        + inventory.height()
        + slot.height()
        + pad * 8;

    let mut atlas = image::RgbaImage::new(width, height);
    atlas.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
    image::imageops::overlay(&mut atlas, &button, pad as i64, pad as i64);
    image::imageops::overlay(
        &mut atlas,
        &button_highlighted,
        pad as i64,
        (pad + button.height() + pad) as i64,
    );
    image::imageops::overlay(
        &mut atlas,
        &panel,
        pad as i64,
        (pad * 2 + button.height() + button_highlighted.height()) as i64,
    );
    let hotbar_y = pad * 3 + button.height() + button_highlighted.height() + panel.height();
    image::imageops::overlay(&mut atlas, &hotbar, pad as i64, hotbar_y as i64);
    let selection_y = hotbar_y + hotbar.height() + pad;
    image::imageops::overlay(
        &mut atlas,
        &hotbar_selection,
        pad as i64,
        selection_y as i64,
    );
    let inventory_y = selection_y + hotbar_selection.height() + pad;
    image::imageops::overlay(&mut atlas, &inventory, pad as i64, inventory_y as i64);
    let slot_y = inventory_y + inventory.height() + pad;
    image::imageops::overlay(&mut atlas, &slot, pad as i64, slot_y as i64);

    let atlas_w = atlas.width();
    let atlas_h = atlas.height();
    let pixels = atlas.into_raw();

    let uv_for = |x: u32, y: u32, w: u32, h: u32| -> UvRect {
        let inset_u = 0.5 / atlas_w as f32;
        let inset_v = 0.5 / atlas_h as f32;
        UvRect {
            min: [
                x as f32 / atlas_w as f32 + inset_u,
                y as f32 / atlas_h as f32 + inset_v,
            ],
            max: [
                (x + w) as f32 / atlas_w as f32 - inset_u,
                (y + h) as f32 / atlas_h as f32 - inset_v,
            ],
        }
    };

    let button_y = pad;
    let highlighted_y = pad + button.height() + pad;
    let panel_y = highlighted_y + button_highlighted.height() + pad;

    Ok(GuiTextures {
        atlas: TextureAtlas {
            tile_size: 1,
            width: atlas_w,
            height: atlas_h,
            pixels,
        },
        solid_uv: uv_for(0, 0, 1, 1),
        button: NineSliceSprite {
            uv: uv_for(pad, button_y, button.width(), button.height()),
            width: button.width(),
            height: button.height(),
            border_left: 20,
            border_top: 4,
            border_right: 20,
            border_bottom: 4,
        },
        button_highlighted: NineSliceSprite {
            uv: uv_for(pad, highlighted_y, button_highlighted.width(), button_highlighted.height()),
            width: button_highlighted.width(),
            height: button_highlighted.height(),
            border_left: 20,
            border_top: 4,
            border_right: 20,
            border_bottom: 4,
        },
        panel: NineSliceSprite {
            uv: uv_for(pad, panel_y, panel.width(), panel.height()),
            width: panel.width(),
            height: panel.height(),
            border_left: 6,
            border_top: 6,
            border_right: 6,
            border_bottom: 6,
        },
        hotbar: GuiSprite {
            uv: uv_for(pad, hotbar_y, hotbar.width(), hotbar.height()),
            width: hotbar.width(),
            height: hotbar.height(),
        },
        hotbar_selection: GuiSprite {
            uv: uv_for(pad, selection_y, hotbar_selection.width(), hotbar_selection.height()),
            width: hotbar_selection.width(),
            height: hotbar_selection.height(),
        },
        inventory: GuiSprite {
            uv: uv_for(pad, inventory_y, inventory.width(), inventory.height()),
            width: inventory.width(),
            height: inventory.height(),
        },
        slot: GuiSprite {
            uv: uv_for(pad, slot_y, slot.width(), slot.height()),
            width: slot.width(),
            height: slot.height(),
        },
    })
}

fn fallback_gui() -> GuiTextures {
    let width = 4u32;
    let height = 4u32;
    let pixels = vec![255u8, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255];
    let uv = UvRect {
        min: [0.0, 0.0],
        max: [1.0, 1.0],
    };
    let slice = NineSliceSprite {
        uv,
        width: 4,
        height: 4,
        border_left: 1,
        border_top: 1,
        border_right: 1,
        border_bottom: 1,
    };
    GuiTextures {
        atlas: TextureAtlas {
            tile_size: 1,
            width,
            height,
            pixels,
        },
        solid_uv: uv,
        button: slice,
        button_highlighted: slice,
        panel: slice,
        hotbar: GuiSprite {
            uv,
            width: 4,
            height: 4,
        },
        hotbar_selection: GuiSprite {
            uv,
            width: 4,
            height: 4,
        },
        inventory: GuiSprite {
            uv,
            width: 4,
            height: 4,
        },
        slot: GuiSprite {
            uv,
            width: 4,
            height: 4,
        },
    }
}
