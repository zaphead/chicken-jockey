# Voxel Engine — Architecture & Engineering Standards

**Status:** Living Document

**Stack:** Rust · wgpu · winit · hecs · quinn

---

## 1. Design Principles

These principles are not preferences — they are constraints. Every architectural decision in this document follows from them, and every feature implementation must be accountable to them.

**No global mutable state.** All game state lives in the ECS as components or resources. No `static mut`, no `lazy_static` holding game data, no interior mutability as a workaround for not fitting something into the ECS.

**ECS-first, always.** Behavior lives in systems. Data lives in components. If you are writing game logic outside a system, something is wrong with your model.

**The server is not a special case.** The client and server share the same game logic crate. The only difference between the two binaries is which systems they register. If you write a system that only works in one environment, that's a code smell unless it's explicitly in a platform-specific crate.

**The renderer is a consumer, not a participant.** Rendering systems never write game state. They read a snapshot of the game world and produce frames. Coupling game logic to rendering is a hard architectural violation.

**Explicit over magic.** Systems are registered explicitly. Execution order is declared explicitly. There is no reflection-based auto-wiring. An engineer reading the startup code should be able to trace the full execution graph.

**Fail loudly in development.** Use `debug_assert!` liberally for invariants. Panicking early on a violated assumption is better than propagating corrupt state.

---

## 2. Repository Structure

The repository is a Cargo workspace. Each crate has a single, narrow responsibility. Features are composed by combining crates — not by growing individual crates.

```
/
├── crates/
│   ├── engine-core/       # ECS scheduler, event bus, math primitives, time
│   ├── engine-world/      # SVO, voxel/block types, world mutation API
│   ├── engine-render/     # wgpu renderer, pipeline, mesh generation
│   ├── engine-net/        # Protocol definitions, transport abstraction, serialization
│   ├── engine-audio/      # Audio abstraction (can be stubbed early)
│   ├── engine-input/      # Input abstraction, action mapping
│   ├── engine-assets/     # Asset handles, async loader, hot reload
│   └── game/              # All gameplay logic — shared between client and server
├── client/                # Client binary: wires render + audio + input + net client + game
└── server/                # Server binary: wires net server + world + game (headless)
```

**Dependency rules:**

* `game` may depend on `engine-core`, `engine-world`, `engine-net` (message types only), `engine-assets`.
* `game` must never depend on `engine-render` or `engine-audio`.
* `client` and `server` are the only places where crates are composed into a running application.
* Circular dependencies between engine crates are never acceptable.

---

## 3. ECS Architecture

The ECS is the backbone of the entire engine. `hecs` is used as the archetype storage layer. A custom scheduler wraps it to provide system ordering, parallelism, and the full execution model described below. Engineers implement features by writing systems and components — they do not interact with `hecs` directly outside of system queries.

### 3.1 Core Concepts

**Entity:** A unique identifier. Has no data or behavior of its own.

**Component:** A plain data struct. No methods that mutate self, no references to other components, no behavior. Components describe  *what an entity is* . If you find yourself putting game logic into a component, move it to a system.

**System:** A function that queries components and implements behavior. Systems declare their access (reads and writes) through their query types. Two systems with non-overlapping write sets can run in parallel — the scheduler enforces this automatically.

**Resource:** Globally unique data that does not belong to any specific entity. The world SVO, the asset server, the block registry, the active game time, and the network connection pool are all resources. Resources are accessed through the scheduler, not through globals.

**Command:** A deferred mutation — adding/removing entities, adding/removing components. Systems queue commands; commands are flushed at a defined synchronization point in the schedule. Systems must never add or remove entities/components inline during iteration.

**Event:** A single-frame pub/sub channel. Systems emit events; other systems consume them within the same tick. Events are not persisted across ticks. Using events is the correct way for systems to communicate — not direct function calls between systems.

### 3.2 The Schedule

The schedule is the execution graph. It is constructed at application startup and does not change at runtime. Each binary (`client`, `server`) constructs its own schedule by registering systems.

The schedule is divided into named  **stages** , executed in order every tick:

