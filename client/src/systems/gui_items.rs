use engine_assets::{ToolId, ToolRegistry};
use engine_render::{
    screen_text::{widget_centered_x, widget_centered_y},
    GuiLabel, GuiRect,
};

pub fn tool_short_label(name: &str) -> String {
    name.split('_')
        .filter_map(|part| part.chars().next())
        .collect::<String>()
        .to_uppercase()
}

pub fn item_label(tool_id: ToolId, tools: Option<&ToolRegistry>) -> String {
    tools
        .and_then(|registry| registry.get(tool_id))
        .map(|tool| tool_short_label(&tool.name))
        .unwrap_or_else(|| "?".to_string())
}

pub fn centered_item_label(
    label: &str,
    rect: GuiRect,
    scale: f32,
) -> GuiLabel {
    GuiLabel {
        x: widget_centered_x(label, rect.x, rect.w, scale),
        y: widget_centered_y(rect.y, rect.h, scale),
        text: label.to_string(),
    }
}
