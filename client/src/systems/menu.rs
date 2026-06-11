use engine_core::SystemContext;
use engine_render::{GuiButton, GuiFrame, GuiLabel, GuiPanel, GuiRect, RenderSurfaceInfo, RenderWorld};
use game::{DayNightCycle, DEFAULT_DAY_LENGTH_SECS};

use engine_render::screen_text::{widget_centered_x, widget_centered_y};

use crate::systems::input::PendingWinitInput;

const BUTTON_W: f32 = 200.0;
const BUTTON_H: f32 = 20.0;
const BUTTON_GAP: f32 = 8.0;
const PANEL_PAD: f32 = 12.0;

const DAY_LENGTH_PRESETS: [f32; 5] = [60.0, 300.0, 600.0, 1200.0, 3600.0];
const GUI_SCALE_PRESETS: [f32; 5] = [2.0, 4.0, 6.0, 8.0, 10.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PauseScreen {
    #[default]
    Closed,
    Main,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuHit {
    Resume,
    Settings,
    Back,
    DayLengthDown,
    DayLengthUp,
    GuiScaleDown,
    GuiScaleUp,
}

#[derive(Debug, Default)]
pub struct PauseMenu {
    pub screen: PauseScreen,
}

#[derive(Debug, Clone)]
pub struct ClientSettings {
    pub gui_scale: f32,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self { gui_scale: 4.0 }
    }
}

#[derive(Debug, Default)]
pub struct CursorGrabRequest {
    pub locked: bool,
}

pub fn pause_menu_input_system(ctx: &mut SystemContext<'_>) {
    let (toggle_pause, menu_click, cursor_pos) = ctx
        .resources
        .get::<PendingWinitInput>()
        .map(|pending| {
            (
                pending.0.toggle_pause,
                pending.0.menu_click,
                pending.0.cursor_pos,
            )
        })
        .unwrap_or_default();

    if toggle_pause {
        let menu = ctx.resources.get_mut::<PauseMenu>().expect("pause menu");
        menu.screen = match menu.screen {
            PauseScreen::Closed => PauseScreen::Main,
            PauseScreen::Main => PauseScreen::Closed,
            PauseScreen::Settings => PauseScreen::Main,
        };
    }

    let open = menu_open(ctx);
    if open && menu_click {
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
        let day_length = ctx
            .resources
            .get::<DayNightCycle>()
            .map(|cycle| cycle.day_length_secs)
            .unwrap_or(DEFAULT_DAY_LENGTH_SECS);
        let menu = ctx.resources.get::<PauseMenu>().expect("pause menu");
        let layout = build_layout(menu, &settings, day_length, surface, cursor_pos);
        if let Some(hit) = layout.hit_at(cursor_pos.x, cursor_pos.y) {
            apply_hit(ctx, hit);
        }
    }

    if let Some(grab) = ctx.resources.get_mut::<CursorGrabRequest>() {
        grab.locked = !open;
    }
}

pub fn extract_pause_gui_system(ctx: &mut SystemContext<'_>) {
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
    let day_length = ctx
        .resources
        .get::<DayNightCycle>()
        .map(|cycle| cycle.day_length_secs)
        .unwrap_or(DEFAULT_DAY_LENGTH_SECS);
    let menu_screen = ctx
        .resources
        .get::<PauseMenu>()
        .map(|menu| menu.screen)
        .unwrap_or(PauseScreen::Closed);
    let menu = PauseMenu {
        screen: menu_screen,
    };
    let cursor = ctx
        .resources
        .get::<PendingWinitInput>()
        .map(|pending| pending.0.cursor_pos)
        .unwrap_or_default();

    let layout = build_layout(&menu, &settings, day_length, surface, cursor);
    if let Some(world) = ctx.resources.get_mut::<RenderWorld>() {
        world.gui_scale = settings.gui_scale;
        world.gui = layout.frame;
    }
}

struct MenuLayout {
    frame: GuiFrame,
    hits: Vec<(GuiRect, MenuHit)>,
}

impl MenuLayout {
    fn hit_at(&self, x: f32, y: f32) -> Option<MenuHit> {
        self.hits
            .iter()
            .rev()
            .find(|(rect, _)| rect.contains(x, y))
            .map(|(_, hit)| *hit)
    }
}

fn menu_open(ctx: &SystemContext<'_>) -> bool {
    ctx.resources
        .get::<PauseMenu>()
        .is_some_and(|menu| menu.screen != PauseScreen::Closed)
}

fn apply_hit(ctx: &mut SystemContext<'_>, hit: MenuHit) {
    match hit {
        MenuHit::Resume => {
            if let Some(menu) = ctx.resources.get_mut::<PauseMenu>() {
                menu.screen = PauseScreen::Closed;
            }
        }
        MenuHit::Settings => {
            if let Some(menu) = ctx.resources.get_mut::<PauseMenu>() {
                menu.screen = PauseScreen::Settings;
            }
        }
        MenuHit::Back => {
            if let Some(menu) = ctx.resources.get_mut::<PauseMenu>() {
                menu.screen = PauseScreen::Main;
            }
        }
        MenuHit::DayLengthDown | MenuHit::DayLengthUp => {
            let current = ctx
                .resources
                .get::<DayNightCycle>()
                .map(|cycle| cycle.day_length_secs)
                .unwrap_or(DEFAULT_DAY_LENGTH_SECS);
            let next = step_preset(current, &DAY_LENGTH_PRESETS, hit == MenuHit::DayLengthUp);
            if let Some(cycle) = ctx.resources.get_mut::<DayNightCycle>() {
                cycle.day_length_secs = next;
            }
        }
        MenuHit::GuiScaleDown | MenuHit::GuiScaleUp => {
            let current = ctx
                .resources
                .get::<ClientSettings>()
                .map(|settings| settings.gui_scale)
                .unwrap_or(4.0);
            let next = step_preset(current, &GUI_SCALE_PRESETS, hit == MenuHit::GuiScaleUp);
            if let Some(settings) = ctx.resources.get_mut::<ClientSettings>() {
                settings.gui_scale = next;
            }
        }
    }
}

fn step_preset(current: f32, presets: &[f32], up: bool) -> f32 {
    let index = presets
        .iter()
        .position(|value| (*value - current).abs() < 0.01)
        .unwrap_or_else(|| {
            presets
                .iter()
                .position(|value| *value >= current)
                .unwrap_or(presets.len().saturating_sub(1))
        });
    if up {
        presets[index.saturating_add(1).min(presets.len() - 1)]
    } else {
        presets[index.saturating_sub(1)]
    }
}

fn build_layout(
    menu: &PauseMenu,
    settings: &ClientSettings,
    day_length_secs: f32,
    surface: RenderSurfaceInfo,
    cursor: glam::Vec2,
) -> MenuLayout {
    let width = surface.width.max(1);
    let height = surface.height.max(1);
    if menu.screen == PauseScreen::Closed {
        return MenuLayout {
            frame: GuiFrame::default(),
            hits: Vec::new(),
        };
    }

    let scale = settings.gui_scale;
    let sw = width as f32;
    let sh = height as f32;
    let btn_w = BUTTON_W * scale;
    let btn_h = BUTTON_H * scale;
    let gap = BUTTON_GAP * scale;
    let pad = PANEL_PAD * scale;

    let mut frame = GuiFrame {
        width,
        height,
        scale,
        dim_background: true,
        ..Default::default()
    };
    let mut hits = Vec::new();

    match menu.screen {
        PauseScreen::Closed => {}
        PauseScreen::Main => {
            let panel_h = pad * 2.0 + btn_h * 2.0 + gap;
            let panel_w = btn_w + pad * 2.0;
            let panel_x = (sw - panel_w) * 0.5;
            let panel_y = (sh - panel_h) * 0.5;
            frame.panels.push(GuiPanel {
                rect: GuiRect {
                    x: panel_x,
                    y: panel_y,
                    w: panel_w,
                    h: panel_h,
                },
            });

            let bx = panel_x + pad;
            let by_resume = panel_y + pad;
            let by_settings = by_resume + btn_h + gap;
            push_button(
                &mut frame,
                &mut hits,
                bx,
                by_resume,
                btn_w,
                btn_h,
                "BACK TO GAME",
                MenuHit::Resume,
                cursor,
            );
            push_button(
                &mut frame,
                &mut hits,
                bx,
                by_settings,
                btn_w,
                btn_h,
                "SETTINGS",
                MenuHit::Settings,
                cursor,
            );
        }
        PauseScreen::Settings => {
            let row_h = btn_h;
            let panel_h = pad * 2.0 + row_h * 3.0 + gap * 2.0;
            let panel_w = btn_w + pad * 2.0;
            let panel_x = (sw - panel_w) * 0.5;
            let panel_y = (sh - panel_h) * 0.5;
            frame.panels.push(GuiPanel {
                rect: GuiRect {
                    x: panel_x,
                    y: panel_y,
                    w: panel_w,
                    h: panel_h,
                },
            });

            let bx = panel_x + pad;
            let mut row_y = panel_y + pad;
            let day_label = format_day_length(day_length_secs);
            push_setting_row(
                &mut frame,
                &mut hits,
                bx,
                row_y,
                btn_w,
                row_h,
                gap,
                scale,
                "DAY LENGTH",
                &day_label,
                MenuHit::DayLengthDown,
                MenuHit::DayLengthUp,
                cursor,
            );
            row_y += row_h + gap;
            let scale_label = format!("{:.0}X", settings.gui_scale);
            push_setting_row(
                &mut frame,
                &mut hits,
                bx,
                row_y,
                btn_w,
                row_h,
                gap,
                scale,
                "GUI SIZE",
                &scale_label,
                MenuHit::GuiScaleDown,
                MenuHit::GuiScaleUp,
                cursor,
            );
            row_y += row_h + gap;
            push_button(
                &mut frame,
                &mut hits,
                bx,
                row_y,
                btn_w,
                row_h,
                "BACK",
                MenuHit::Back,
                cursor,
            );
        }
    }

    MenuLayout { frame, hits }
}

fn push_button(
    frame: &mut GuiFrame,
    hits: &mut Vec<(GuiRect, MenuHit)>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
    hit: MenuHit,
    cursor: glam::Vec2,
) {
    let rect = GuiRect { x, y, w, h };
    hits.push((rect, hit));
    frame.buttons.push(GuiButton {
        rect,
        highlighted: rect.contains(cursor.x, cursor.y),
    });
    frame.labels.push(GuiLabel {
        x: widget_centered_x(label, x, w, frame.scale),
        y: widget_centered_y(y, h, frame.scale),
        text: label.to_string(),
    });
}

fn push_setting_row(
    frame: &mut GuiFrame,
    hits: &mut Vec<(GuiRect, MenuHit)>,
    x: f32,
    y: f32,
    btn_w: f32,
    btn_h: f32,
    gap: f32,
    scale: f32,
    title: &str,
    value: &str,
    down: MenuHit,
    up: MenuHit,
    cursor: glam::Vec2,
) {
    let small_w = 20.0 * scale;
    push_button(frame, hits, x, y, small_w, btn_h, "-", down, cursor);
    let value_x = x + small_w + gap;
    let value_w = btn_w - small_w * 2.0 - gap * 2.0;
    let row_label = format!("{title}  {value}");
    frame.labels.push(GuiLabel {
        x: widget_centered_x(&row_label, value_x, value_w, scale),
        y: widget_centered_y(y, btn_h, scale),
        text: row_label,
    });
    push_button(
        frame,
        hits,
        x + btn_w - small_w,
        y,
        small_w,
        btn_h,
        "+",
        up,
        cursor,
    );
}

fn format_day_length(secs: f32) -> String {
    if secs >= 3600.0 && (secs - 3600.0).abs() < 0.01 {
        "1 HOUR".to_string()
    } else if secs >= 60.0 {
        format!("{:.0} MIN", secs / 60.0)
    } else {
        format!("{:.0} SEC", secs)
    }
}
