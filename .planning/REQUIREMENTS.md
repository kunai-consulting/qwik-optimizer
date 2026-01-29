# Requirements: Qwik Optimizer OXC Port

**Defined:** 2026-01-29
**Core Value:** All 162 tests from qwik-core pass with exact output parity to the SWC implementation.

## v1 Requirements

Requirements for achieving test parity with qwik-core SWC optimizer.

### OXC Update

- [x] **OXC-01**: Update oxc_parser to 0.111.0
- [x] **OXC-02**: Update oxc_ast to 0.111.0
- [x] **OXC-03**: Update oxc_semantic to 0.111.0
- [x] **OXC-04**: Update oxc_transformer to 0.111.0
- [x] **OXC-05**: Update oxc_codegen to 0.111.0
- [x] **OXC-06**: Update oxc_index to latest compatible version
- [x] **OXC-07**: Fix all breaking API changes from version update
- [x] **OXC-08**: Existing 31 tests pass after update

### Core QRL & Components

- [ ] **QRL-01**: QRL extraction from arrow functions
- [ ] **QRL-02**: QRL extraction from function declarations
- [ ] **QRL-03**: Component$ transformation
- [ ] **QRL-04**: Nested QRL handling
- [ ] **QRL-05**: QRL in ternary expressions
- [ ] **QRL-06**: Multiple QRLs per file
- [ ] **QRL-07**: QRL with captured variables (lexical scope)
- [ ] **QRL-08**: QRL display name generation
- [ ] **QRL-09**: QRL hash generation (stable, unique)
- [ ] **QRL-10**: Component with normal function transformation

### Event Handlers

- [ ] **EVT-01**: onClick$ transformation
- [ ] **EVT-02**: onInput$ transformation
- [ ] **EVT-03**: Multiple event handlers on single element
- [ ] **EVT-04**: Event handler with captured state
- [ ] **EVT-05**: Event names without JSX transpile
- [ ] **EVT-06**: Event handlers on non-element nodes (skip)
- [ ] **EVT-07**: Prevent default patterns
- [ ] **EVT-08**: Custom event handlers (onCustomEvent$)

### Props Handling

- [ ] **PRP-01**: Props destructuring in component signature
- [ ] **PRP-02**: Props spread handling
- [ ] **PRP-03**: Immutable props optimization
- [ ] **PRP-04**: bind:value directive
- [ ] **PRP-05**: bind:checked directive
- [ ] **PRP-06**: Props in variable declarations
- [ ] **PRP-07**: Destructured args reconstruction
- [ ] **PRP-08**: Props with default values
- [ ] **PRP-09**: Rest props (...rest)
- [ ] **PRP-10**: Props aliasing

### Signals & Reactivity

- [ ] **SIG-01**: useSignal$ extraction
- [ ] **SIG-02**: useStore$ extraction
- [ ] **SIG-03**: useComputed$ extraction
- [ ] **SIG-04**: Signal access in JSX
- [ ] **SIG-05**: Store mutations
- [ ] **SIG-06**: Derived signals

### JSX Transformation

- [ ] **JSX-01**: Basic JSX element transformation
- [ ] **JSX-02**: JSX with dynamic children
- [ ] **JSX-03**: JSX fragment handling
- [ ] **JSX-04**: JSX spread attributes
- [ ] **JSX-05**: JSX conditional rendering
- [ ] **JSX-06**: JSX list rendering (map)
- [ ] **JSX-07**: _jsxSorted output format
- [ ] **JSX-08**: Immutable props in JSX

### Imports & Exports

- [ ] **IMP-01**: Import capture for QRL scope
- [ ] **IMP-02**: Unused import cleanup
- [ ] **IMP-03**: Synthesized imports (qrl, component$)
- [ ] **IMP-04**: Named exports
- [ ] **IMP-05**: Default exports
- [ ] **IMP-06**: Re-exports
- [ ] **IMP-07**: Side-effect imports preservation
- [ ] **IMP-08**: Dynamic import generation

### Entry Strategies

- [ ] **ENT-01**: InlineStrategy implementation
- [ ] **ENT-02**: SingleStrategy implementation
- [ ] **ENT-03**: PerSegmentStrategy implementation
- [ ] **ENT-04**: PerComponentStrategy implementation
- [ ] **ENT-05**: SmartStrategy implementation

### SSR & Build Modes

- [ ] **SSR-01**: isServer const replacement
- [ ] **SSR-02**: isDev const replacement
- [ ] **SSR-03**: Server-only code elimination
- [ ] **SSR-04**: Client-only code elimination
- [ ] **SSR-05**: Mode-specific transformations

### TypeScript Support

- [ ] **TSX-01**: TSX file parsing and transformation
- [ ] **TSX-02**: Type annotation preservation where needed
- [ ] **TSX-03**: Type-only import handling
- [ ] **TSX-04**: Generic component types

### Edge Cases & Regressions

- [ ] **EDG-01**: Nested loops in QRL
- [ ] **EDG-02**: Skip transform marker
- [ ] **EDG-03**: Illegal code detection (classes in QRL)
- [ ] **EDG-04**: Illegal code detection (functions in QRL)
- [ ] **EDG-05**: Empty component handling
- [ ] **EDG-06**: Unicode in identifiers
- [ ] **EDG-07**: Comments preservation
- [ ] **EDG-08**: Async/await in QRL
- [ ] **EDG-09**: Issue regression tests (6 tests)

