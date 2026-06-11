use std::sync::Arc;

use engine_assets::{BlockRegistry, GuiTextures, ToolRegistry};
use engine_core::SystemContext;
use engine_render::{GuiFrame, RenderSurfaceInfo, RenderWorld};
use game::{local_player_entity, ActivePlayMode, PlayMode, PlayerInventory};

use crate::systems::hotbar::append_hotbar;
use crate::systems::input::PendingWinitInput;
use crate::systems::inventory::append_inventory;
use crate::systems::menu::{build_pause_layout, ClientSettings, PauseScreen};
use crate::systems::ui_state::{ClientModal, ClientUiState};

pub fn extract_client_gui_system(ctx: &mut SystemContext<'_>) {
    let surface = ctx
        .resources
        .get::<RenderSurfaceInfo>()
        .copied()
        .unwrap_or_default();
    let settings = ctx
        .resources
        .get::<ClientSettings>()
        .cloned()
        .unwrap_or_default();
    let scale = settings.gui_scale.max(0.25);
    let ui = ctx
        .resources
        .get::<ClientUiState>()
        .cloned()
        .unwrap_or_default();
    let textures = ctx
        .resources
        .get::<Arc<GuiTextures>>()
        .cloned()
        .expect("gui textures");
    let cursor = ctx
        .resources
        .get::<PendingWinitInput>()
        .map(|pending| pending.0.cursor_pos)
        .unwrap_or_default();
    let survival = ctx
        .resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.0 == PlayMode::Survival);
    let inventory = local_player_entity(ctx).and_then(|entity| {
        ctx.world
            .get::<&PlayerInventory>(entity)
            .ok()
            .map(|inventory| *inventory)
    });
    let tools = ctx.resources.get::<ToolRegistry>();
    let blocks = ctx.resources.get::<BlockRegistry>();

    let mut frame = GuiFrame {
        width: surface.width.max(1),
        height: surface.height.max(1),
        scale,
        ..GuiFrame::default()
    };

    if let ClientModal::Pause(screen) = ui.modal {
        if screen != PauseScreen::Closed {
            let pause = build_pause_layout(ctx, screen, &settings, surface, cursor);
            frame.dim_background = pause.dim_background;
            frame.panels = pause.panels;
            frame.buttons = pause.buttons;
            frame.labels = pause.labels;
        }
    } else if matches!(ui.modal, ClientModal::Inventory) {
        if let Some(inventory) = inventory.as_ref() {
            append_inventory(
                &mut frame,
                &textures,
                surface,
                inventory,
                blocks,
                tools,
                cursor,
                ui.carried,
            );
        }
    }

    if survival && matches!(ui.modal, ClientModal::None) {
        if let Some(inventory) = inventory.as_ref() {
            append_hotbar(&mut frame, &textures, surface, inventory, blocks, tools);
        }
    }

    if let Some(world) = ctx.resources.get_mut::<RenderWorld>() {
        world.gui_scale = scale;
        world.gui = frame;
    }
}
