# Roadmap: Qwik Optimizer OXC Port

## Overview

This roadmap delivers test parity between the OXC-based Qwik optimizer and the production SWC implementation. The journey begins with updating OXC dependencies (0.94.0 to 0.111.0), then systematically implements transformation features category by category, and concludes with comprehensive test validation. Success is measured by all 162 qwik-core snapshot tests passing with exact output parity.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: OXC Foundation** - Update all OXC crates to 0.111.0 and fix breaking API changes
- [x] **Phase 2: QRL Core** - Implement complete QRL extraction and transformation
- [ ] **Phase 3: Event Handlers** - Transform onClick$, onInput$, and custom event handlers
- [ ] **Phase 4: Props & Signals** - Handle component props destructuring and signal reactivity
- [ ] **Phase 5: JSX Transformation** - Complete JSX element and attribute handling
- [ ] **Phase 6: Imports & Exports** - Module system completeness including dynamic imports
- [ ] **Phase 7: Entry Strategies** - Code-splitting control (Inline, Single, PerSegment, etc.)
- [ ] **Phase 8: SSR & Build Modes** - Server/client code handling and dev/prod modes
- [ ] **Phase 9: TypeScript Support** - TSX parsing, type annotations, and generics
- [ ] **Phase 10: Edge Cases** - Nested loops, illegal code detection, and regression fixes
- [ ] **Phase 11: Test Infrastructure** - Port all 162 tests and validate parity

## Phase Details

### Phase 1: OXC Foundation
**Goal**: Codebase compiles on OXC 0.111.0 with all existing 31 tests passing
**Depends on**: Nothing (first phase)
**Requirements**: OXC-01, OXC-02, OXC-03, OXC-04, OXC-05, OXC-06, OXC-07, OXC-08
**Plans**: 2 plans

Plans:
- [x] 01-01-PLAN.md — Update OXC versions and fix compilation errors
- [x] 01-02-PLAN.md — Verify tests pass and document upgrade completion

**Success Criteria** (what must be TRUE):
  1. All OXC crates updated to 0.111.0 in Cargo.toml
  2. Codebase compiles without errors after API migration
  3. All 31 existing tests pass unchanged
  4. CI builds succeed on all platforms (Linux, macOS, Windows)

### Phase 2: QRL Core
**Goal**: QRL extraction works for all function types with correct hash generation
**Depends on**: Phase 1
**Requirements**: QRL-01, QRL-02, QRL-03, QRL-04, QRL-05, QRL-06, QRL-07, QRL-08, QRL-09, QRL-10
**Plans**: 7 plans

Plans:
- [x] 02-01-PLAN.md — Port IdentCollector for variable usage collection
- [x] 02-02-PLAN.md — Implement compute_scoped_idents and decl_stack tracking
- [x] 02-03-PLAN.md — Create SegmentData structure for segment metadata
- [x] 02-04-PLAN.md — Port code_move.rs for useLexicalScope injection
- [x] 02-05-PLAN.md — Wire QRL transformation and validate parity
- [ ] 02-06-PLAN.md — Fix capture detection ScopeId mismatch (gap closure)
- [ ] 02-07-PLAN.md — Verify hash stability and SWC parity (human verification)

**Success Criteria** (what must be TRUE):
  1. QRL extracts correctly from arrow functions and function declarations
  2. Component$ wrappers transform into lazy-loadable segments
  3. Nested QRLs and ternary expressions handle correctly
  4. Hash generation produces stable, unique identifiers matching SWC output
  5. Captured variables (lexical scope) are tracked correctly in segment metadata

### Phase 3: Event Handlers
**Goal**: All Qwik event handlers transform correctly for interactive components
**Depends on**: Phase 2
**Requirements**: EVT-01, EVT-02, EVT-03, EVT-04, EVT-05, EVT-06, EVT-07, EVT-08
**Success Criteria** (what must be TRUE):
  1. onClick$ and onInput$ transform into QRL-wrapped handlers
  2. Multiple event handlers on a single element work correctly
  3. Event handlers capture state variables correctly
  4. Custom event handlers (onCustomEvent$) transform correctly
  5. Non-element nodes skip event handler transformation
