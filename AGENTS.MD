# Chicken Jockey — Agent Instructions

## Read before code

**Always read [`docs/design-doc.md`](./docs/design-doc.md) before inspecting or modifying any source.** It is the source of truth for architecture, constraints, and engineering standards. Every change must be accountable to it.

Then read [`docs/implementation-plan.md`](./docs/implementation-plan.md) for the current build phase, active milestones, and what is in or out of scope.

Do not explore the codebase or write code until both documents are understood and your task is mapped to a phase in the implementation plan.

## Document hierarchy

| Document | Role |
| -------- | ---- |
| `docs/design-doc.md` | Architecture and engineering constraints (immutable principles) |
| `docs/implementation-plan.md` | Phased build plan and milestone tracking (living document) |

If implementation plan guidance conflicts with the design doc, the design doc wins. Update the implementation plan — do not bend architecture to match a shortcut.

## When completing work

- Update the implementation plan phase status and notes when a milestone is reached or scope shifts.
- Do not add architecture decisions to this file; put them in the design doc (if permanent) or implementation plan (if phased).

---

## Engineering rules (architecture audit)

These rules encode lessons from the thermo-nuclear code-quality audit and the design-doc hardening pass. They are the delta between what the codebase *used to do wrong* and what it must do now. Follow them on every change so we do not reintroduce the same debt.

### Crate boundaries

| Wrong | Right |
| ----- | ----- |
| `game` depends on `engine-render`, `engine-audio`, or `engine-input` | `game` depends only on `engine-core`, `engine-world`, `engine-assets`, and `engine-net` **message types** |
| Gameplay logic in `client/main.rs` or `server/main.rs` | Gameplay in `game` systems; binaries only wire the schedule |
| Render or net orchestration in `main.rs` | Thin `main.rs` + systems in `client/src/systems/` or `server/src/systems/` |
| `engine-render` reads `hecs::World` or game components during draw | Renderer consumes `RenderWorld` snapshot only; extract writes the snapshot |

### Binary responsibilities

**`client/main.rs`** must stay a winit loop: window lifecycle, input event forwarding, `run_client_frame()`, renderer creation. It must not own mesh caches, chunk queues, block-change drains, net packet handling, or terrain orchestration.

**`server/main.rs`** must stay a tick loop: fixed timestep, schedule tick, sleep. No gameplay branches inline.

**Client-only systems** (register in `client/src/systems/mod.rs`):

- `PreUpdate` — map `engine-input` → `GameplayInput`, client net receive
- `Extract` — drain `BlockChanged`, queue/rebuild chunk meshes, write `RenderWorld`
- `Render` — upload meshes and present from `RenderWorld`

**Server-only systems** (register in `server/src/systems/mod.rs`):

- `PreUpdate` — net receive → `PlayerInputs`
- `PostUpdate` — `BlockChanged` / entity state → wire packets

### ECS and systems

| Wrong | Right |
| ----- | ----- |
| `SimulationMode` enum with `if mode == …` inside game systems | Marker resources (`NetworkClient`, `AuthoritativeServer`) + **run conditions** at registration time |
| `if !condition { return; }` at the top of a system for binary-specific behavior | `app.add_system_with_condition(stage, system, run_if)` in the binary that owns the behavior |
| `insert_one` / `remove_one` / `spawn` during query iteration | Queue structural changes through `ctx.commands`; flush at `PostUpdate` |
| Duplicate input resolution in player, mount, block systems | Single `resolve_input(ctx, net_id)` in `game/src/input.rs` |
| `.query::<&Player>().next()` for “the” player in multiplayer | Lookup by `LocalPlayerId` or `NetPlayerId` |
| Duplicated AABB collision in player, chicken, mount | Shared `collides_aabb` in `game/src/systems/physics/collision.rs` |
| One `spawn_player_system` branching on mode | `spawn_local_player_system` and `spawn_network_player_system` with run conditions |
| One `register_game_systems` for all binaries | `register_shared_systems`, `register_player_systems`, `register_local_client_systems`, `register_network_client_systems`, `register_server_systems` |

### Input boundary

| Wrong | Right |
| ----- | ----- |
| `game` imports `engine-input` or `InputState` | `GameplayInput` + `PlayerInputs` + `LocalPlayerId` live in `game/src/input.rs` |
| Winit or `InputState` in game systems | `client` maps `InputState` → `GameplayInput` in `PreUpdate`; `server` maps `PlayerInput` (net) → `GameplayInput` in `PreUpdate` |

### World and assets

