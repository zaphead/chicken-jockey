use glam::Vec2;

#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub move_axis: Vec2,
    pub look_delta: Vec2,
    pub ascend: bool,
    pub descend: bool,
    pub sprint: bool,
    pub jump: bool,
    pub interact: bool,
    pub break_block: bool,
    pub place_block: bool,
    pub cursor_locked: bool,
}

impl InputState {
    pub fn vertical_axis(&self) -> f32 {
        (self.ascend as i32 - self.descend as i32) as f32
    }

    pub fn pressed(&self, action: Action) -> bool {
        match action {
            Action::MoveForward => self.move_axis.y > 0.0,
            Action::MoveBack => self.move_axis.y < 0.0,
            Action::MoveLeft => self.move_axis.x < 0.0,
            Action::MoveRight => self.move_axis.x > 0.0,
            Action::Jump => self.jump,
            Action::Interact => self.interact,
            Action::BreakBlock => self.break_block,
            Action::PlaceBlock => self.place_block,
        }
    }

    pub fn clear_frame_state(&mut self) {
        self.look_delta = Vec2::ZERO;
        self.jump = false;
        self.interact = false;
        self.break_block = false;
        self.place_block = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
    Jump,
    Interact,
    BreakBlock,
    PlaceBlock,
}