### Test Infrastructure

- [ ] **TST-01**: All 162 snapshot tests ported from qwik-core
- [ ] **TST-02**: Snapshot format matches insta requirements
- [ ] **TST-03**: Test input files organized consistently

## v2 Requirements

Deferred to future release. Not in current roadmap.

### Source Maps

- **SRC-01**: Source map generation for transformed code
- **SRC-02**: Inline source map option
- **SRC-03**: External source map file option

### Performance

- **PRF-01**: Memory optimization for large files
- **PRF-02**: Parallel file processing
- **PRF-03**: Incremental compilation support

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Production deployment | This milestone is test parity only |
| Performance benchmarking | Focus on correctness first |
| New features beyond SWC parity | Match existing behavior only |
| WASM build | Focus on native first, WASM later |
| Breaking API changes to NAPI interface | Maintain compatibility |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| OXC-01 | Phase 1 | Complete |
| OXC-02 | Phase 1 | Complete |
| OXC-03 | Phase 1 | Complete |
| OXC-04 | Phase 1 | Complete |
| OXC-05 | Phase 1 | Complete |
| OXC-06 | Phase 1 | Complete |
| OXC-07 | Phase 1 | Complete |
| OXC-08 | Phase 1 | Complete |
| QRL-01 | Phase 2 | Pending |
| QRL-02 | Phase 2 | Pending |
| QRL-03 | Phase 2 | Pending |
| QRL-04 | Phase 2 | Pending |
| QRL-05 | Phase 2 | Pending |
| QRL-06 | Phase 2 | Pending |
| QRL-07 | Phase 2 | Pending |
| QRL-08 | Phase 2 | Pending |
| QRL-09 | Phase 2 | Pending |
| QRL-10 | Phase 2 | Pending |
| EVT-01 | Phase 3 | Pending |
| EVT-02 | Phase 3 | Pending |
| EVT-03 | Phase 3 | Pending |
| EVT-04 | Phase 3 | Pending |
| EVT-05 | Phase 3 | Pending |
| EVT-06 | Phase 3 | Pending |
| EVT-07 | Phase 3 | Pending |
| EVT-08 | Phase 3 | Pending |
| PRP-01 | Phase 4 | Pending |
| PRP-02 | Phase 4 | Pending |
| PRP-03 | Phase 4 | Pending |
| PRP-04 | Phase 4 | Pending |
| PRP-05 | Phase 4 | Pending |
| PRP-06 | Phase 4 | Pending |
| PRP-07 | Phase 4 | Pending |
| PRP-08 | Phase 4 | Pending |
| PRP-09 | Phase 4 | Pending |
| PRP-10 | Phase 4 | Pending |
| SIG-01 | Phase 4 | Pending |
| SIG-02 | Phase 4 | Pending |
| SIG-03 | Phase 4 | Pending |
| SIG-04 | Phase 4 | Pending |
| SIG-05 | Phase 4 | Pending |
| SIG-06 | Phase 4 | Pending |
| JSX-01 | Phase 5 | Pending |
| JSX-02 | Phase 5 | Pending |
| JSX-03 | Phase 5 | Pending |
| JSX-04 | Phase 5 | Pending |
| JSX-05 | Phase 5 | Pending |
| JSX-06 | Phase 5 | Pending |
| JSX-07 | Phase 5 | Pending |
| JSX-08 | Phase 5 | Pending |
| IMP-01 | Phase 6 | Pending |
| IMP-02 | Phase 6 | Pending |
| IMP-03 | Phase 6 | Pending |
| IMP-04 | Phase 6 | Pending |
| IMP-05 | Phase 6 | Pending |
| IMP-06 | Phase 6 | Pending |
| IMP-07 | Phase 6 | Pending |
| IMP-08 | Phase 6 | Pending |
| ENT-01 | Phase 7 | Pending |
| ENT-02 | Phase 7 | Pending |
| ENT-03 | Phase 7 | Pending |
| ENT-04 | Phase 7 | Pending |
| ENT-05 | Phase 7 | Pending |
| SSR-01 | Phase 8 | Pending |
| SSR-02 | Phase 8 | Pending |
| SSR-03 | Phase 8 | Pending |
| SSR-04 | Phase 8 | Pending |
| SSR-05 | Phase 8 | Pending |
| TSX-01 | Phase 9 | Pending |
| TSX-02 | Phase 9 | Pending |
| TSX-03 | Phase 9 | Pending |
| TSX-04 | Phase 9 | Pending |
| EDG-01 | Phase 10 | Pending |
| EDG-02 | Phase 10 | Pending |
| EDG-03 | Phase 10 | Pending |
| EDG-04 | Phase 10 | Pending |
| EDG-05 | Phase 10 | Pending |
| EDG-06 | Phase 10 | Pending |
| EDG-07 | Phase 10 | Pending |
| EDG-08 | Phase 10 | Pending |
| EDG-09 | Phase 10 | Pending |
| TST-01 | Phase 11 | Pending |
| TST-02 | Phase 11 | Pending |
| TST-03 | Phase 11 | Pending |

**Coverage:**
- v1 requirements: 84 total
- Mapped to phases: 84
- Unmapped: 0

---
*Requirements defined: 2026-01-29*
*Last updated: 2026-01-29 Phase 1 complete*
