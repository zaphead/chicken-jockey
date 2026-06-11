# OpenCraft — Architecture Fundamentals

**Status:** Living Document

**Stack:** Rust · wgpu · winit · hecs · quinn

This document defines **what must stay true** about the system — crate boundaries, ECS shape, world authority, and render isolation. It is not a build spec. Phased deliverables, pipeline details, and tuning live in [`implementation-plan.md`](./implementation-plan.md). Day-to-day engineering rules for agents and contributors live in [`AGENTS.md`](../AGENTS.md).

---

## 1. Engineering Principles

These are constraints, not preferences. When a requirement conflicts with them, simplify or delete the requirement — do not bend the architecture.

### Subtractive engineering

Before adding code, ask whether the thing driving the change is still necessary. Prefer **delete → simplify → add minimum**. Do not preserve legacy paths, compatibility shims, or defensive layers around a bad prior decision. One correct implementation beats stacked guards.

### Core constraints

**No global mutable state.** Game state lives in the ECS as components or resources.

**ECS-first.** Behavior in systems; data in components. Game logic outside systems is a smell.

**The server is not a special case.** Client and server share the `game` crate. The only difference is which systems each binary registers.

**The renderer is a consumer.** Rendering reads a snapshot; it never writes game state or queries gameplay ECS during draw.

**Explicit over magic.** Systems, stages, and dependencies are registered explicitly — no reflection or hidden wiring.

**Fail loudly in development.** Use `debug_assert!` for invariants. Panic on violated assumptions rather than propagate corrupt state.

---

## 2. Repository Structure

Cargo workspace. One responsibility per crate; features compose at the binary layer.

```
/
├── crates/
│   ├── engine-core/       # ECS scheduler, events, time, math
│   ├── engine-world/      # SVO, block types, world mutation API
│   ├── engine-render/     # wgpu renderer, pipelines, mesh types
│   ├── engine-net/        # Protocol types, transport
│   ├── engine-audio/      # Audio abstraction
│   ├── engine-input/      # Input abstraction, action mapping
│   ├── engine-assets/     # Handles, async loader
│   └── game/              # Gameplay logic — shared by client and server
├── client/                # Wires render, input, audio, net client, game
└── server/                # Wires net server, world, game (headless)
```

**Dependency rules:**

* `game` may depend on `engine-core`, `engine-world`, `engine-net` (message types only), `engine-assets`.
* `game` must not depend on `engine-render`, `engine-audio`, or `engine-input`.
* `client` and `server` are the only composition roots.
* No circular dependencies between engine crates.

---

## 3. ECS

`hecs` stores entities; a custom scheduler owns ordering, parallelism, and resources. Feature work adds systems and components — not ad-hoc calls outside the schedule.

**Entity** — ID only. **Component** — plain data. **System** — behavior via queries. **Resource** — global singleton data (SVO, registries, time). **Command** — deferred structural changes; flush at a defined sync point, never during query iteration. **Event** — single-frame pub/sub within a tick.

### Schedule stages

Constructed at startup; unchanged at runtime. Each binary registers its own systems.

| Stage        | Purpose |
| ------------ | ------- |
| `PreUpdate`  | Input, network receive, time |
| `Update`     | Core gameplay |
| `Physics`    | Fixed-timestep simulation (may run multiple times per frame) |
| `PostUpdate` | Command flush, net send, cleanup |
| `Extract`    | *(Client)* Snapshot game state for rendering |
| `Render`     | *(Client)* GPU work |

Run conditions belong at registration time — not `if mode` branches inside shared systems.

---

## 4. World

**Coordinates:** Z-up (Z vertical, XY horizontal). Shared helpers in `game::axes`.

**SVO:** Authoritative sparse voxel octree. One instance per running process (server authoritative; client holds a copy for local use). Empty space is not allocated. Collision, gameplay, and rendering query the same structure.

**Mutations:** Systems never mutate the SVO directly. All edits go through `WorldMutationQueue`, flush in `PostUpdate`, emit change events.

**Blocks:** Types defined in data files, loaded into `BlockRegistry`. Game logic resolves behavior through the registry — not hardcoded numeric IDs.

---

## 5. Rendering Boundary

Game logic and rendering are separated by an **extract snapshot** (`RenderWorld` or equivalent):

* Extract runs in the client schedule; it copies renderable state from the game world.
* `engine-render` consumes the snapshot only — no gameplay ECS queries during draw.
* Platform threading details (main-thread surface, submit workers, compute meshing) are implementation choices documented in the implementation plan, not fixed here.

Visual/material specifics: [`material-engine.md`](./material-engine.md).

---

## 6. Threading

| Concern        | Rule |
| -------------- | ---- |
| Game simulation | Main thread runs the ECS schedule |
| Rendering      | Reads render snapshot; no game ECS access |
| IO / assets    | Async pool; results arrive on main via channels |
| Heavy CPU work | Job pools via channels; no direct ECS access |

Cross-thread communication uses channels or explicit snapshots — not shared mutable game state.

---

## 7. Networking

**Authority:** Server owns world truth. Client predicts for responsiveness and reconciles on server correction.

**Transport:** QUIC (`quinn`). Message types live in `engine-net`. `game` emits intents/events; `client` and `server` net systems translate wire ↔ resources — `game` never calls transport directly.

**Clocks:** Fixed sim tick rate is authoritative for gameplay. Render frame rate is client-only and must not drive simulation math. Wire timing and prediction details belong in the implementation plan.

---

## 8. Assets & Physics

**Assets:** Typed `Handle<T>`, async load via `AssetServer`, non-blocking on the sim thread. Shaders and pipeline variants are prepared at startup, not compiled mid-gameplay.

**Physics:** Fixed-timestep systems in the `Physics` stage. Voxel collision queries the SVO — no duplicate terrain collision mesh.

---

## 9. What This Document Does Not Specify

Intentionally out of scope here:

* Render pass order, shadow resolution, post-processing stack, LOD algorithms
* Tick rates, view distance, tuning constants
* Gameplay feature algorithms (crafting, pathfinding, UI framework choice)
* Persistence format (constraint: must not block the sim thread)
* Specific struct layouts and field types

When unsure, ask: *does this choice make the system harder to reason about, parallelize, or run headlessly?* If yes, reconsider. Otherwise decide in the implementation plan or feature PR and move on.
