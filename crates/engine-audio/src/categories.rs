#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Blocks,
    Player,
    Music,
}

impl SoundCategory {
    pub const ALL: [SoundCategory; 4] = [Self::Master, Self::Blocks, Self::Player, Self::Music];

    pub fn voice_cap(self) -> usize {
        match self {
            Self::Master => usize::MAX,
            Self::Blocks => 32,
            Self::Player => 16,
            Self::Music => 1,
        }
    }
}
