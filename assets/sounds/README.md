# Sound assets

Imported from `source-packs/sound-resource-pack/` via `cargo run -p engine-assets --bin import-sound-pack`.
Curated events live in `assets/import/sounds-manifest.toml`.

## Vanilla file → motion (Minecraft parity)

| Pack path | Used for |
| --------- | -------- |
| `dig/grass{1-4}.ogg` | Grass block break / place |
| `dig/gravel{1-4}.ogg` | Dirt break / place (MC gravel sound type) |
| `dig/stone{1-4}.ogg` | Stone break / place |
| `step/grass{1-6}.ogg` | Grass mining hits, footsteps, leaves |
| `step/gravel{1-4}.ogg` | Dirt mining hits, footsteps |
| `step/stone{1-6}.ogg` | Stone mining hits, footsteps |
| `damage/fallsmall.ogg` | Land after 4–7 block fall |
| `damage/fallbig.ogg` | Land after 8+ block fall |

OpenCraft `sound_group` on blocks (`assets/blocks/*.toml`) maps to the rows above.

**Mining ticks** use `step/*` (vanilla `block.*.hit`), not `dig/*`.

**Fall sounds** only when airborne drop ≥ 4 blocks; normal jumps are silent.
