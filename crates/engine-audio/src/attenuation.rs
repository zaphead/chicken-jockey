#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attenuation {
    /// Full volume regardless of listener distance (still panned when a position is set).
    None,
    /// Linear falloff to silence at `max_distance` blocks.
    Linear { max_distance: f32 },
}

impl Default for Attenuation {
    fn default() -> Self {
        Self::Linear {
            max_distance: 16.0,
        }
    }
}
