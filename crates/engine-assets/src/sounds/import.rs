use std::fs;
use std::path::{Path, PathBuf};

use crate::import::PackSource;
use crate::sounds::manifest::{load_sounds_manifest, SoundsManifest};

#[derive(Debug, Default)]
pub struct SoundImportReport {
    pub copied: Vec<String>,
}

pub fn import_sound_pack(
    pack_path: &Path,
    manifest: &SoundsManifest,
    assets_root: &Path,
) -> Result<SoundImportReport, String> {
    let mut source = PackSource::open(pack_path)?;
    let sounds_root = assets_root.join("sounds");
    fs::create_dir_all(&sounds_root)
        .map_err(|error| format!("create {}: {error}", sounds_root.display()))?;

    let mut report = SoundImportReport::default();
    for spec in &manifest.events {
        for rel in spec.clip_paths() {
            let pack_key = format!("sounds/{rel}");
            let bytes = source.read_mc(&pack_key)?;
            let dest = sounds_root.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("create {}: {error}", parent.display()))?;
            }
            fs::write(&dest, bytes)
                .map_err(|error| format!("write {}: {error}", dest.display()))?;
            report.copied.push(rel);
        }
    }
    Ok(report)
}

pub fn import_sound_pack_from_paths(
    pack_path: &Path,
    manifest_path: &Path,
    assets_root: &Path,
) -> Result<SoundImportReport, String> {
    let manifest = load_sounds_manifest(manifest_path)?;
    import_sound_pack(pack_path, &manifest, assets_root)
}

pub fn default_sound_pack_path(repo_root: &Path) -> PathBuf {
    repo_root.join("source-packs/sound-resource-pack/sound-resource-pack")
}
