# Block textures

Blocks use a **UV layout type** plus a per-block **albedo** image. The engine packs albedos into a GPU atlas at startup — you never edit a master `blocks.png` by hand.

## Folder layout

```
textures/
  layouts/
    cube_v1_template.png     ← Photoshop template (64×32 cross-net)
  blocks/
    grass/albedo.png
    dirt/albedo.png
    stone/albedo.png
```

Regenerate placeholder art:

```bash
cargo run -p engine-assets --bin generate-block-textures
```

## `cube_v1` albedo format

- **Size:** 64×32 pixels, RGBA8 PNG
- **Faces:** 16×16 each, Minecraft-style cross-net (Z-up world)

```
        [top]
[left][front][right][back]
       [bottom]
```

| Face | Pixel origin (x, y) | World normal |
| ---- | ------------------- | ------------ |
| top | (16, 0) | +Z |
| bottom | (32, 0) | −Z |
| left | (0, 16) | −X |
| front | (16, 16) | +Y |
| right | (32, 16) | +X |
| back | (48, 16) | −Y |

Open `layouts/cube_v1_template.png` as a reference. Each face is a flat color with a letter: **T**op green (+Z), **D**own orange (−Z), **L**eft red (−X), **F**ront blue (+Y), **R**ight yellow (+X), **A**back purple (−Y). Paint your art in the same face regions, then save as `blocks/{name}/albedo.png`.

### Grass-style blocks

Paint different regions on the net: green top, dirt bottom, grass-over-dirt sides (green fringe along the top edge of side faces).

### Uniform blocks (stone, dirt)

Paint the same tileable pattern on all six face regions.

## Block definitions

In `assets/blocks/{name}.toml`:

```toml
layout = "cube_v1"   # optional; default when omitted
# texture path defaults to blocks/{name}/
# texture = "blocks/dirt"   # optional override
```

Missing `albedo.png` → **black** fallback at runtime.

## Future: items (torch, etc.)

Non-cube shapes will add new layout types (`torch_prism_v1`, …) with authored meshes and fixed UVs. Multiple albedos can share one layout type — same UV map, different PNG.
