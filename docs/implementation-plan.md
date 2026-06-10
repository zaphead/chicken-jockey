# Chicken Jockey — Implementation Plan

**Status:** Living Document

**Goal:** Build a playable rough prototype — a voxel world you can walk in, break/place blocks, find chickens, mount them, and ride around — on the architecture defined in [`design-doc.md`](./design-doc.md).

**Authority:** [`design-doc.md`](./design-doc.md) defines *how* we build. This document defines *what* we build next and in what order.

---

## Target playable loop (MVP)

1. Launch client → flat or procedurally generated terrain appears.
2. WASD + mouse look → move and look around.
3. Left-click breaks voxels; right-click places voxels.
4. Chickens spawn in the world and wander.
5. Interact near a chicken → mount; dismount with a key.
6. While mounted, chicken moves faster with its own steering + player input.

Multiplayer, audio, advanced LOD, and polish are explicitly out of scope until after this loop works locally.

---

## System overview (target state)

```mermaid
flowchart TB
    subgraph binaries
        Client[client binary]
        Server[server binary]
    end

    subgraph engine
        Core[engine-core<br/>ECS scheduler · time · events]
        World[engine-world<br/>SVO · mutations · blocks]
        Render[engine-render<br/>extract · wgpu pipeline]
        Input[engine-input<br/>actions · polling]
        Assets[engine-assets<br/>handles · loader]
        Net[engine-net<br/>protocol stubs]
        Audio[engine-audio<br/>stub]
    end

    subgraph gameplay
        Game[game crate<br/>player · chicken · mount · spawn]
    end

    Client --> Core
    Client --> Render
    Client --> Input
    Client --> Assets
    Client --> Game
    Server --> Core
    Server --> World
    Server --> Net
    Server --> Game

    Game --> Core
    Game --> World
    Render --> World
    Input --> Core
```

---

## Phase tracker

| Phase | Name | Status | Done when |
| ----- | ---- | ------ | --------- |
| 0 | Workspace bootstrap | Complete | `cargo build` succeeds; crate layout matches design doc §2 |
| 1 | ECS core + schedule | Complete | Headless tick loop runs PreUpdate → Update → PostUpdate |
| 2 | Client window + clear screen | Complete | Window opens, wgpu clears to a color, closes cleanly |
| 3 | Voxel world (minimal SVO) | Complete | Flat test world queryable; mutations via `WorldMutationQueue` |
| 4 | Block registry + data files | Complete | Blocks defined in JSON/TOML; loaded into `BlockRegistry` |
| 5 | Voxel rendering (MVP) | Complete | Greedy-meshed chunks visible; camera moves through scene |
| 6 | Input + player controller | Complete | WASD look/move; gravity; stand on terrain |
| 7 | Block interaction | Complete | Break/place voxels through mutation queue |
| 8 | Terrain generation | Complete | Procedural heightmap fills a playable area at startup |
| 9 | Chickens + mounting | Complete | Spawn, wander, mount/dismount, ride movement |
| 10 | Server binary (local) | Complete | Headless authoritative server; QUIC on `127.0.0.1:4242` |
| 11 | Render hardening | Partial | Parallel chunk meshing (rayon), distance LOD, extract snapshot; GPU compute + render thread deferred (macOS surface constraint) |
| 12 | Networking (QUIC) | Complete | `quinn` transport, bincode protocol, authoritative blocks, client prediction stub |

