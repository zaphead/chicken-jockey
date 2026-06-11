# Source packs

Upstream Minecraft resource packs kept for curated import into `assets/`. Nothing here is loaded at runtime.

| Pack | Contents | Import |
| ---- | -------- | ------ |
| `whimscape-26.1-r2/` | Textures, models, blockstates | See below |
| `sound-resource-pack/` | Vanilla sounds (~3k `.ogg`) | See below |

## Textures (Whimscape)

```bash
cargo run -p engine-assets --bin import-texture-pack -- \
  --pack source-packs/whimscape-26.1-r2/whimscape-26.1-r2.zip
```

Manifest: `assets/import/manifest.toml`

## Sounds (Vanilla Sounds by TheRealKuchen)

```bash
cargo run -p engine-assets --bin import-sound-pack
```

Manifest: `assets/import/sounds-manifest.toml`

Browse `assets/minecraft/sounds/` under `sound-resource-pack/`.
