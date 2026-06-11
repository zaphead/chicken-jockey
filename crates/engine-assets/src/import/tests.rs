#[cfg(test)]
mod whimscape {
    use std::path::{Path, PathBuf};

    use crate::import::{import_texture_pack, load_manifest};
    use crate::material::pack_block_materials;
    use crate::server::{assets_dir, blocks_asset_path};
    use crate::{load_block_registry, textures_asset_path};

    fn whimscape_zip() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../source-packs/whimscape-26.1-r2/whimscape-26.1-r2.zip")
    }

    #[test]
    fn imports_whimscape_blocks_into_temp_assets() {
        let whimscape = whimscape_zip();
        if !whimscape.is_file() {
            eprintln!(
                "skip imports_whimscape_blocks_into_temp_assets: missing {}",
                whimscape.display()
            );
            return;
        }

        let temp = tempfile::tempdir().expect("tempdir");
        let assets = temp.path().join("assets");
        let blocks_dir = assets.join("blocks");
        std::fs::create_dir_all(&blocks_dir).expect("blocks dir");

        let repo_blocks = blocks_asset_path(env!("CARGO_MANIFEST_DIR"));
        for name in ["grass", "dirt", "stone", "leaves", "air"] {
            std::fs::copy(
                repo_blocks.join(format!("{name}.toml")),
                blocks_dir.join(format!("{name}.toml")),
            )
            .expect("copy block toml");
        }

        let manifest =
            load_manifest(&assets_dir(env!("CARGO_MANIFEST_DIR")).join("import/manifest.toml"))
                .expect("manifest");

        let report = import_texture_pack(&whimscape, &manifest, &assets)
            .expect("import");
        assert_eq!(report.blocks_imported.len(), 4);
        assert_eq!(report.items_imported.len(), 5);
        for name in ["dirt", "grass", "stone", "leaves", "wooden_pickaxe"] {
            assert!(
                assets
                    .join("textures/items")
                    .join(format!("{name}.png"))
                    .is_file(),
                "missing item icon {name}"
            );
        }

        let registry = load_block_registry(&blocks_dir);
        let textures = textures_asset_path(assets.to_str().unwrap());
        pack_block_materials(&textures, &registry).expect("pack imported materials");
    }
}