Update the **Status** column as work completes. Add dated notes under [Progress log](#progress-log).

---

## Phase details

### Phase 0 — Workspace bootstrap

Create the Cargo workspace and empty crates exactly as specified in design doc §2:

```
crates/engine-core
crates/engine-world
crates/engine-render
crates/engine-net
crates/engine-audio      # empty stub
crates/engine-input
crates/engine-assets
crates/game
client/
server/
```

**Deliverables**

- Root `Cargo.toml` workspace with shared dependency versions (`hecs`, `wgpu`, `winit`, `glam`, `serde`, `bincode`).
- Each crate compiles with a `lib.rs` (or `main.rs` for binaries) and a one-line crate doc comment.
- `client` and `server` binaries print a startup message and exit (or enter an empty loop).

**Constraints:** Enforce dependency rules from design doc §2 from day one. `game` must not depend on `engine-render` or `engine-audio`.

---

### Phase 1 — ECS core + schedule

Implement `engine-core`:

- `hecs` `World` wrapped in an application-owned struct.
- Resource storage (type map or `Resource` trait + insert/get).
- Command buffer for deferred entity/component changes; flush at `PostUpdate`.
- Single-frame event bus (emit/consume, cleared each tick).
- `Schedule` with stages: `PreUpdate`, `Update`, `PostUpdate` (add `Physics`, `Extract`, `Render` later).
- System registration with explicit ordering where needed.

**Deliverables**

- `server` runs a headless loop: fixed timestep, runs schedule, logs tick count.
- At least one test system proves commands and events work.

**Done when:** No game logic outside systems; no global mutable game state.

---

### Phase 2 — Client window + clear screen

Wire `client` binary:

- `winit` event loop on main thread.
- `engine-render` initializes wgpu `Device`/`Queue`/`Surface`.
- Each frame: poll input events, run schedule (empty stages OK), present swapchain.

**Deliverables**

- Window titled "Chicken Jockey".
- Stable 60 FPS clear-color frame (no voxels yet).

**Done when:** Clean shutdown on window close; no wgpu validation errors.

---

### Phase 3 — Voxel world (minimal SVO)

Implement `engine-world` with a **minimal** SVO — not full LOD/Transvoxel yet:

- Fixed leaf size (1 voxel per leaf at max depth).
- Sparse allocation (air = no node).
- Query API: `get_block(pos)`, `is_solid(pos)`, region iteration.
- `WorldMutationQueue` resource: queue `set_block` during `Update`, flush in `PostUpdate` with parent aggregate propagation (even if aggregates are trivial at first).
- Emit `BlockChanged` events on flush.

**Deliverables**

- Startup fills a 64×16×64 flat stone floor + air above (hardcoded in a world-init system for now).
- Unit tests for set/get and queue flush consistency.

**Note:** Full LOD and Transvoxel come in Phase 11. Structure the API now so callers never touch SVO internals.

---

### Phase 4 — Block registry + data files

- `assets/blocks/*.toml` (or JSON) defining block id, name, solid, opaque, texture keys.
- `BlockRegistry` resource loaded at startup via `engine-assets` (sync load acceptable for MVP; async loader stubbed).
- Systems resolve behavior through registry lookups — no magic block IDs in game logic.

**Deliverables**

- At minimum: `air`, `stone`, `dirt`, `grass`.
- Registry hot-reload can wait; loader trait exists.

---

### Phase 5 — Voxel rendering (MVP)

**MVP exception (documented):** Design doc §5.4 targets GPU compute meshing. For this phase only, use **CPU greedy meshing** per chunk to unblock gameplay. Track migration to compute in Phase 11.

Implement in `engine-render`:

- Chunk keys (e.g. 16³) mapped to mesh handles.
- Subscribe to `BlockChanged` → mark dirty chunks → regenerate mesh.
- Basic vertex format: position + normal + UV.
- Simple unlit or flat-lit shader.
- `Camera` component + uniform for view-projection.

**Extract phase (minimal):** Copy camera + visible chunk mesh list into render world each frame. Full render-thread split deferred to Phase 11.

**Deliverables**

- Fly camera (temporary) orbiting or moving through the flat world.
- Meshes update when blocks change.

---

### Phase 6 — Input + player controller

Implement `engine-input`:

- Action map: `MoveForward`, `MoveBack`, `MoveLeft`, `MoveRight`, `Jump`, `Look` (mouse delta).
- Poll in `PreUpdate`; write to `InputState` resource.

Implement in `game`:

- `Player` entity with `Transform`, `Velocity`, `Collider` (AABB).
- `Camera` attached to player (first-person).
- Movement system in `Update`; physics in `Physics` stage (add stage in this phase).

Physics MVP:

- Gravity, ground collision via SVO voxel queries (design doc §9).
- No separate terrain collider mesh.

**Deliverables**

- Player spawns above terrain, lands, walks and jumps.

---

### Phase 7 — Block interaction

In `game`:

- Raycast from camera into SVO (DDA grid traversal).
- `BreakBlock` on left mouse; `PlaceBlock` on right mouse (adjacent empty cell).
- All changes through `WorldMutationQueue`.
- Crosshair overlay deferred; use screen-center ray for now.

**Deliverables**

- Break stone, place stone, see mesh update.

---

### Phase 8 — Terrain generation

Replace flat test floor with procedural terrain:

- `TerrainGen` system runs once at world init (or on chunk demand later).
- Simple 2D heightmap noise → grass top, dirt below, stone deep.
- Bounded world size for MVP (e.g. 256×256 horizontal).

**Deliverables**

- Varied hills; player spawns at safe height.
- Generation runs on main thread for MVP; IO/compute pool later.

---

### Phase 9 — Chickens + mounting

Core game fantasy. All logic in `game` crate.

**Components**

| Component | Data |
| --------- | ---- |
| `Chicken` | wander state, speed |
| `Mountable` | mount offset, rider slot |
| `Rider` | reference to mount entity |
| `Mounted` | reference to rider entity |

**Systems**

- `chicken_spawn_system` — scatter N chickens on grass at startup.
- `chicken_wander_system` — idle random walk, simple obstacle avoidance (ray or voxel step-up).
- `mount_system` — on interact key, if player within range of `Mountable` and no rider, attach `Rider`/`Mounted`, parent player transform to chicken.
- `dismount_system` — interact key while mounted; place player beside chicken.
- `mounted_movement_system` — player input steers chicken; boosted speed.

**Deliverables**

- Ride a chicken across generated terrain.
- Dismount and remount.

**Out of scope for this phase:** Chicken animations, breeding, inventory, combat.

---

### Phase 10 — Server binary (local)

Bring `server` up as headless authority:

- Same `game` systems registered (no render/input systems).
- `engine-net` stub: in-process channel or localhost QUIC (prefer QUIC if Phase 10 net stub is ready).
- Server owns authoritative SVO; client receives block deltas.

**Deliverables**

- `cargo run -p server` + `cargo run -p client` → two processes, one shared world.
- Single player connection works.

**MVP simplification:** Full prediction/reconciliation deferred to Phase 12; client may be dumb for now.

---

### Phase 11 — Render hardening

Align rendering with design doc §5:

- Dedicated render thread; extract snapshot on main, consume on render.
- Depth prepass → opaque → transparent → post → UI pipeline structure (post/UI can be passthrough initially).
- Replace CPU greedy meshing with GPU compute mesh generation.
- SVO-driven LOD selection (screen-space error); Transvoxel seams for LOD boundaries.

**Deliverables**

- No game ECS access from render thread.
- Mesh gen off main thread via compute pool.

---

### Phase 12 — Networking (QUIC)

Full client–server model per design doc §7:

- `quinn` transport; reliable streams for world load, datagrams for movement.
- Protocol version + `bincode` messages in `engine-net`.
- Server tick rate fixed; client prediction + reconciliation for player.
- Game systems emit events; net systems in `client`/`server` translate.

**Deliverables**

- Two clients + one server on LAN.
- Block breaks/places authoritative on server.

---

## Progress log

<!-- Append dated entries as phases complete. Example:
### 2026-06-10 — Phase 0 complete
- Workspace scaffolded, all crates compile.
-->

### 2026-06-10 — Phases 0–9 implemented

- Full Cargo workspace scaffolded per design doc §2.
- ECS scheduler, voxel world, block registry, CPU chunk meshing, wgpu client, and shared `game` systems through chicken mounting.
- Run with `cargo run -p client` (click to capture mouse; WASD, Space, E, mouse look, LMB/RMB blocks).

### 2026-06-10 — Phases 10–12 implemented

- **Phase 10:** `server` runs persistent 60 Hz tick loop with authoritative `game` systems.
- **Phase 11:** Rayon-parallel chunk mesh rebuild, camera-distance LOD culling, `extract_render_scene` snapshot before draw. Dedicated render thread + GPU compute meshing remain future work (macOS requires main-thread surface).
- **Phase 12:** `engine-net` QUIC (`quinn`) + bincode protocol; server broadcasts `BlockDeltas` and `EntitySnapshots`; client reconciles with prediction stub.
- **Multiplayer:** `cargo run -p server` then `CJ_SERVER=127.0.0.1:4242 cargo run -p client`.

---

## Explicit MVP shortcuts (must be removed later)

| Shortcut | Phase | Replaced in |
| -------- | ----- | ----------- |
| CPU greedy meshing | 5 | 11 |
| Sync asset load | 4 | 11+ |
| Main-thread terrain gen | 8 | Later |
| Extract on main thread (no render thread) | 5 | 11 |
| In-process / dumb client networking | 10 | 12 |

Do not let shortcuts leak into `game` or `engine-world` APIs. Isolate them inside `engine-render` or `client`/`server` wiring.

---

## Suggested first session (Phase 0 + 1)

A single focused session can complete:

1. Scaffold workspace and crates (Phase 0).
2. Implement minimal scheduler + headless server loop (Phase 1).

That produces a compiling repo with a ticking ECS — the foundation everything else attaches to.

---

## Open questions (resolve before Phase 9)

- **Camera while mounted:** third-person chase cam vs first-person on chicken?
- **Chicken count / biome rules:** fixed spawn count vs density-based?
- **World persistence:** needed for MVP or always fresh world?

Record decisions here when made; promote permanent gameplay rules to a future `docs/game-design.md` if needed.
