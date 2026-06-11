use engine_assets::{
    item_kind_registry_name, item_name_short_label, BlockRegistry, GuiTextures, ItemStack,
    ToolRegistry,
};
use engine_render::{
    screen_text::{widget_centered_x, widget_centered_y},
    GuiLabel, GuiRect, GuiSpriteInstance,
};

pub fn stack_label(
    stack: ItemStack,
    blocks: Option<&BlockRegistry>,
    tools: Option<&ToolRegistry>,
) -> String {
    let (Some(blocks), Some(tools)) = (blocks, tools) else {
        return "?".to_string();
    };
    item_kind_registry_name(stack.kind, blocks, tools)
        .map(|name| item_name_short_label(&name))
        .unwrap_or_else(|| "?".to_string())
}

pub fn stack_icon_name(
    stack: ItemStack,
    blocks: Option<&BlockRegistry>,
    tools: Option<&ToolRegistry>,
) -> Option<String> {
    let (blocks, tools) = (blocks?, tools?);
    item_kind_registry_name(stack.kind, blocks, tools)
}

pub fn stack_icon_uv(
    textures: &GuiTextures,
    stack: ItemStack,
    blocks: Option<&BlockRegistry>,
    tools: Option<&ToolRegistry>,
) -> Option<engine_assets::UvRect> {
    let name = stack_icon_name(stack, blocks, tools)?;
    textures.item_icons.get(&name).copied()
}

pub fn item_icon_rect(slot: GuiRect, scale: f32) -> GuiRect {
    let margin = 2.0 * scale;
    GuiRect {
        x: slot.x + margin,
        y: slot.y + margin,
        w: (slot.w - margin * 2.0).max(1.0),
        h: (slot.h - margin * 2.0).max(1.0),
    }
}

pub fn append_stack_icon(
    frame: &mut engine_render::GuiFrame,
    textures: &GuiTextures,
    stack: ItemStack,
    slot: GuiRect,
    scale: f32,
    blocks: Option<&BlockRegistry>,
    tools: Option<&ToolRegistry>,
) {
    let Some(uv) = stack_icon_uv(textures, stack, blocks, tools) else {
        let label = stack_label(stack, blocks, tools);
        frame.labels.push(centered_item_label(&label, slot, scale));
        return;
    };
    frame.sprites.push(GuiSpriteInstance {
        rect: item_icon_rect(slot, scale),
        uv,
    });
}

pub fn stack_count_label(count: u16) -> Option<String> {
    (count > 1).then(|| count.to_string())
}

pub fn centered_item_label(label: &str, rect: GuiRect, scale: f32) -> GuiLabel {
    GuiLabel {
        x: widget_centered_x(label, rect.x, rect.w, scale),
        y: widget_centered_y(rect.y, rect.h, scale),
        text: label.to_string(),
    }
}

pub fn stack_count_gui_label(count: u16, rect: GuiRect, scale: f32) -> Option<GuiLabel> {
    let text = stack_count_label(count)?;
    Some(GuiLabel {
        x: rect.x + rect.w - text.len() as f32 * 4.0 * scale,
        y: rect.y + rect.h - 8.0 * scale,
        text,
    })
}
