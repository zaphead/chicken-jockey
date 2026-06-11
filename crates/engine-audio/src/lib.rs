//! Client-side audio playback (Kira backend).

mod attenuation;
mod categories;
mod clip;
mod engine;
mod spatial;

pub use attenuation::Attenuation;
pub use categories::SoundCategory;
pub use clip::SoundClip;
pub use engine::{AudioEngine, PlayRequest, SoundHandle};
pub use spatial::{Listener, SpatialMix};