| Stage          | Purpose                                                     |
| -------------- | ----------------------------------------------------------- |
| `PreUpdate`  | Input polling, network receive, time update                 |
| `Update`     | Core game logic — the primary stage for gameplay systems   |
| `Physics`    | Fixed-timestep simulation; may run multiple times per frame |
| `PostUpdate` | Constraint solving, event cleanup, command flush            |
| `Extract`    | *(Client only)*Snapshot game state into render world        |
| `Render`     | *(Client only)*GPU command recording and submission         |

Within each stage, systems run in parallel where their access patterns allow. Dependencies between systems in the same stage are declared explicitly when ordering matters. Avoid over-constraining ordering — only declare a dependency when one system genuinely must see the output of another.

### 3.3 System Registration Contract

When adding a new system:

1. Identify which stage it belongs to.
2. Declare any explicit ordering dependencies within that stage.
3. If the system should only run under a condition, attach a run condition — do not put conditional logic at the top of the system body.
4. Decide whether it belongs in `game` (shared), `client`, or `server`.

---

## 4. World Representation

### 4.0 Coordinate system

World space uses **Z-up**:

* **Z** is vertical — gravity, jump, spectator fly up/down (Space / Ctrl).
* **XY** is the horizontal plane — terrain surfaces, WASD movement.
* `glam::Vec3` components map directly: `(x, y, z)` → world `(X, Y, Z)`.

All gameplay, physics, camera, and meshing code must use this convention. Shared helpers live in `game::axes`.

### 4.1 Sparse Voxel Octree (SVO)

The world is represented as a Sparse Voxel Octree. The SVO is a resource — there is one per running instance (server holds the authoritative SVO; client holds its own copy for local queries and rendering).

The octree subdivides world space recursively. At maximum depth, leaf nodes correspond to individual voxels. Interior nodes store aggregate material and density data representing the coarse LOD of their subtree. This structure provides LOD inherently — rendering at LOD level N means traversing the octree to depth (max_depth - N).

**Key properties the SVO must maintain:**

* Interior nodes are always valid aggregates of their children. Any mutation to a leaf must propagate upward through parent nodes before the frame is rendered.
* Empty regions are not allocated. Sparsity is the whole point — do not pre-allocate subtrees for air.
* The SVO is the source of truth for all world queries: collision, lighting, rendering, and gameplay logic all query the same structure.

### 4.2 World Mutation API

Systems must never mutate the SVO directly. All modifications go through a `WorldMutationQueue` resource. The queue collects mutations during `Update` and flushes them in `PostUpdate`, propagating parent-node aggregates and dispatching change events that other systems (renderer, physics, etc.) can subscribe to.

This indirection exists for two reasons: it makes mutations safe to issue from parallel systems, and it ensures dependent systems always see a consistent world state within a given frame.

### 4.3 Block Registry

Voxel/block types are defined in data files (not hardcoded) and loaded into a `BlockRegistry` resource at startup. Systems that need to know about block properties query the registry by block ID. Hard-coding block behavior by ID in game logic is not permitted — use registry lookups.

---

## 5. Rendering Architecture

The renderer runs on a dedicated render thread. It never touches the game world ECS directly. The boundary between game logic and rendering is the  **extract phase** .

### 5.1 Extract Phase

Once per frame, at the `Extract` stage, the main thread snapshots all renderable game state into a separate, renderer-owned data structure (the  *render world* ). This is a one-way copy: game → render. After extraction, the game thread continues into the next tick while the render thread processes the snapshot asynchronously.

**Rules for extraction:**

* Only components tagged as renderable are extracted.
* Extraction systems are in `engine-render` and run on the main thread during `Extract`. They are the only systems permitted to write to render world data.
* No game logic system may read from or write to the render world.

### 5.2 Pipeline Stages

The render thread executes these stages each frame, in order:

1. **Prepare** — Upload extracted data to GPU (vertex buffers, uniform buffers, texture updates from the asset system).
2. **Depth Prepass** — Render scene depth only. Enables early-z rejection for opaque geometry.
3. **Opaque Geometry** — Render fully opaque voxel meshes.
4. **Cutout Geometry** — Alpha-tested meshes (leaves, flora) with depth write.
5. **Transparent Geometry** — Render alpha-blended geometry, sorted back-to-front.
6. **Post-Processing** — Screen-space effects (ambient occlusion, bloom, tone mapping).
7. **UI** — Rendered last, on top of everything, in screen space.

