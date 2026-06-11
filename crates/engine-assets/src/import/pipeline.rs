use std::fs;
use std::path::Path;

use crate::blocks::TintModeToml;
use crate::material::DrawCategory;
use crate::import::compose::{compose_albedo, compose_overlay};
use crate::import::manifest::{BlockImportSpec, ColormapManifest, ImportManifest, ItemImportSpec};
use crate::layouts::{face_region, CubeFace, FACE_SIZE};
use crate::import::model::{read_texture_mcmeta, resolve_cube_model};
use crate::import::source::PackSource;

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub blocks_imported: Vec<String>,
    pub colormaps_imported: Vec<String>,
    pub items_imported: Vec<String>,
}

pub fn import_texture_pack(
    pack_path: &Path,
    manifest: &ImportManifest,
    assets_root: &Path,
) -> Result<ImportReport, String> {
    let mut source = PackSource::open(pack_path)?;
    let textures_dir = assets_root.join("textures");
    let blocks_dir = assets_root.join("blocks");

    let mut report = ImportReport {
        blocks_imported: Vec::new(),
        colormaps_imported: Vec::new(),
        items_imported: Vec::new(),
    };

    import_colormaps(&mut source, &manifest.colormaps, &textures_dir, &mut report)?;

    for spec in &manifest.blocks {
        import_block(&mut source, spec, &textures_dir, &blocks_dir)?;
        report.blocks_imported.push(spec.engine.clone());
    }

    for spec in &manifest.items {
        import_item(&mut source, spec, &textures_dir)?;
        report.items_imported.push(spec.engine.clone());
    }

    Ok(report)
}

fn import_colormaps(
    source: &mut PackSource,
    colormaps: &ColormapManifest,
    textures_dir: &Path,
    report: &mut ImportReport,
) -> Result<(), String> {
    let out_dir = textures_dir.join("colormap");
    fs::create_dir_all(&out_dir).map_err(|error| format!("mkdir {}: {error}", out_dir.display()))?;

    if let Some(path) = &colormaps.grass {
        copy_colormap(source, path, &out_dir.join("grass.png"))?;
        report.colormaps_imported.push("grass".into());
    }
    if let Some(path) = &colormaps.foliage {
        copy_colormap(source, path, &out_dir.join("foliage.png"))?;
        report.colormaps_imported.push("foliage".into());
    }
    Ok(())
}

fn import_item(
    source: &mut PackSource,
    spec: &ItemImportSpec,
    textures_dir: &Path,
) -> Result<(), String> {
    let out_dir = textures_dir.join("items");
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("mkdir {}: {error}", out_dir.display()))?;
    let dest = out_dir.join(format!("{}.png", spec.engine));

    if let Some(pack) = &spec.pack {
        let bytes = source.read_mc(&format!("textures/{pack}.png"))?;
        fs::write(&dest, bytes).map_err(|error| format!("write {}: {error}", dest.display()))?;
    } else if spec.from_block_top {
        let albedo_path = textures_dir
            .join("blocks")
            .join(&spec.engine)
            .join("albedo.png");
        let albedo = image::open(&albedo_path)
            .map_err(|error| format!("load {}: {error}", albedo_path.display()))?
            .into_rgba8();
        let region = face_region(CubeFace::Top);
        let icon = image::imageops::crop_imm(&albedo, region.x, region.y, region.w, region.h)
            .to_image();
        if icon.width() != FACE_SIZE || icon.height() != FACE_SIZE {
            return Err(format!(
                "item '{}' top-face crop must be {FACE_SIZE}×{FACE_SIZE}",
                spec.engine
            ));
        }
        icon.save(&dest)
            .map_err(|error| format!("write {}: {error}", dest.display()))?;
    } else {
        return Err(format!(
            "item '{}' needs `pack` or `from_block_top = true`",
            spec.engine
        ));
    }

    println!(
        "imported item '{}' -> {}",
        spec.engine,
        dest.display()
    );
    Ok(())
}

fn copy_colormap(source: &mut PackSource, mc_path: &str, dest: &Path) -> Result<(), String> {
    let bytes = source.read_mc(&format!("textures/{mc_path}"))?;
    fs::write(dest, bytes).map_err(|error| format!("write {}: {error}", dest.display()))
}