**Plans**: TBD

Plans:
- [ ] 03-01: Core event handler transformation (onClick$, onInput$)
- [ ] 03-02: Multiple handlers and state capture
- [ ] 03-03: Custom events and prevent default patterns
- [ ] 03-04: Edge cases (non-element nodes, JSX transpile bypass)

### Phase 4: Props & Signals
**Goal**: Component props and signals handle correctly for reactive data flow
**Depends on**: Phase 3
**Requirements**: PRP-01, PRP-02, PRP-03, PRP-04, PRP-05, PRP-06, PRP-07, PRP-08, PRP-09, PRP-10, SIG-01, SIG-02, SIG-03, SIG-04, SIG-05, SIG-06
**Success Criteria** (what must be TRUE):
  1. Props destructuring in component signatures transforms correctly
  2. Props spread and rest patterns handle correctly
  3. bind:value and bind:checked directives transform correctly
  4. useSignal$, useStore$, and useComputed$ extract correctly
  5. Signal access in JSX and store mutations work correctly
**Plans**: TBD

Plans:
- [ ] 04-01: Props destructuring and reconstruction
- [ ] 04-02: Props spread, rest, defaults, and aliasing
- [ ] 04-03: Bind directives (bind:value, bind:checked)
- [ ] 04-04: Signal hooks (useSignal$, useStore$, useComputed$)
- [ ] 04-05: Signal access in JSX and derived signals

### Phase 5: JSX Transformation
**Goal**: All JSX patterns transform correctly to _jsxSorted output
**Depends on**: Phase 4
**Requirements**: JSX-01, JSX-02, JSX-03, JSX-04, JSX-05, JSX-06, JSX-07, JSX-08
**Success Criteria** (what must be TRUE):
  1. Basic JSX elements transform to correct function calls
  2. Dynamic children and fragments handle correctly
  3. Spread attributes and conditional rendering work correctly
  4. List rendering with map produces correct output
  5. _jsxSorted output format matches SWC exactly
**Plans**: TBD

Plans:
- [ ] 05-01: Basic JSX and fragment transformation
- [ ] 05-02: Dynamic children and conditional rendering
- [ ] 05-03: Spread attributes and list rendering
- [ ] 05-04: _jsxSorted output format validation

### Phase 6: Imports & Exports
**Goal**: Module system transforms correctly with proper import cleanup
**Depends on**: Phase 5
**Requirements**: IMP-01, IMP-02, IMP-03, IMP-04, IMP-05, IMP-06, IMP-07, IMP-08
**Success Criteria** (what must be TRUE):
  1. Imports captured for QRL scope resolution work correctly
  2. Unused imports are cleaned up after transformation
  3. Synthesized imports (qrl, component$) are added correctly
  4. Named, default, and re-exports transform correctly
  5. Dynamic import generation for lazy loading works correctly
**Plans**: TBD

Plans:
- [ ] 06-01: Import capture for QRL scope
- [ ] 06-02: Unused import cleanup
- [ ] 06-03: Synthesized imports and dynamic import generation
- [ ] 06-04: Named, default, re-exports, and side-effect imports

### Phase 7: Entry Strategies
**Goal**: All code-splitting strategies produce correct output
**Depends on**: Phase 6
**Requirements**: ENT-01, ENT-02, ENT-03, ENT-04, ENT-05
**Success Criteria** (what must be TRUE):
  1. InlineStrategy keeps code in single output
  2. SingleStrategy produces one segment file
  3. PerSegmentStrategy produces multiple segment files
  4. PerComponentStrategy groups by component
  5. SmartStrategy optimizes based on analysis
**Plans**: TBD