All rendering is done through `wgpu`. No platform-specific graphics API calls appear outside of `engine-render`. Pipeline state objects are created at startup and reused — avoid creating pipelines at runtime.

### 5.3 Level of Detail

LOD is driven by the SVO structure and a screen-space error metric computed per octree node. The renderer traverses the SVO each frame, selecting the depth at which each subtree should be rendered based on its projected size in screen space. This produces a per-region LOD level naturally.

LOD zones are managed as concentric clipmaps around the camera. Moving between clipmap levels triggers mesh regeneration for the transition region.

Seams between adjacent LOD levels are resolved using the Transvoxel algorithm. The mesh generator is responsible for producing correct transition cells wherever two adjacent regions are at different LOD levels. Systems that modify the world must be aware that a change at high-detail LOD may require regenerating transition meshes in neighboring regions at coarser LOD.

### 5.4 Mesh Generation

**Current MVP:** chunk meshes are built on the CPU during the Extract stage (`mesh_chunk` in `engine-render`). Material resolution (UV, draw category, tint, CTM) runs in the mesher, not at draw time.

**Future:** GPU compute mesh generation may replace CPU extraction. The material resolution API must remain callable from extract workers; draw passes must not read game ECS or SVO data.

See [`material-engine.md`](./material-engine.md) for block material tables and milestone gates.

---

## 6. Threading Model

| Thread       | Owns                                 | Rules                                                                       |
| ------------ | ------------------------------------ | --------------------------------------------------------------------------- |
| Main         | ECS scheduler, windowing, input      | Runs game systems. Is the only thread that may flush commands into the ECS. |
| Render       | wgpu device, render world            | Reads render world (written during Extract). Never reads game ECS.          |
| IO Pool      | Asset loading, file reads/writes     | Communicates results to main thread via channels. No direct ECS access.     |
| Compute Pool | Mesh generation jobs, heavy CPU work | Receives work via channels, returns results via channels. No ECS access.    |

Within the main thread, the scheduler parallelizes systems across worker threads drawn from a thread pool. These workers operate on ECS queries for the duration of a stage. They are coordinated by the scheduler — engineers do not manage worker threads manually.

**The general rule:** data flows between threads through channels and snapshot copies (like the render world extract). Shared mutable state between threads is not permitted outside of the internal scheduler machinery.

---

## 7. Networking & Client–Server Model

### 7.1 Authority Model

The server is the authoritative source of truth for all world state. The client maintains a local simulation for responsiveness (input prediction) but defers to server corrections on conflict. This means:

* The server runs the same ECS systems as the client for game logic (they live in `game`).
* The server runs additional systems for session management, world authority enforcement, and state diffing.
* The client runs additional systems for input prediction and server reconciliation.

### 7.2 Transport

The network transport is QUIC, via the `quinn` crate. QUIC provides reliable ordered streams (used for world loading, chat, state that must arrive in order) and unreliable datagrams (used for real-time position updates and input packets) over a single connection. The choice of stream type per message category is defined in `engine-net`.

### 7.3 Message Model

All network message types are defined in `engine-net`. Game systems in `game` never call into networking directly — they emit game events, and dedicated networking systems in `client` or `server` translate between game events and network messages. This keeps `game` free of any networking concern.

Message serialization uses `bincode`. The wire format is always versioned. Breaking changes to message types require a protocol version bump.

### 7.4 Server Tick vs. Client Frame

Three operational rates exist; only the first is authoritative for gameplay:

| Clock | Rate | Owner |
| ----- | ---- | ----- |
| **Sim tick** | 60 Hz (`SIM_HZ` in `engine-core`) | Server; client matches via fixed steps |
| **Render frame** | Variable (display refresh) | Client only |
| **Net send/recv** | Per render frame today (PreUpdate); may throttle later | Client/server binaries |

The server runs a fixed-rate sim tick loop independent of any render loop. The client accumulates wall-clock frame time and drains zero or more fixed sim steps per render frame (Glenn Fiedler accumulator). Rendering never drives gameplay math.

**Client stage contract (each render frame):**

1. **Once:** poll input → `PreUpdate` (assets, input sync, local look, spectator, net).
2. **0–N times:** `advance_fixed()` → `Update` → `Physics` → `PostUpdate` → `end_frame()`.
3. **Once:** set interpolation alpha → `Extract` → `Render`.

