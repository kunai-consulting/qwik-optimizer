# Project Research Summary

**Project:** Qwik Optimizer OXC Migration (0.94.0 to 0.111.0)
**Domain:** Compiler/Transformer Toolchain Migration
**Researched:** 2026-01-29
**Confidence:** HIGH

## Executive Summary

This project migrates the Qwik optimizer from OXC 0.94.0 to 0.111.0 while achieving feature parity with the production SWC implementation. The Qwik optimizer is a critical AST transformation tool that extracts QRL (Qwik Resource Locator) functions into separate modules, enabling lazy-loading and resumability for Qwik framework applications.

The recommended approach is incremental: first update OXC dependencies and fix breaking API changes (primarily in semantic analysis and traverse APIs), then systematically port the 143 missing test cases organized by feature category. The current OXC implementation covers only 19 of 162 snapshot tests from the reference SWC implementation, with basic QRL extraction working but critical features like event handlers, component transformation, props handling, and source maps missing.

Key risks include hash generation divergence (which breaks runtime lazy-loading), codegen output format differences (causing snapshot test failures), and semantic scope mismatches (leading to incorrect variable capture). These can be mitigated by normalizing hash inputs, defining parity as semantic rather than textual equivalence, and thoroughly studying OXC's semantic analysis model. The migration is well-documented but requires careful attention to behavioral differences between SWC's fold-based transformation and OXC's traverse-based approach.

## Key Findings

### Recommended Stack

The project continues using Rust-based OXC toolchain with targeted version updates across 8 crates. The migration from 0.94.0 to 0.111.0 introduces several breaking changes but the codebase is already partially compatible with many new APIs.

**Core technologies:**
- **OXC 0.111.0** (parser, AST, semantic, traverse, codegen) - Target version with breaking changes in Scoping API and traverse patterns
- **oxc_semantic** - Combined Scoping API (replaces separate SymbolTable/ScopeTree) for symbol resolution
- **oxc_traverse** - Mutable traversal with State type parameter (codebase already using `TraverseCtx<'a, ()>`)
- **oxc_allocator** - Arena-based allocation (increased chunk size from 512B to 16KB affects memory usage)
- **oxc_transformer** - TypeScript/JSX transformation (takes `&TransformOptions`, already compatible)

**Critical version requirements:**
- All OXC crates must update to 0.111.0 together (breaking changes span multiple crates)
- `oxc_index` stays at 4.1.0 (no changes needed)
- Incremental update strategy recommended (0.94 -> 0.100 -> 0.105 -> 0.111) to isolate breakage

### Expected Features

The feature landscape is defined by 162 snapshot tests in the SWC reference implementation. Current coverage is 19 tests (12%), leaving 143 tests to port across 10 major categories.

**Must have (table stakes):**
- **Component transformation** (5 tests) - `component$()` wrappers, essential for every Qwik app
- **Event handlers** (8 tests) - `onClick$` etc., required for interactive components
- **Props handling** (24 tests) - Signal forwarding, destructuring, essential for component authoring
- **JSX transformation** (6 remaining tests) - Keyed elements, spread props, custom import sources
- **Signals/Stores** (8 tests) - Core reactivity system, high priority
- **Import/Export** (10 tests) - Module system completeness, dynamic imports, default exports
- **Remaining QRL extraction** (4 tests) - Nested QRLs, index parameters, complex contexts

**Should have (competitive):**
- **Server/Client code handling** (11 tests) - SSR/SSG support, strip server-only code
- **Entry strategies** (7 tests) - Inline, manual chunks, preserve filenames (affects build output)
- **Spread props & JSX split** (8 tests) - Common in component libraries
- **Bind directives** (8 tests) - Form handling (`bind:value`, `bind:checked`)
- **Dev/Prod mode** (4 tests) - DX features, different output for debugging
- **TypeScript** (5 remaining tests) - Enums, generator types, completeness

**Defer (v2+):**
- **Immutability analysis** (3 tests) - Performance optimization, not critical for correctness
- **Edge cases** (12 tests) - Bug fixes for specific issues (117, 150, 476, 964, etc.)
- **Miscellaneous** (30+ tests) - Qwik React integration, custom inlined functions, advanced features

### Architecture Approach

The migration maintains architectural alignment with the SWC implementation but adapts to OXC's different traversal patterns. SWC uses an immutable fold-based approach while OXC uses mutable in-place traversal, requiring different state management strategies.

