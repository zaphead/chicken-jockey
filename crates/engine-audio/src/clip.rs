use std::fmt;
use std::io::Cursor;
use std::sync::Arc;

use kira::sound::static_sound::StaticSoundData;

#[derive(Clone)]
pub struct SoundClip {
    data: Arc<StaticSoundData>,
}

impl fmt::Debug for SoundClip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoundClip")
            .field("frames", &self.data.num_frames())
            .finish()
    }
}

impl SoundClip {
    pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = Cursor::new(bytes.to_vec());
        let data = StaticSoundData::from_cursor(cursor)
            .map_err(|error| format!("decode ogg: {error}"))?;
        Ok(Self {
            data: Arc::new(data),
        })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let data = StaticSoundData::from_file(path)
            .map_err(|error| format!("load {}: {error}", path.display()))?;
        Ok(Self {
            data: Arc::new(data),
        })
    }

    pub(crate) fn data(&self) -> &StaticSoundData {
        &self.data
    }
}
