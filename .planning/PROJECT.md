# Qwik Optimizer OXC Port

## What This Is

A Rust-based Qwik optimizer using OXC (Oxide JavaScript Compiler) that replaces the existing SWC-based implementation. The optimizer extracts and transforms QRL (Qualified Runtime Literal) expressions for Qwik's lazy-loading architecture.

## Core Value

All 162 tests from qwik-core pass with exact output parity to the SWC implementation.

## Requirements

### Validated

- ✓ Core optimizer architecture — existing
- ✓ AST traversal and transformation — existing
- ✓ QRL extraction and component generation — existing
- ✓ NAPI bindings for Node.js consumption — existing
- ✓ 19 snapshot tests passing — existing

### Active

- [ ] OXC updated to latest version (0.111.0)
- [ ] Breaking changes from OXC update resolved
- [ ] Existing 19 tests still pass after update
- [ ] All 162 qwik-core snapshot tests ported
- [ ] Exact output parity with SWC implementation

### Out of Scope

- Production deployment — this milestone is test parity only
- Performance benchmarking — focus on correctness first
- New features beyond qwik-core parity — match existing behavior only

## Context

**Current state:**
- OXC version: 0.94.0 (17 minor versions behind latest 0.111.0)
- 19 snapshot tests passing in `optimizer/src/snapshots/`
- Reference implementation: `qwik-core/src/optimizer/core/` (SWC-based)
- Reference tests: 162 snapshots in `qwik-core/src/optimizer/core/src/snapshots/`

**Architecture:**
- Two-tier Rust architecture: core optimizer (`optimizer/`) + NAPI bindings (`napi/`)
- Visitor pattern via OXC's `Traverse` trait
- Entry strategy pattern for code extraction

**Reference materials:**
- OXC API docs: https://docs.rs/oxc/latest/oxc/
- qwik-core SWC implementation: `qwik-core/src/optimizer/core/src/`

## Constraints

- **Tech stack**: Must use OXC (not SWC) — this is the core purpose of the port
- **Output parity**: Generated code must match SWC output exactly to avoid production regressions
- **API stability**: OXC 0.111.0 may have breaking changes from 0.94.0

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Update OXC before porting tests | Access latest APIs, avoid rework on outdated patterns | — Pending |
| Target exact output parity | Prevents production regressions when replacing SWC optimizer | — Pending |

---
*Last updated: 2026-01-29 after initialization*
