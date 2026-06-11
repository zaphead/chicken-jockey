use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rand::Rng;

use super::manifest::{AttenuationToml, SoundEventSpec, SoundsManifest};

#[derive(Debug, Clone)]
pub struct ResolvedSoundEvent {
    pub category: String,
    pub clip_paths: Vec<String>,
    pub volume: f32,
    pub pitch: [f32; 2],
    pub attenuation: AttenuationToml,
    pub max_distance: f32,
}

#[derive(Debug, Default, Clone)]
pub struct SoundRegistry {
    events: HashMap<(String, Option<String>), ResolvedSoundEvent>,
    clips: HashMap<String, Vec<u8>>,
}

impl SoundRegistry {
    pub fn from_manifest(manifest: &SoundsManifest, sounds_dir: &Path) -> Result<Self, String> {
        let mut registry = Self::default();
        for spec in &manifest.events {
            let mut clip_paths = Vec::with_capacity(spec.variants as usize);
            for index in 1..=spec.variants {
                let rel = if spec.variants == 1 && !spec.indexed {
                    format!("{}.ogg", spec.pack)
                } else {
                    format!("{}{}.ogg", spec.pack, index)
                };
                let disk = sounds_dir.join(&rel);
                if !disk.is_file() {
                    return Err(format!("missing sound file {}", disk.display()));
                }
                let bytes = fs::read(&disk)
                    .map_err(|error| format!("read {}: {error}", disk.display()))?;
                registry.clips.insert(rel.clone(), bytes);
                clip_paths.push(rel);
            }
            registry.events.insert(
                (spec.kind.clone(), spec.sound_group.clone()),
                ResolvedSoundEvent {
                    category: spec.category.clone(),
                    clip_paths,
                    volume: spec.volume,
                    pitch: spec.pitch,
                    attenuation: spec.attenuation,
                    max_distance: spec.max_distance,
                },
            );
        }
        Ok(registry)
    }

    pub fn resolve(&self, kind: &str, sound_group: Option<&str>) -> Option<&ResolvedSoundEvent> {
        self.events
            .get(&(kind.to_string(), sound_group.map(str::to_string)))
    }

    pub fn pick_clip_path<'a>(&self, event: &'a ResolvedSoundEvent) -> Option<(&'a str, f32)> {
        if event.clip_paths.is_empty() {
            return None;
        }
        let index = rand::thread_rng().gen_range(0..event.clip_paths.len());
        let path = &event.clip_paths[index];
        if !self.clips.contains_key(path) {
            return None;
        }
        let pitch = random_pitch(event.pitch);
        Some((path.as_str(), pitch))
    }

    pub fn clip_bytes(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.clips.iter()
    }

    pub fn clip_bytes_for(&self, path: &str) -> Option<&[u8]> {
        self.clips.get(path).map(Vec::as_slice)
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

pub fn sounds_asset_path(manifest_dir: impl AsRef<Path>) -> PathBuf {
    super::super::assets_dir(manifest_dir).join("sounds")
}

pub fn sounds_manifest_path(manifest_dir: impl AsRef<Path>) -> PathBuf {
    super::super::assets_dir(manifest_dir).join("import/sounds-manifest.toml")
}

pub fn load_sound_registry(manifest_dir: impl AsRef<Path>) -> Result<SoundRegistry, String> {
    let manifest_dir = manifest_dir.as_ref();
    let manifest = super::manifest::load_sounds_manifest(&sounds_manifest_path(manifest_dir))?;
    SoundRegistry::from_manifest(&manifest, &sounds_asset_path(manifest_dir))
}

fn random_pitch(range: [f32; 2]) -> f32 {
    if (range[1] - range[0]).abs() < f32::EPSILON {
        range[0]
    } else {
        rand::thread_rng().gen_range(range[0]..=range[1])
    }
}

impl SoundEventSpec {
    pub fn clip_paths(&self) -> Vec<String> {
        (1..=self.variants)
            .map(|index| {
                if self.variants == 1 && !self.indexed {
                    format!("{}.ogg", self.pack)
                } else {
                    format!("{}{}.ogg", self.pack, index)
                }
            })
            .collect()
    }
}
