use std::collections::{HashMap, VecDeque};

use glam::Vec3;
use kira::{
    sound::static_sound::StaticSoundHandle, track::TrackHandle, AudioManager, AudioManagerSettings,
    Decibels, DefaultBackend, Panning, PlaybackRate, Tween,
};
use kira::track::TrackBuilder;

use crate::attenuation::Attenuation;
use crate::categories::SoundCategory;
use crate::clip::SoundClip;
use crate::spatial::{mix_for_listener, Listener};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(u64);

#[derive(Debug, Clone)]
pub struct PlayRequest {
    pub clip: SoundClip,
    pub position: Vec3,
    pub volume: f32,
    pub pitch: f32,
    pub category: SoundCategory,
    pub attenuation: Attenuation,
}

struct ActiveVoice {
    category: SoundCategory,
}

pub struct AudioEngine {
    #[allow(dead_code)]
    manager: AudioManager<DefaultBackend>,
    category_tracks: HashMap<SoundCategory, TrackHandle>,
    category_volumes: HashMap<SoundCategory, f32>,
    listener: Listener,
    next_handle: u64,
    active: HashMap<SoundHandle, ActiveVoice>,
    voice_order: VecDeque<SoundHandle>,
    music_handle: Option<StaticSoundHandle>,
    enabled: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|error| format!("audio device: {error}"))?;

        let mut category_tracks = HashMap::new();
        for category in [
            SoundCategory::Blocks,
            SoundCategory::Player,
            SoundCategory::Music,
        ] {
            let track = manager
                .add_sub_track(TrackBuilder::new())
                .map_err(|error| format!("category track {:?}: {error}", category))?;
            category_tracks.insert(category, track);
        }

        let mut category_volumes = HashMap::new();
        for category in SoundCategory::ALL {
            category_volumes.insert(category, 1.0);
        }

        Ok(Self {
            manager,
            category_tracks,
            category_volumes,
            listener: Listener::default(),
            next_handle: 1,
            active: HashMap::new(),
            voice_order: VecDeque::new(),
            music_handle: None,
            enabled: true,
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_listener(&mut self, listener: Listener) {
        self.listener = listener;
    }

    pub fn set_category_volume(&mut self, category: SoundCategory, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        self.category_volumes.insert(category, volume);
        if category == SoundCategory::Master {
            for cat in [
                SoundCategory::Blocks,
                SoundCategory::Player,
                SoundCategory::Music,
            ] {
                self.apply_track_volume(cat);
            }
        } else {
            self.apply_track_volume(category);
        }
    }

    fn master_volume(&self) -> f32 {
        *self
            .category_volumes
            .get(&SoundCategory::Master)
            .unwrap_or(&1.0)
    }

    fn category_volume(&self, category: SoundCategory) -> f32 {
        self.master_volume() * (*self.category_volumes.get(&category).unwrap_or(&1.0))
    }

    fn apply_track_volume(&mut self, category: SoundCategory) {
        let db = linear_to_decibels(self.category_volume(category));
        let Some(track) = self.category_tracks.get_mut(&category) else {
            return;
        };
        track.set_volume(db, Tween::default());
    }

    pub fn play(&mut self, request: PlayRequest) -> Option<SoundHandle> {
        if !self.enabled {
            return None;
        }

        let category = request.category;
        self.evict_voices_if_needed(category);

        let spatial = mix_for_listener(&self.listener, request.position, request.attenuation);
        let linear = request.volume * spatial.volume * self.category_volume(category);
        if linear <= 0.0001 {
            return None;
        }

        let track = self.category_tracks.get_mut(&category)?;
        let pan = Panning::from(spatial.pan * 2.0 - 1.0);
        let sound = request
            .clip
            .data()
            .clone()
            .volume(linear_to_decibels(linear))
            .playback_rate(PlaybackRate(request.pitch as f64))
            .panning(pan);

        if track.play(sound).is_err() {
            return None;
        }

        let handle = SoundHandle(self.next_handle);
        self.next_handle += 1;
        self.active.insert(handle, ActiveVoice { category });
        self.voice_order.push_back(handle);
        Some(handle)
    }

    pub fn stop(&mut self, handle: SoundHandle) {
        self.active.remove(&handle);
        self.voice_order.retain(|id| *id != handle);
    }

    pub fn play_music(&mut self, clip: SoundClip, volume: f32) -> bool {
        if !self.enabled || self.music_is_playing() {
            return false;
        }

        let linear = volume.clamp(0.0, 1.0) * self.category_volume(SoundCategory::Music);
        if linear <= 0.0001 {
            return false;
        }

        let track = match self.category_tracks.get_mut(&SoundCategory::Music) {
            Some(track) => track,
            None => return false,
        };
        let sound = clip
            .data()
            .clone()
            .volume(linear_to_decibels(linear));
        let handle = match track.play(sound) {
            Ok(handle) => handle,
            Err(_) => return false,
        };
        self.music_handle = Some(handle);
        true
    }

    pub fn music_is_playing(&mut self) -> bool {
        let Some(handle) = self.music_handle.as_ref() else {
            return false;
        };
        if handle.state().is_advancing() {
            true
        } else {
            self.music_handle = None;
            false
        }
    }

    pub fn stop_music(&mut self) {
        if let Some(mut handle) = self.music_handle.take() {
            handle.stop(Tween::default());
        }
    }

    pub fn tick(&mut self) {
        while self.voice_order.len() > self.active.len() {
            self.voice_order.pop_front();
        }
        while self.voice_order.len() > 128 {
            if let Some(oldest) = self.voice_order.pop_front() {
                self.active.remove(&oldest);
            }
        }
    }

    fn evict_voices_if_needed(&mut self, category: SoundCategory) {
        let cap = category.voice_cap();
        if cap == usize::MAX {
            return;
        }
        let count = self
            .active
            .values()
            .filter(|voice| voice.category == category)
            .count();
        if count < cap {
            return;
        }
        if let Some(index) = self.voice_order.iter().position(|handle| {
            self.active
                .get(handle)
                .is_some_and(|voice| voice.category == category)
        }) {
            let handle = self.voice_order.remove(index).expect("voice index");
            self.active.remove(&handle);
        }
    }
}

fn linear_to_decibels(linear: f32) -> Decibels {
    if linear <= 0.0001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * linear.log10())
    }
}
