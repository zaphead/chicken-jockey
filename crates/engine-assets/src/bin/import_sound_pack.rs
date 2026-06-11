//! Import curated sounds from a Minecraft resource pack into engine assets.
use std::env;
use std::path::PathBuf;

use engine_assets::{
    assets_dir, import_sound_pack_from_paths, load_sounds_manifest, sounds_manifest_path,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut pack_path: Option<PathBuf> = None;
    let mut manifest_path: Option<PathBuf> = None;
    let mut assets_root: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pack" => {
                i += 1;
                pack_path = Some(PathBuf::from(args.get(i).expect("--pack requires a path")));
            }
            "--manifest" => {
                i += 1;
                manifest_path = Some(PathBuf::from(
                    args.get(i).expect("--manifest requires a path"),
                ));
            }
            "--assets" => {
                i += 1;
                assets_root = Some(PathBuf::from(args.get(i).expect("--assets requires a path")));
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pack_path = pack_path.unwrap_or_else(|| {
        engine_assets::default_sound_pack_path(
            &PathBuf::from(manifest_dir)
                .parent()
                .and_then(|p| p.parent())
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(manifest_dir)),
        )
    });
    let manifest_path = manifest_path.unwrap_or_else(|| sounds_manifest_path(manifest_dir));
    let assets_root = assets_root.unwrap_or_else(|| assets_dir(manifest_dir));

    let manifest = load_sounds_manifest(&manifest_path).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    });

    let report =
        import_sound_pack_from_paths(&pack_path, &manifest_path, &assets_root).unwrap_or_else(
            |error| {
                eprintln!("import failed: {error}");
                std::process::exit(1);
            },
        );

    println!(
        "imported {} sound files for {} events",
        report.copied.len(),
        manifest.events.len()
    );
}

fn print_usage() {
    eprintln!(
        "usage: import-sound-pack [--pack <zip-or-dir>] [--manifest path] [--assets path]\n\
         \n\
         Copies manifest-listed .ogg files from a Minecraft pack into assets/sounds/."
    );
}