Mouse look is applied once per render frame in client `PreUpdate`. Movement and physics integrate with `fixed_delta` each sim step. Extract interpolates the local player camera between previous and current sim poses using `interpolation_alpha`.

**Subsystem divisors:** slower logic (future circuits/electricity, crop-style random ticks) runs on `sim_tick % N == 0` via scheduler run conditions—not a separate clock. Example: `N = 3` ≈ 20 Hz from a 60 Hz base. Circuit propagation should use explicit produce/consume stages and events, never render or implicit neighbor order.

Network tick indices on wire messages are deferred until reconciliation/rollback work; sim tick is still the in-process authority counter today.

---

## 8. Asset System

Assets are loaded asynchronously via the IO thread pool. Every asset is accessed through a typed `Handle<T>`. Handles are cheap to clone and can be stored in components. The underlying data behind a handle may not be available immediately after the handle is created — systems that use asset data must check for readiness.

The `AssetServer` resource manages loading and caching. In development builds, the asset server watches the filesystem and hot-reloads changed assets. Asset types include textures, audio files, shaders, and block definition data files.

Shaders are compiled at startup from source. There is no runtime shader compilation during gameplay. Shader permutations (for different material types, pipeline variants) are compiled as distinct pipeline state objects.

### 8.1 Block materials

Block appearance is defined in data (`assets/blocks/*.toml` + texture folders) and packed at startup into `ResolvedBlockMaterials` (atlas + per-face lookup tables). Draw category, overlays, state variants, biome tint, animation, and connected textures are documented in [`material-engine.md`](./material-engine.md).

---

## 9. Physics

Physics runs as a fixed-timestep system in the `Physics` stage. It is implemented entirely as ECS systems operating on physics-relevant components (`RigidBody`, `Collider`, `Velocity`, etc.).

Collision against the voxel world is done by querying the SVO for occupied voxels in the relevant region. There is no separate physics representation of the terrain — the SVO is the source of truth. This means physics code must go through the same query API as all other world-reading code.

The physics stage runs after `Update` so that gameplay systems can set velocities and positions this tick and have them resolved before the next tick's game logic runs.

---

## 10. Feature Implementation Contract

This section defines the rules that govern all feature work. When implementing any feature, an engineer must be able to answer each of the following questions affirmatively.

**State:** Is all new game state represented as ECS components or resources? If you introduced a struct that holds state outside of these, it must be justified and documented.

**Behavior:** Is all new game behavior implemented as ECS systems registered in the correct stage? If game logic is running outside of systems, it is a bug.

**World modification:** Do all voxel world changes go through `WorldMutationQueue`? Direct SVO mutation from a system is not permitted.

**Client/server split:** Does shared gameplay logic live in `game`? Does the feature compile and run correctly as a headless server (no rendering, no audio, no windowing)?

**Rendering isolation:** Do rendering systems only read from the render world, never from the game ECS? Does the feature introduce no coupling between game logic and rendering?

**Asset loading:** Are all new assets loaded through the `AssetServer` using typed handles? Is the loading asynchronous and non-blocking on the game thread?

**Events vs. direct calls:** Does inter-system communication use events? Two systems should not have a direct call relationship.

**Block types:** Are new voxel/block types defined in data files and registered through the `BlockRegistry`? No block behavior is hardcoded by numeric ID.

**LOD correctness:** If the feature produces renderable geometry, does it behave correctly at all LOD levels, including transition cells?

**Thread safety:** Does any new data structure crossed between threads go through a channel or be part of an explicit snapshot? No new shared mutable state between threads.

---

## 11. What This Document Does Not Specify

The following are intentionally left to implementation:

* Specific data layouts, field types, or struct sizes. These are implementation details.
* Specific tick rates, view distances, or tuning parameters. These are operational parameters.
* Specific algorithms for gameplay features (pathfinding, crafting, etc.). These are feature-level decisions that must conform to the architecture above but are not prescribed here.
* UI framework. The constraint is that UI rendering happens in the final render stage on the client. The specific library or approach is an implementation decision.
* Persistence format. The constraint is that world persistence reads and writes through the IO thread pool and never blocks the main thread.

When an implementation decision is not covered by this document, the guiding question is: *does this choice make the system harder to reason about, harder to parallelize, or harder to run headlessly?* If yes, reconsider.