**Major components:**
1. **TransformGenerator (transform.rs)** - Main orchestration via traverse trait, replaces SWC's QwikTransform fold
2. **Component generation (component/)** - QrlComponent, Qrl struct, hash/ID generation, segment building
3. **Segment extraction (segment.rs)** - SegmentBuilder creates extractable code units, manages naming and hierarchy
4. **Entry strategy (entry_strategy.rs)** - Controls code-splitting behavior (Segment/Inline/Component/Smart modes)
5. **Import cleanup (import_clean_up.rs)** - Normalizes imports post-transformation, uses BTreeSet for ordering
6. **Source/semantic** - SourceInfo tracking, semantic analysis for scope/symbol resolution

**Key patterns:**
- **Arena allocation** - All AST nodes allocated via `Allocator`, requiring careful lifetime management
- **Traverse-based mutation** - Enter/exit methods modify AST in-place vs SWC's immutable fold
- **Semantic-first** - Build semantic model before transformation (OXC pattern vs SWC's hygiene marks)
- **Component separation** - QRL data structures isolated in `component/` module for modularity

### Critical Pitfalls

1. **OXC Breaking API Changes** - `Scoping::scope_build_child_ids` removed in 0.111.0, new `Ident` type added, TSEnum scope moved. Prevention: incremental updates with full test runs after each version bump, read changelogs thoroughly.

2. **Codegen Output Format Differences** - Arrow function spacing (`()=>` vs `() =>`), import hoisting patterns, JSX handling differ between SWC and OXC. Prevention: define parity as semantic equivalence not textual, create format-normalized comparison tooling.

3. **Hash Generation Divergence** - Different formatting in codegen leads to different hash inputs, breaking runtime lazy-loading. Prevention: normalize/canonicalize strings before hashing, use identical algorithm (DefaultHasher), verify platform-independent path handling.

4. **Source Map Generation Missing** - OXC implementation returns `None` for source maps while SWC generates full mappings. Prevention: integrate `oxc_sourcemap` crate, pass options to codegen, collect mappings during transformation.

5. **Semantic Scope Mismatch** - SWC uses SyntaxContext-based Id tuples, OXC uses SymbolId/ReferenceId with different traversal semantics. Prevention: study `oxc_semantic` thoroughly, write explicit capture detection tests, verify `useLexicalScope` parameters.

## Implications for Roadmap

Based on research, suggested phase structure follows a dependencies-first approach: fix foundation (OXC update), build core functionality (components/events/props), add build features (SSR/strategies), then polish edge cases.

### Phase 1: OXC Dependency Update & API Fixes
**Rationale:** Must establish stable foundation before porting features. Breaking changes in Scoping, traverse, and AST APIs affect all subsequent work.
**Delivers:** Compiling codebase on OXC 0.111.0 with all current tests passing
**Addresses:** Incremental version bumps (0.94 -> 0.100 -> 0.105 -> 0.111)
**Avoids:** Pitfall 1 (Breaking API changes) - Catching issues early prevents cascading failures
**Research needed:** NO - Well-documented in STACK.md, clear migration path

### Phase 2: Core QRL & Component Features
**Rationale:** Component transformation, event handlers, and props are interdependent and form the foundation for all Qwik apps.
**Delivers:** Full component lifecycle support, interactive components, signal forwarding
**Addresses:** Component transformation (5 tests), Event handlers (8 tests), Props handling (24 tests), Remaining QRL extraction (4 tests)
**Avoids:** Pitfall 5 (Semantic scope mismatch) - Core functionality exercises semantic model thoroughly
**Research needed:** MAYBE - Props destructuring logic may need deeper investigation if test failures arise

### Phase 3: Signals & Reactivity
**Rationale:** Signals are core to Qwik's reactivity model and depend on props handling from Phase 2.
**Delivers:** Full reactive signal support, store expressions, derived signals
**Addresses:** Signals/Stores (8 tests)
**Avoids:** Integration issues with component props from Phase 2
**Research needed:** NO - Standard patterns, well-covered in FEATURES.md

### Phase 4: JSX & Import/Export Completeness
**Rationale:** JSX edge cases and module system features are table stakes but don't block core functionality.
**Delivers:** Complete JSX support, full module resolution, TypeScript completeness
**Addresses:** JSX remaining (6 tests), Import/Export (10 tests), TypeScript (5 tests)
**Avoids:** Pitfall 7 (JSX runtime naming) - Aligning function names early prevents runtime breaks
**Research needed:** NO - Patterns documented in reference implementation

### Phase 5: Build System Features
**Rationale:** SSR, entry strategies, and code-splitting control affect build output but not transformation correctness.
**Delivers:** Server/client mode, entry strategies, dev/prod modes
**Addresses:** Server/Client (11 tests), Entry strategies (7 tests), Dev/Prod mode (4 tests)
**Avoids:** Pitfall 4 (Source maps) - Must implement for production debugging
**Research needed:** YES - Source map generation needs `oxc_sourcemap` integration research

### Phase 6: Advanced Features & Polish
**Rationale:** Spread props, bind directives, and edge cases provide completeness but aren't critical path.
**Delivers:** Component library patterns, form handling, bug fixes
**Addresses:** Spread props (8 tests), Bind directives (8 tests), Edge cases (12 tests)
**Avoids:** Pitfall 6 (Import ordering) - Standardize early to prevent snapshot churn
**Research needed:** NO - Edge cases are isolated, low interdependency

### Phase 7: Optimization & v2 Features
**Rationale:** Immutability analysis and miscellaneous features are performance optimizations, defer to post-parity.
**Delivers:** Performance optimizations, Qwik React integration, advanced patterns
**Addresses:** Immutability (3 tests), Miscellaneous (30+ tests)
**Avoids:** Pitfall 8 (Memory pressure) - Profile and optimize allocator usage
**Research needed:** YES - Memory profiling and allocator reuse patterns need investigation

### Phase Ordering Rationale

- **Dependencies-first:** OXC update blocks everything; component/props/events are interdependent and form core
- **Feature clusters:** Group related tests (all props together, all JSX together) to avoid context switching
- **Risk mitigation:** Address hash divergence and scope tracking in Phase 2 when test volume is manageable
- **Build features late:** SSR and entry strategies don't affect correctness, can iterate separately
- **Defer optimization:** Memory and performance tuning comes after correctness established

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (Build System)** - Source map generation with `oxc_sourcemap` lacks examples, may need trial-and-error
- **Phase 7 (Optimization)** - Allocator pooling and memory profiling patterns not well-documented
- **Phase 2 (Props)** - If props destructuring tests fail, may need to study SWC's `props_destructuring.rs` deeply

Phases with standard patterns (skip research-phase):
- **Phase 1 (OXC Update)** - Migration path documented in STACK.md, just execute
- **Phase 2 (Core QRL)** - Reference implementation clear, direct port
- **Phase 3 (Signals)** - Established patterns in test snapshots
- **Phase 4 (JSX/Imports)** - Well-documented in both implementations
- **Phase 6 (Advanced)** - Isolated features, low complexity

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Direct docs.rs verification, codebase already partially migrated |
| Features | HIGH | Complete test catalog from reference implementation |
| Architecture | HIGH | Direct source code analysis of both implementations |
| Pitfalls | HIGH | Based on snapshot comparison and OXC release notes |

**Overall confidence:** HIGH

### Gaps to Address

**Hash algorithm verification:** STACK.md notes potential hash divergence but doesn't verify the exact algorithm match. During Phase 2, compare `DefaultHasher` seed values and normalization steps between implementations to ensure runtime compatibility.

**Allocator memory behavior:** Pitfall 8 identifies potential memory pressure but lacks quantitative data. During Phase 7, establish baseline memory metrics with large test projects (1000+ components) and compare against SWC to catch regressions early.

**Source map integration:** Phase 5 requires `oxc_sourcemap` which isn't covered in current research. During planning, investigate `oxc_codegen::SourcemapBuilder` API and review Vite's OXC integration as reference implementation.

**Capture detection completeness:** Current OXC implementation has `captures` field but incomplete logic. During Phase 2 test porting, verify `scoped_idents` metadata matches SWC by comparing segment JSON for every test case with captures.

**Platform path handling:** Windows path separator normalization mentioned but not tested. Add CI matrix (Windows/Mac/Linux) during Phase 1 to catch platform-specific issues early.

## Sources

### Primary (HIGH confidence)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/` - OXC implementation analysis
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/` - SWC reference implementation
- [OXC Releases](https://github.com/oxc-project/oxc/releases) - Breaking changes 0.94.0 to 0.111.0
- [docs.rs/oxc/0.111.0](https://docs.rs/oxc/0.111.0/oxc/) - API verification
- [docs.rs/oxc/0.94.0](https://docs.rs/oxc/0.94.0/oxc/) - Baseline API
- OXC Transformer/Codegen CHANGELOGs - Detailed breaking change documentation
- 162 snapshot tests in `qwik-core/src/optimizer/core/src/snapshots/` - Feature catalog
- 19 snapshot tests in `optimizer/src/snapshots/` - Current coverage baseline

### Secondary (MEDIUM confidence)
- OXC vs SWC benchmarks - Performance characteristics
- Vite migration guide - OXC integration patterns

### Tertiary (LOW confidence)
- None - All findings verified against primary sources

---
*Research completed: 2026-01-29*
*Ready for roadmap: yes*
