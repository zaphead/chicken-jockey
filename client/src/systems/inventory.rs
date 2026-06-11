use engine_assets::{GuiTextures, ToolId, ToolRegistry};
use engine_core::SystemContext;
use engine_render::{GuiFrame, GuiLabel, GuiRect, GuiSpriteInstance, RenderSurfaceInfo};
use game::{
    local_player_entity, ActivePlayMode, HOTBAR_SLOTS, PlayMode, PlayerInventory,
};

use crate::systems::gui_items::{centered_item_label, item_label};
use crate::systems::input::PendingWinitInput;
use crate::systems::menu::ClientSettings;
use crate::systems::ui_state::{ClientModal, ClientUiState};

const TEX_W: f32 = 256.0;
const TEX_H: f32 = 256.0;
const SLOT: f32 = 18.0;
const SLOT_X: f32 = 8.0;
const MAIN_ROW0_Y: f32 = 88.0;
const HOTBAR_ROW_Y: f32 = 152.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventorySlot {
    Main(usize),
    Hotbar(usize),
}

impl InventorySlot {
    pub fn flat_index(self) -> usize {
        match self {
            Self::Hotbar(index) => PlayerInventory::hotbar_slot_index(index),
            Self::Main(index) => PlayerInventory::main_slot_index(index),
        }
    }
}

pub fn inventory_input_system(ctx: &mut SystemContext<'_>) {
    let survival = ctx
        .resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.0 == PlayMode::Survival);
    if !survival {
        return;
    }

    let (toggle, menu_click, cursor) = ctx
        .resources
        .get::<PendingWinitInput>()
        .map(|pending| {
            (
                pending.0.toggle_inventory,
                pending.0.menu_click,
                pending.0.cursor_pos,
            )
        })
        .unwrap_or_default();

    if toggle {
        let ui = ctx.resources.get_mut::<ClientUiState>().expect("client ui");
        ui.modal = match ui.modal {
            ClientModal::Inventory => ClientModal::None,
            _ => ClientModal::Inventory,
        };
        ui.carried = None;
    }

    let inventory_open = ctx
        .resources
        .get::<ClientUiState>()
        .is_some_and(|ui| matches!(ui.modal, ClientModal::Inventory));
    if inventory_open && menu_click {
        let surface = ctx
            .resources
            .get::<RenderSurfaceInfo>()
            .copied()
            .unwrap_or_default();
        let scale = ctx
            .resources
            .get::<ClientSettings>()
            .map(|settings| settings.gui_scale)
            .unwrap_or(4.0)
            .max(0.25);
        let panel = inventory_panel_rect(surface, scale);
        if let Some(slot) = hit_slot(panel, scale, cursor) {
            handle_slot_click(ctx, slot);
        }
    }
}

pub(crate) fn append_inventory(
    frame: &mut GuiFrame,
    textures: &GuiTextures,
    surface: RenderSurfaceInfo,
    inventory: &PlayerInventory,
    tools: Option<&ToolRegistry>,
    cursor: glam::Vec2,
    carried: Option<ToolId>,
) {
    let scale = frame.scale;
    let panel = inventory_panel_rect(surface, scale);
    let hovered = hit_slot(panel, scale, cursor);

    frame.dim_background = true;
    frame.sprites.push(GuiSpriteInstance {
        rect: panel,
        uv: textures.inventory.uv,
    });

    if let Some(slot) = hovered {
        frame.sprites.push(GuiSpriteInstance {
            rect: slot_rect(panel, scale, slot),
            uv: textures.slot.uv,
        });
    }

    let selection = slot_rect(
        panel,
        scale,
        InventorySlot::Hotbar(inventory.selected_hotbar as usize),
    );
    frame.sprites.push(GuiSpriteInstance {
        rect: selection_rect(selection, scale),
        uv: textures.hotbar_selection.uv,
    });

    append_item_labels(frame, panel, scale, inventory, tools);

    if let Some(tool_id) = carried {
        let label = item_label(tool_id, tools);
        frame.labels.push(GuiLabel {
            x: cursor.x,
            y: cursor.y,
            text: label,
        });
    }
}

