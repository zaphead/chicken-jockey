use engine_assets::UvRect;

#[derive(Debug, Clone, Copy)]
pub struct GuiSpriteInstance {
    pub rect: GuiRect,
    pub uv: UvRect,
}

#[derive(Debug, Clone)]
pub struct GuiFrame {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub dim_background: bool,
    pub panels: Vec<GuiPanel>,
    pub buttons: Vec<GuiButton>,
    pub labels: Vec<GuiLabel>,
    pub sprites: Vec<GuiSpriteInstance>,
}

#[derive(Debug, Clone, Copy)]
pub struct GuiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl GuiRect {
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GuiPanel {
    pub rect: GuiRect,
}

#[derive(Debug, Clone)]
pub struct GuiButton {
    pub rect: GuiRect,
    pub highlighted: bool,
}

#[derive(Debug, Clone)]
pub struct GuiLabel {
    pub x: f32,
    pub y: f32,
    pub text: String,
}

impl Default for GuiFrame {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            scale: 1.0,
            dim_background: false,
            panels: Vec::new(),
            buttons: Vec::new(),
            labels: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

impl GuiFrame {
    pub fn has_menu(&self) -> bool {
        self.dim_background
            || !self.panels.is_empty()
            || !self.buttons.is_empty()
    }

    pub fn needs_gui_pass(&self) -> bool {
        self.has_menu() || !self.sprites.is_empty() || !self.labels.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        !self.needs_gui_pass()
    }
}
