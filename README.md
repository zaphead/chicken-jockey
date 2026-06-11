# OpenCraft

An open-source, high-fantasy voxel sandbox built from the ground up in Rust. Explore procedurally generated worlds, reshape the terrain block by block, and ride chickens across the hills — with an engine designed for performance, moddability, and multiplayer from day one.

OpenCraft exists because popular voxel games have outgrown their original architectures. Modding is bolted on, draw distance fights the renderer, and multiplayer authority is an afterthought. This project treats those problems as first-class design constraints: a sparse voxel octree (SVO) for scalable worlds, a strict entity–component–system (ECS) architecture for parallel gameplay, and a shared client/server codebase so the same logic runs headless on a server or in a window.

---

## What you can do

| Today | Coming soon |
| ----- | ----------- |
| Fly through a voxel world with a spectator camera | First-person player controller with gravity and jumping |
| See terrain rendered with chunk meshing, level-of-detail (LOD), and material atlases | Break and place blocks with raycast interaction |
| Connect a client to a local authoritative server over QUIC (Quick UDP Internet Connections) | Spawn chickens, mount them, and ride across the landscape |
| Run the server headless with no graphics stack | Full client prediction and reconciliation polish |

The playable fantasy loop is simple: **walk a living world, change it, find chickens, mount up, and go.** Multiplayer, rich biomes, and persistence follow the same architecture — they are phased in, not patched on.

---

## Why this engine is different

**Performance by structure, not hacks.** Worlds are stored in an SVO with inherent LOD — distant terrain uses coarser detail automatically. Mesh generation runs on the graphics processing unit (GPU); the renderer reads a snapshot of game state, never the live ECS world. Large view distances and dense worlds are the goal, not a stretch target.

**Extensibility without recompilation theater.** Block types, textures, and material behavior live in data files (`assets/blocks/`, texture atlases). Game logic is ECS systems in a shared `game` crate — the same code powers the client and the headless server. Add a block, wire a system, register it in the schedule.

**Multiplayer as a peer, not a port.** The server is authoritative over world state. Clients predict input and reconcile against server snapshots. Transport is QUIC — reliable ordered streams for world data, unreliable datagrams for low-latency movement. One protocol, one game logic crate, two binaries.

**Explicit over magical.** No hidden globals, no render thread touching gameplay, no systems that only work on one side of the wire unless they belong there. If you can read the startup code, you can trace the whole execution graph.

For the full engineering contract, see [`docs/design-doc.md`](docs/design-doc.md). For current build status and phase tracking, see [`docs/implementation-plan.md`](docs/implementation-plan.md).

---

## Quick start

**Requirements:** Rust (2021 edition), a GPU with Vulkan, Metal, or DirectX 12 support via wgpu.

```bash
# Build everything
cargo build

# Run the client (local spectator camera)
cargo run -p client

# Run the authoritative server (headless, QUIC on 127.0.0.1:4242)
cargo run -p server

# Client connected to server
OC_SERVER=127.0.0.1:4242 cargo run -p client
```

Click the window to capture the mouse. **WASD** moves on the horizontal plane, **Space / Ctrl** moves up and down, mouse looks around.

---

## Project layout

```
crates/
  engine-core/      ECS scheduler, events, time
  engine-world/     SVO, block registry, world mutations
  engine-render/    wgpu pipeline, meshing, extract/render boundary
  engine-net/       QUIC transport, protocol messages
  engine-input/     Action mapping and polling
  engine-assets/      Async asset loading, block materials
  engine-audio/     Audio stub (future)
  game/             All shared gameplay — player, chickens, terrain, physics
client/             Window, rendering, input, net client wiring
server/             Headless authority, net server wiring
assets/             Blocks, textures, shaders
docs/               Architecture and implementation plan
```

`game` never depends on rendering or audio. `client` and `server` are the only places crates are composed into runnable applications.

---

## Stack

| Layer | Technology |
| ----- | ---------- |
| Language | Rust |
| Graphics | wgpu |
| Windowing | winit |
| ECS (entity–component–system) | hecs + custom scheduler |
| Networking | quinn (QUIC transport) |
| Serialization | bincode |
| Math | glam |

---

## Contributing

This project is pre-launch and moving fast. Before changing code:

1. Read [`docs/design-doc.md`](docs/design-doc.md) — architectural constraints are not negotiable.
2. Read [`docs/implementation-plan.md`](docs/implementation-plan.md) — know which phase your work belongs to.
3. Keep shared gameplay in `game`, keep rendering out of game logic, route all voxel edits through `WorldMutationQueue`.

Issues and pull requests welcome. Prefer focused changes that match existing crate boundaries and ECS patterns.

---

## Glossary

| Term | Meaning |
| ---- | ------- |
| **ECS** | Entity–component–system — game objects are entities; data lives in components; behavior lives in systems that query components each tick |
| **SVO** | Sparse voxel octree — a tree that subdivides 3D space; only stores voxels where something exists, enabling huge worlds and natural LOD |
| **LOD** | Level of detail — rendering coarser geometry farther from the camera to save GPU work |
| **GPU** | Graphics processing unit — the chip that draws the world |
| **QUIC** | Quick UDP Internet Connections — a modern network protocol (built on UDP) with both reliable streams and fast unreliable packets on one connection |
| **wgpu** | WebGPU implementation in Rust — cross-platform graphics API used for rendering |
| **MIT** | Permissive open-source license — use, modify, and distribute with minimal restrictions |

---

## License

MIT — see workspace `Cargo.toml`. Use it, fork it, mod it, ride chickens in it.
