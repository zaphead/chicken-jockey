use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::server::assets_dir;

#[derive(Debug, Clone, Deserialize)]
pub struct MusicManifest {
    #[serde(default)]
    pub tracks: Vec<MusicTrack>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicTrack {
    pub id: String,
    pub file: String,
    pub title: String,
    pub author: String,
    pub album: String,
    #[serde(default)]
    pub source_url: Option<String>,
}

pub fn music_manifest_path(manifest_dir: impl AsRef<Path>) -> PathBuf {
    assets_dir(manifest_dir).join("music/manifest.toml")
}

pub fn music_asset_path(manifest_dir: impl AsRef<Path>) -> PathBuf {
    assets_dir(manifest_dir).join("music")
}

pub fn load_music_manifest(path: &Path) -> Result<MusicManifest, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    toml::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_music_manifest() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let manifest = load_music_manifest(&music_manifest_path(&root))
        .expect("music manifest");
        assert!(!manifest.tracks.is_empty());
        assert!(manifest.tracks.iter().any(|track| track.id == "as_time_flies"));
    }
}