Plans:
- [ ] 07-01: InlineStrategy implementation
- [ ] 07-02: SingleStrategy and PerSegmentStrategy
- [ ] 07-03: PerComponentStrategy and SmartStrategy

### Phase 8: SSR & Build Modes
**Goal**: Server/client code handling and build modes work correctly
**Depends on**: Phase 7
**Requirements**: SSR-01, SSR-02, SSR-03, SSR-04, SSR-05
**Success Criteria** (what must be TRUE):
  1. isServer const replaced correctly based on build target
  2. isDev const replaced correctly based on build mode
  3. Server-only code eliminated in client builds
  4. Client-only code eliminated in server builds
  5. Mode-specific transformations apply correctly
**Plans**: TBD

Plans:
- [ ] 08-01: isServer and isDev const replacement
- [ ] 08-02: Server/client code elimination
- [ ] 08-03: Mode-specific transformation validation

### Phase 9: TypeScript Support
**Goal**: TSX files parse and transform correctly with type handling
**Depends on**: Phase 8
**Requirements**: TSX-01, TSX-02, TSX-03, TSX-04
**Success Criteria** (what must be TRUE):
  1. TSX files parse and transform without errors
  2. Type annotations preserved where semantically required
  3. Type-only imports handled correctly (not captured for QRL)
  4. Generic component types work correctly
**Plans**: TBD

Plans:
- [ ] 09-01: TSX parsing and transformation
- [ ] 09-02: Type annotation and import handling
- [ ] 09-03: Generic component types

### Phase 10: Edge Cases
**Goal**: All edge cases and regression tests pass
**Depends on**: Phase 9
**Requirements**: EDG-01, EDG-02, EDG-03, EDG-04, EDG-05, EDG-06, EDG-07, EDG-08, EDG-09
**Success Criteria** (what must be TRUE):
  1. Nested loops in QRL handle correctly
  2. Skip transform marker works correctly
  3. Illegal code (classes, functions in QRL) detected and reported
  4. Empty components, unicode identifiers, and comments handle correctly
  5. All 6 issue regression tests pass (issues 117, 150, 476, 964, etc.)
**Plans**: TBD

Plans:
- [ ] 10-01: Nested loops and skip transform marker
- [ ] 10-02: Illegal code detection
- [ ] 10-03: Empty components, unicode, and comments
- [ ] 10-04: Async/await in QRL
- [ ] 10-05: Issue regression tests

### Phase 11: Test Infrastructure
**Goal**: All 162 snapshot tests ported and passing with output parity
**Depends on**: Phase 10
**Requirements**: TST-01, TST-02, TST-03
**Success Criteria** (what must be TRUE):
  1. All 162 snapshot tests from qwik-core ported to OXC implementation
  2. Snapshot format matches insta requirements
  3. Test input files organized consistently with qwik-core structure
  4. All 162 tests pass with exact output parity to SWC
**Plans**: TBD

Plans:
- [ ] 11-01: Port remaining snapshot tests
- [ ] 11-02: Snapshot format and organization validation
- [ ] 11-03: Final parity verification

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> ... -> 11

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. OXC Foundation | 2/2 | Complete | 2026-01-29 |
| 2. QRL Core | 7/7 | Complete | 2026-01-29 |
| 3. Event Handlers | 0/4 | Not started | - |
| 4. Props & Signals | 0/5 | Not started | - |
| 5. JSX Transformation | 0/4 | Not started | - |
| 6. Imports & Exports | 0/4 | Not started | - |
| 7. Entry Strategies | 0/3 | Not started | - |
| 8. SSR & Build Modes | 0/3 | Not started | - |
| 9. TypeScript Support | 0/3 | Not started | - |
| 10. Edge Cases | 0/5 | Not started | - |
| 11. Test Infrastructure | 0/3 | Not started | - |

---
*Created: 2026-01-29*
*Last updated: 2026-01-29 (Phase 2 complete)*