fn inventory_panel_rect(surface: RenderSurfaceInfo, scale: f32) -> GuiRect {
    let sw = surface.width.max(1) as f32;
    let sh = surface.height.max(1) as f32;
    let w = TEX_W * scale;
    let h = TEX_H * scale;
    GuiRect {
        x: (sw - w) * 0.5,
        y: (sh - h) * 0.5,
        w,
        h,
    }
}

fn slot_rect(panel: GuiRect, scale: f32, slot: InventorySlot) -> GuiRect {
    let size = SLOT * scale;
    let (tex_x, tex_y) = match slot {
        InventorySlot::Main(index) => {
            let col = (index % 9) as f32;
            let row = (index / 9) as f32;
            (SLOT_X + col * SLOT, MAIN_ROW0_Y + row * SLOT)
        }
        InventorySlot::Hotbar(index) => (SLOT_X + index as f32 * SLOT, HOTBAR_ROW_Y),
    };
    GuiRect {
        x: panel.x + tex_x * scale,
        y: panel.y + tex_y * scale,
        w: size,
        h: size,
    }
}

fn selection_rect(slot: GuiRect, scale: f32) -> GuiRect {
    GuiRect {
        x: slot.x - 2.0 * scale,
        y: slot.y - 1.0 * scale,
        w: 24.0 * scale,
        h: 23.0 * scale,
    }
}

fn append_item_labels(
    frame: &mut GuiFrame,
    panel: GuiRect,
    scale: f32,
    inventory: &PlayerInventory,
    tools: Option<&ToolRegistry>,
) {
    for index in HOTBAR_SLOTS..game::INVENTORY_SLOTS {
        let Some(tool_id) = inventory.slots[index] else {
            continue;
        };
        let main = index - HOTBAR_SLOTS;
        let rect = slot_rect(panel, scale, InventorySlot::Main(main));
        let label = item_label(tool_id, tools);
        frame.labels.push(centered_item_label(&label, rect, scale));
    }
    for (index, tool_id) in inventory.slots[..HOTBAR_SLOTS].iter().enumerate() {
        let Some(tool_id) = tool_id else {
            continue;
        };
        let rect = slot_rect(panel, scale, InventorySlot::Hotbar(index));
        let label = item_label(*tool_id, tools);
        frame.labels.push(centered_item_label(&label, rect, scale));
    }
}

fn hit_slot(panel: GuiRect, scale: f32, cursor: glam::Vec2) -> Option<InventorySlot> {
    for index in (0..game::MAIN_INVENTORY_SLOTS).rev() {
        let rect = slot_rect(panel, scale, InventorySlot::Main(index));
        if rect.contains(cursor.x, cursor.y) {
            return Some(InventorySlot::Main(index));
        }
    }
    for index in (0..HOTBAR_SLOTS).rev() {
        let rect = slot_rect(panel, scale, InventorySlot::Hotbar(index));
        if rect.contains(cursor.x, cursor.y) {
            return Some(InventorySlot::Hotbar(index));
        }
    }
    None
}

fn handle_slot_click(ctx: &mut SystemContext<'_>, slot: InventorySlot) {
    let slot_item = read_slot(ctx, slot);
    let carried = ctx
        .resources
        .get::<ClientUiState>()
        .and_then(|ui| ui.carried);

    let (next_carried, place) = match (carried, slot_item) {
        (None, None) => return,
        (None, Some(item)) => (Some(item), None),
        (Some(item), None) => (None, Some(item)),
        (Some(a), Some(b)) => (Some(b), Some(a)),
    };

    if let Some(item) = place {
        write_slot(ctx, slot, Some(item));
    } else {
        write_slot(ctx, slot, None);
    }

    if let Some(ui) = ctx.resources.get_mut::<ClientUiState>() {
        ui.carried = next_carried;
    }
}

fn read_slot(ctx: &SystemContext<'_>, slot: InventorySlot) -> Option<ToolId> {
    let entity = local_player_entity(ctx)?;
    ctx.world
        .get::<&PlayerInventory>(entity)
        .ok()
        .and_then(|inventory| inventory.slot(slot.flat_index()))
}

fn write_slot(ctx: &mut SystemContext<'_>, slot: InventorySlot, item: Option<ToolId>) {
    let Some(entity) = local_player_entity(ctx) else {
        return;
    };
    if let Ok(mut inventory) = ctx.world.get::<&mut PlayerInventory>(entity) {
        inventory.set_slot(slot.flat_index(), item);
    }
}
