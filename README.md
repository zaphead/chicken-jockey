# OpenCraft

A voxel sandbox prototype built in Rust.

## Release (macOS)

No Rust toolchain required.

1. Open [Releases](https://github.com/zaphead/OpenCraft/releases/tag/macos-latest) and download the latest **OpenCraft-*-macos-*.dmg**.
2. Open the DMG and drag **OpenCraft** into Applications.
3. If macOS says the app is **damaged** or won't open, remove the download quarantine flag (this is normal for unsigned builds):

```bash
xattr -cr /Applications/OpenCraft.app
```

4. Launch OpenCraft. If macOS still blocks it, right-click the app → **Open**, then confirm.

Each push to `main` rebuilds the DMG automatically. Fully seamless installs (no quarantine workaround) require Apple code signing and notarization, which we can add later.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- macOS, Linux, or Windows with a working GPU driver for wgpu

### Run the client (local single-player)

From the repo root:

```bash
cargo run -p client
```

Click the window to capture the mouse. **Esc** releases the cursor when the menu is open.

### Optional: multiplayer dev setup

Terminal 1 — server:

```bash
cargo run -p server
```

Terminal 2 — client connected to the server:

```bash
OC_SERVER=127.0.0.1:4242 cargo run -p client
```

### Build a macOS DMG locally

```bash
bash scripts/package-macos.sh
```

Output: `dist/OpenCraft-0.1.0-macos-local.dmg`

### Controls (local client)

| Key | Action |
| --- | --- |
| WASD | Move |
| Mouse | Look |
| Space | Jump |
| LMB / RMB | Break / place blocks |
| 1 / 2 | Hand / wooden pickaxe |
| E | Inventory |
| M | Survival ↔ Spectator |
| Tab | Cycle debug worlds |
| F | Interact |

### Troubleshooting

```bash
bash scripts/diagnose-client.sh
```

Or run the headless pipeline check:

```bash
cargo run --bin client-diagnose
```

For live diagnostics while playing:

```bash
OC_DIAGNOSTIC=1 RUST_LOG=info cargo run -p client
```
