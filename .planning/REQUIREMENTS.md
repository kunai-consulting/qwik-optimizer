# Requirements: Qwik Optimizer OXC Port

**Defined:** 2026-01-29
**Core Value:** All 162 tests from qwik-core pass with exact output parity to the SWC implementation.

## v1 Requirements

Requirements for achieving test parity with qwik-core SWC optimizer.

### OXC Update

- [ ] **OXC-01**: Update oxc_parser to 0.111.0
- [ ] **OXC-02**: Update oxc_ast to 0.111.0
- [ ] **OXC-03**: Update oxc_semantic to 0.111.0
- [ ] **OXC-04**: Update oxc_transformer to 0.111.0
- [ ] **OXC-05**: Update oxc_codegen to 0.111.0
- [ ] **OXC-06**: Update oxc_index to latest compatible version
- [ ] **OXC-07**: Fix all breaking API changes from version update
- [ ] **OXC-08**: Existing 19 tests pass after update

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
| OXC-01 through OXC-08 | Phase 1 | Pending |
| QRL-01 through QRL-10 | Phase 2 | Pending |
| EVT-01 through EVT-08 | Phase 2 | Pending |
| PRP-01 through PRP-10 | Phase 3 | Pending |
| SIG-01 through SIG-06 | Phase 3 | Pending |
| JSX-01 through JSX-08 | Phase 4 | Pending |
| IMP-01 through IMP-08 | Phase 4 | Pending |
| ENT-01 through ENT-05 | Phase 5 | Pending |
| SSR-01 through SSR-05 | Phase 5 | Pending |
| TSX-01 through TSX-04 | Phase 6 | Pending |
| EDG-01 through EDG-09 | Phase 6 | Pending |
| TST-01 through TST-03 | Phase 7 | Pending |

**Coverage:**
- v1 requirements: 76 total
- Mapped to phases: 76
- Unmapped: 0 ✓

---
*Requirements defined: 2026-01-29*
*Last updated: 2026-01-29 after initial definition*
