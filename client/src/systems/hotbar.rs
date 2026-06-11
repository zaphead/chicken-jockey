use engine_assets::{GuiTextures, ToolRegistry};
use engine_render::{GuiFrame, GuiRect, GuiSpriteInstance, RenderSurfaceInfo};
use game::PlayerInventory;

use crate::systems::gui_items::{centered_item_label, item_label};

const HOTBAR_W: f32 = 182.0;
const HOTBAR_H: f32 = 22.0;
const SLOT_SIZE: f32 = 20.0;
const HOTBAR_MARGIN: f32 = 4.0;
const SELECTION_W: f32 = 24.0;
const SELECTION_H: f32 = 23.0;

pub(crate) fn append_hotbar(
    frame: &mut GuiFrame,
    textures: &GuiTextures,
    surface: RenderSurfaceInfo,
    inventory: &PlayerInventory,
    tools: Option<&ToolRegistry>,
) {
    let scale = frame.scale;
    let sw = surface.width.max(1) as f32;
    let sh = surface.height.max(1) as f32;
    let hotbar_w = HOTBAR_W * scale;
    let hotbar_h = HOTBAR_H * scale;
    let x = (sw - hotbar_w) * 0.5;
    let y = sh - hotbar_h - HOTBAR_MARGIN * scale;

    frame.sprites.push(GuiSpriteInstance {
        rect: GuiRect {
            x,
            y,
            w: hotbar_w,
            h: hotbar_h,
        },
        uv: textures.hotbar.uv,
    });
    frame.sprites.push(GuiSpriteInstance {
        rect: hotbar_selection_rect(x, y, scale, inventory.selected_hotbar),
        uv: textures.hotbar_selection.uv,
    });

    let slot = SLOT_SIZE * scale;
    for (index, tool_id) in inventory.slots[..game::HOTBAR_SLOTS]
        .iter()
        .enumerate()
    {
        let Some(tool_id) = tool_id else {
            continue;
        };
        let rect = GuiRect {
            x: x + index as f32 * slot,
            y,
            w: slot,
            h: slot,
        };
        let label = item_label(*tool_id, tools);
        frame.labels.push(centered_item_label(&label, rect, scale));
    }
}

fn hotbar_selection_rect(hotbar_x: f32, hotbar_y: f32, scale: f32, selected: u8) -> GuiRect {
    let slot = SLOT_SIZE * scale;
    let slot_x = hotbar_x + selected as f32 * slot;
    GuiRect {
        x: slot_x - 2.0 * scale,
        y: hotbar_y - 1.0 * scale,
        w: SELECTION_W * scale,
        h: SELECTION_H * scale,
    }
}