| Wrong | Right |
| ----- | ----- |
| Direct `svo.set_block` from gameplay systems | All edits through `WorldMutationQueue`; flush in `PostUpdate`; consume `BlockChanged` events |
| HashMap pretending to be an SVO | Pointer octree with upward aggregate propagation (`engine-world/src/svo.rs`) |
| Magic block IDs in game logic | `BlockRegistry` lookups by name/ID from data files |
| Sync `load_block_registry` blocking the hot path without a fallback | `AssetServer` async IO for loads; **bootstrap `BlockRegistry` synchronously at startup** so terrain and physics are not blocked on the first frames |
| Game systems running before `BlockRegistry` exists with no recovery | Ensure registry is present before systems that need it, or gate those systems with a run condition |

### Rendering

| Wrong | Right |
| ----- | ----- |
| `Renderer::render(&app)` or passing `&World` into `engine-render` | `RenderWorld { camera, meshes, ready }` written in `Extract`, read in `Render` |
| Dead or broken `RenderThread` owning the winit `Surface` on macOS | **Main thread:** `get_current_texture`, `present`. **Render worker:** encode passes + `queue.submit` only |
| `queue.submit` from rayon workers, compute mesh jobs, and render thread concurrently | **One owner** for GPU submission (render-submit thread); no wgpu calls from parallel mesh workers |
| Synchronous CPU terrain meshing as the only path without LOD | Screen-space LOD step per chunk + seam fix at boundaries; compute path must not block or race the render queue |
| Gating `ecs.tick()` on renderer existence | Game simulation ticks even before GPU init; only `Render` stage needs `ClientRenderer` |

### Networking

| Wrong | Right |
| ----- | ----- |
| `game` calling `NetClient` / `NetServer` directly | `game` emits events (`BlockChangeIntent`, etc.); `client`/`server` net systems translate wire ↔ resources |
| All packets on QUIC bi-streams | **Reliable streams:** `Join`, `Welcome`, `BlockDeltas`. **Datagrams:** `Input`, `EntitySnapshots` |
| Client applies server snapshots with hard teleport only | Predict locally; reconcile on snapshot with error threshold + velocity adjustment |
| Network client runs `block_interaction_system` | Authoritative blocks from server deltas only; local client registers block interaction |

### Plugin registration matrix

Use this when adding or moving systems:

| Binary | Registers |
| ------ | --------- |
| Shared (`register_shared_systems`) | terrain, chickens, mount, mounted movement/physics, mutation flush |
| Server (`register_server_systems`) | shared + player systems + `block_interaction` |
| Local client (`register_local_client_systems`) | shared + player systems + `block_interaction` |
| Network client (`register_network_client_systems`) | shared + player systems only (no local block authority) |

### Schedule stages (do not invent ad-hoc per-frame calls)

```
PreUpdate → Update → Physics → PostUpdate → Extract → Render
```

- Input and net **receive** in `PreUpdate`.
- Net **send/broadcast** in `PostUpdate` (after authoritative tick).
- Render snapshot in `Extract`; GPU present in `Render`.
- Do not call net drain/send or mesh rebuild from `main` between frames.

### Code smell checklist

Before submitting a change, verify:

1. **`game` compiles without `engine-render`, `engine-audio`, `engine-input`.**
2. **No new `if SimulationMode` / mode enum branching in `game`.**
3. **No structural ECS mutation inside system iteration** (use `Commands`).
4. **`client/main.rs` did not grow** — new client wiring goes in `client/src/systems/`.
5. **Renderer path does not query game ECS.**
6. **World edits go through `WorldMutationQueue`.**
7. **Multiplayer player lookup is keyed by net ID, not “first query result”.**
8. **GPU work is not submitted from rayon or arbitrary threads.**

### Where to put new code

| Kind of change | Location |
| -------------- | -------- |
| Shared gameplay behavior | `crates/game/src/systems/` |
| Shared components/resources | `crates/game/src/components.rs` or new module in `game` |
| Platform input (winit) | `engine-input` + mapping in `client/src/systems/input.rs` |
| Extract / present / client net | `client/src/systems/` |
| Server net | `server/src/systems/` |
| Protocol types / transport | `crates/engine-net/` |
| SVO / mutations | `crates/engine-world/` |
| Mesh / pipeline / `RenderWorld` types | `crates/engine-render/` (extract **systems** that touch game ECS stay in `client`) |

When in doubt, ask: *can the server run this headlessly?* If yes, it belongs in `game` or `engine-*`, not in `client`.

### Diagnosing client issues (no visual inspection)

Before guessing about blue screens or missing terrain, run automated checks:

```bash
bash scripts/diagnose-client.sh          # headless + live log sample → /tmp/chicken-jockey-diagnose.txt
cargo run --bin client-diagnose          # quick headless pipeline only (exit 0 = DIAGNOSE_OK)
cargo test -p client headless_pipeline   # integration test
```

Live client logs (`CJ_DIAGNOSTIC=1 RUST_LOG=info cargo run --bin client`) emit `cj diag:` lines each second with registry, terrain, mesh, and vertex counts. **GPU encoding must run on the main thread on macOS** — do not submit wgpu command buffers from a background render thread.