fn import_block(
    source: &mut PackSource,
    spec: &BlockImportSpec,
    textures_dir: &Path,
    blocks_dir: &Path,
) -> Result<(), String> {
    let cube = resolve_cube_model(source, &spec.model)?;
    let material_dir = textures_dir.join("blocks").join(&spec.engine);
    fs::create_dir_all(&material_dir)
        .map_err(|error| format!("mkdir {}: {error}", material_dir.display()))?;

    let albedo = compose_albedo(source, &cube)?;
    let albedo_path = material_dir.join("albedo.png");
    albedo
        .save(&albedo_path)
        .map_err(|error| format!("write {}: {error}", albedo_path.display()))?;

    copy_animation_mcmeta(source, &cube.faces, &material_dir)?;

    let import_overlay = spec.overlay || cube.overlay_sides.is_some();
    if import_overlay {
        let overlay_ref = cube
            .overlay_sides
            .as_deref()
            .ok_or_else(|| format!("block '{}' marked overlay but model has none", spec.engine))?;
        let overlay = compose_overlay(source, overlay_ref)?;
        let overlay_path = material_dir.join("overlay.png");
        overlay
            .save(&overlay_path)
            .map_err(|error| format!("write {}: {error}", overlay_path.display()))?;
    }

    let tint = spec
        .tint
        .or_else(|| {
            if cube.tint_grass {
                Some(TintModeToml::BiomeGrass)
            } else if cube.tint_foliage {
                Some(TintModeToml::BiomeFoliage)
            } else {
                None
            }
        });

    let draw = spec.draw.or_else(|| {
        if cube.tint_foliage {
            Some(DrawCategory::Cutout)
        } else {
            None
        }
    });

    patch_block_toml(
        &blocks_dir.join(format!("{}.toml", spec.engine)),
        draw,
        tint,
        import_overlay,
    )?;

    println!(
        "imported block '{}' from model '{}' -> {}",
        spec.engine,
        spec.model,
        material_dir.display()
    );
    Ok(())
}

fn copy_animation_mcmeta(
    source: &mut PackSource,
    faces: &std::collections::HashMap<crate::layouts::CubeFace, String>,
    material_dir: &Path,
) -> Result<(), String> {
    let dest = material_dir.join("albedo.png.mcmeta");
    for texture_ref in faces.values() {
        let Ok(mcmeta) = read_texture_mcmeta(source, texture_ref) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&mcmeta) else {
            continue;
        };
        if value.get("animation").is_none() {
            continue;
        }
        fs::write(&dest, mcmeta)
            .map_err(|error| format!("write {}: {error}", dest.display()))?;
        return Ok(());
    }
    let _ = fs::remove_file(dest);
    Ok(())
}

fn patch_block_toml(
    path: &Path,
    draw: Option<DrawCategory>,
    tint: Option<TintModeToml>,
    overlay: bool,
) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!(
            "block definition missing at {} — add a base .toml before importing",
            path.display()
        ));
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut value: toml::Value = toml::from_str(&contents)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;

    let table = value
        .as_table_mut()
        .ok_or_else(|| format!("{} must be a TOML table", path.display()))?;

    if let Some(draw) = draw {
        table.insert(
            "draw".into(),
            toml::Value::String(match draw {
                DrawCategory::Opaque => "opaque".into(),
                DrawCategory::Cutout => "cutout".into(),
                DrawCategory::Transparent => "transparent".into(),
            }),
        );
    }

    if let Some(tint) = tint {
        let tint_str = match tint {
            TintModeToml::None => "none",
            TintModeToml::BiomeGrass => "biome_grass",
            TintModeToml::BiomeFoliage => "biome_foliage",
        };
        table.insert("tint".into(), toml::Value::String(tint_str.into()));
    }

    if overlay {
        table.insert(
            "overlays".into(),
            toml::Value::Array(vec![toml::Value::Table({
                let mut overlay = toml::map::Map::new();
                overlay.insert("faces".into(), toml::Value::String("side".into()));
                overlay.insert("path".into(), toml::Value::String("overlay.png".into()));
                overlay
            })]),
        );
    } else {
        table.remove("overlays");
    }

    let out = toml::to_string_pretty(&value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    fs::write(path, out).map_err(|error| format!("write {}: {error}", path.display()))
}

