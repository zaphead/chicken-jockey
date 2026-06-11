use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SoundsManifest {
    #[serde(default)]
    pub events: Vec<SoundEventSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SoundEventSpec {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub sound_group: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
    pub pack: String,
    #[serde(default = "default_variants")]
    pub variants: u8,
    /// When false and `variants == 1`, resolves to `{pack}.ogg` instead of `{pack}1.ogg`.
    #[serde(default)]
    pub indexed: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default = "default_pitch")]
    pub pitch: [f32; 2],
    #[serde(default)]
    pub attenuation: AttenuationToml,
    #[serde(default = "default_max_distance")]
    pub max_distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttenuationToml {
    #[default]
    Linear,
    None,
}

fn default_category() -> String {
    "blocks".to_string()
}

fn default_variants() -> u8 {
    1
}

fn default_volume() -> f32 {
    1.0
}

fn default_pitch() -> [f32; 2] {
    [1.0, 1.0]
}

fn default_max_distance() -> f32 {
    16.0
}

pub fn load_sounds_manifest(path: &Path) -> Result<SoundsManifest, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    toml::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))
}
