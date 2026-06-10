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
