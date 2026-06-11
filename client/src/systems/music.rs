use std::collections::HashMap;
use std::path::{Path, PathBuf};

use engine_assets::{MusicManifest, MusicTrack};
use engine_audio::SoundClip;
use engine_core::SystemContext;
use game::{world_time_crossed_anchors, DayNightCycle};
use rand::seq::SliceRandom;

use super::audio::ClientAudio;

pub const MUSIC_COOLDOWN_SECS: f32 = 600.0;

pub struct MusicBank {
    music_dir: PathBuf,
    tracks: Vec<MusicTrack>,
    clips: HashMap<String, SoundClip>,
}

impl MusicBank {
    pub fn from_manifest(manifest: &MusicManifest, music_dir: &Path) -> Result<Self, String> {
        for track in &manifest.tracks {
            let path = music_dir.join(&track.file);
            if !path.is_file() {
                return Err(format!("missing music file {}", path.display()));
            }
        }
        Ok(Self {
            music_dir: music_dir.to_path_buf(),
            tracks: manifest.tracks.clone(),
            clips: HashMap::new(),
        })
    }

    fn pick_random(&self, exclude: Option<&str>) -> Option<&MusicTrack> {
        if self.tracks.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let mut candidates: Vec<&MusicTrack> = self
            .tracks
            .iter()
            .filter(|track| exclude != Some(track.id.as_str()))
            .collect();
        if candidates.is_empty() {
            candidates = self.tracks.iter().collect();
        }
        candidates.choose(&mut rng).copied()
    }

    fn clip(&mut self, track_id: &str) -> Option<&SoundClip> {
        if !self.clips.contains_key(track_id) {
            let track = self.tracks.iter().find(|t| t.id == track_id)?;
            let path = self.music_dir.join(&track.file);
            let clip = SoundClip::from_file(&path).ok()?;
            self.clips.insert(track_id.to_string(), clip);
        }
        self.clips.get(track_id)
    }
}

#[derive(Debug, Default)]
pub struct MusicPlaybackState {
    pub last_world_time: f32,
    pub last_track_id: Option<String>,
    pub cooldown_secs: f32,
    pub seeded: bool,
    pub was_playing: bool,
}

pub fn client_music_system(ctx: &mut SystemContext<'_>) {
    let Some(time) = ctx.resources.get::<engine_core::Time>() else {
        return;
    };
    let dt = time.fixed_delta;

    let world_time = match ctx.resources.get::<DayNightCycle>() {
        Some(cycle) => cycle.world_time,
        None => return,
    };

    if ctx.resources.get::<MusicBank>().is_none()
        || ctx.resources.get::<ClientAudio>().is_none()
        || ctx.resources.get::<MusicPlaybackState>().is_none()
    {
        return;
    }

    let playing = {
        let audio = ctx.resources.get_mut::<ClientAudio>().expect("checked");
        audio.engine.music_is_playing()
    };

    let track_to_play = {
        let state = ctx.resources.get_mut::<MusicPlaybackState>().expect("checked");

        if playing {
            state.was_playing = true;
            state.last_world_time = world_time;
            return;
        }

        if state.was_playing {
            state.cooldown_secs = MUSIC_COOLDOWN_SECS;
            state.was_playing = false;
        }

        if state.cooldown_secs > 0.0 {
            state.cooldown_secs = (state.cooldown_secs - dt).max(0.0);
        }

        if !state.seeded {
            state.last_world_time = world_time;
            state.seeded = true;
            return;
        }

        if state.cooldown_secs > 0.0 {
            state.last_world_time = world_time;
            return;
        }

        let crossed = world_time_crossed_anchors(state.last_world_time, world_time);
        let exclude = state.last_track_id.clone();
        state.last_world_time = world_time;

        if crossed.is_empty() {
            None
        } else {
            ctx.resources
                .get::<MusicBank>()
                .and_then(|bank| bank.pick_random(exclude.as_deref()))
                .cloned()
        }
    };

    let Some(track) = track_to_play else {
        return;
    };

    let clip = {
        let bank = ctx.resources.get_mut::<MusicBank>().expect("checked");
        bank.clip(&track.id).cloned()
    };
    let Some(clip) = clip else {
        return;
    };
    let Some(audio) = ctx.resources.get_mut::<ClientAudio>() else {
        return;
    };

    if audio.engine.play_music(clip, 1.0) {
        if let Some(state) = ctx.resources.get_mut::<MusicPlaybackState>() {
            state.last_track_id = Some(track.id.clone());
            state.was_playing = true;
        }
        log::info!(
            "ambient music: {} — {} ({})",
            track.title,
            track.author,
            track.album
        );
    }
}
