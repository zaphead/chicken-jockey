#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Survival,
    Spectator,
}

impl PlayMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Survival => "Survival",
            Self::Spectator => "Spectator",
        }
    }
}

/// Local client play mode. Absent on server (player systems always run).
use engine_core::SystemContext;

#[derive(Debug, Clone, Copy)]
pub struct ActivePlayMode(pub PlayMode);

impl Default for ActivePlayMode {
    fn default() -> Self {
        Self(PlayMode::Survival)
    }
}

impl ActivePlayMode {
    pub fn allows_player_sim(self) -> bool {
        self.0 == PlayMode::Survival
    }
}

/// Server has no `ActivePlayMode`; survival systems always run there.
pub fn survival_active(ctx: &SystemContext<'_>) -> bool {
    ctx.resources
        .get::<ActivePlayMode>()
        .is_none_or(|mode| mode.allows_player_sim())
}
