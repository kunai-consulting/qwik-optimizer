# Domain Pitfalls: OXC Migration (0.94.0 to 0.111.0) and SWC Parity

**Domain:** Compiler/Transformer Migration - Qwik Optimizer
**Researched:** 2026-01-29
**Confidence:** HIGH (direct source analysis + OXC release notes)

---

## Critical Pitfalls

Mistakes that cause rewrites, production regressions, or major feature breakage.

### Pitfall 1: OXC Breaking API Changes Between 0.94.0 and 0.111.0

**What goes wrong:** Code compiles but produces incorrect transformations, or fails to compile entirely after OXC update.

**Why it happens:** OXC is pre-1.0 and makes breaking changes in minor versions. Between 0.94.0 and 0.111.0, several breaking changes occurred:
- `Scoping::scope_build_child_ids` was removed (affects scope traversal)
- `TSEnumDeclaration` scope moved to `TSEnumBody` (affects TypeScript enum handling)
- `ThisExpression` removed from `TSModuleReference` (affects module reference patterns)
- New `Ident` type introduced (affects identifier handling across codebase)

**Consequences:**
- Semantic analysis breaks silently (scopes computed incorrectly)
- TypeScript transforms fail or produce wrong output
- Identifier matching logic fails without obvious errors

**Prevention:**
1. Read OXC changelog between 0.94.0 and 0.111.0 before updating
2. Update incrementally (e.g., 0.94 -> 0.100 -> 0.105 -> 0.111) to isolate breaking changes
3. Run full test suite after each version bump
4. Use `cargo doc` to verify API signatures match expected patterns

**Detection:**
- Compilation errors mentioning removed methods/types
- Snapshot tests failing with scope-related differences
- TypeScript files producing different output than before update

**Phase:** Address during OXC update phase (before test porting)

---

### Pitfall 2: Codegen Output Format Differences Between SWC and OXC

**What goes wrong:** Semantically equivalent code has different textual representation, causing snapshot test failures and potentially breaking downstream consumers that parse the output.

**Why it happens:** SWC and OXC have different codegen formatting defaults:

| Aspect | SWC Output | OXC Output |
|--------|------------|------------|
| Arrow functions | `()=>` (no space) | `() =>` (space before arrow) |
| Import hoisting | Creates `const i_hash = ()=>import(...)` | Inline in qrl call |
| JSX handling | Keeps JSX in output segments | Transforms to `_jsxSorted`/`_jsxSplit` |
| PURE comments | `/*#__PURE__*/` (no space) | `/* @__PURE__ */` (with spaces) |
| Semicolons | Minimal | Consistent |
| Indentation | Tab-based | Tab-based (but different tab stops) |
| Trailing newlines | Single | May vary |

**Consequences:**
- All 162 snapshot tests fail on whitespace differences
- Source maps have different column offsets
- Production bundles differ in size (even if semantically equivalent)
- Git diffs polluted with formatting noise

**Prevention:**
1. Define "parity" as semantic equivalence, not textual equivalence
2. Create format-normalized comparison for snapshot validation
3. Consider using a shared formatter (e.g., Prettier) as post-processing step
4. Document accepted differences explicitly

**Detection:**
- Snapshot tests fail but code executes correctly
- Diff shows only whitespace/formatting changes
- Hash values differ due to different input strings

**Phase:** Address during test porting phase with explicit parity strategy

---

### Pitfall 3: Hash Generation Divergence

**What goes wrong:** Generated symbol names and file names differ between SWC and OXC implementations, breaking runtime lazy-loading.

**Why it happens:** Hash generation depends on:
- Input string content (which differs due to codegen formatting)
- Hash algorithm implementation details
- Path normalization (different on Windows/Mac/Linux)
- Scope handling (which differs between SWC's SyntaxContext and OXC's SymbolId)

**Example from snapshots:**
- SWC: `renderHeader_zBbHWn4e8Cg`
- OXC: `renderHeader_ZgC5rsivXF0`

**Consequences:**
- Runtime import() calls reference non-existent files
- QRL strings don't match generated file names
- Caching invalidated unnecessarily on migration

**Prevention:**
1. Use identical hash algorithm (DefaultHasher with same seed)
2. Hash normalized/canonicalized strings (remove formatting differences before hashing)
3. Verify path handling is platform-independent
4. Add explicit hash stability tests

**Detection:**
- Runtime errors: "Failed to load module"
- QRL name doesn't match exported symbol
- File name in import() doesn't exist in output

**Phase:** Address during core implementation phase

---

### Pitfall 4: Source Map Generation Missing

**What goes wrong:** Debugging production code becomes impossible; error stack traces point to minified code.

**Why it happens:** OXC implementation currently returns `None` for source maps while SWC generates full source maps:

```rust
// OXC current state (component.rs, transform.rs)
// Source map: None

// SWC produces
Some("{\"version\":3,\"sources\":[...],\"mappings\":\"...\"}")
```

**Consequences:**
- Production debugging impossible
- Error reporting tools (Sentry, etc.) show useless stack traces
- Integration tests that validate source maps fail

**Prevention:**
1. Use `oxc_sourcemap` crate alongside `oxc_codegen`
2. Pass source map options to codegen
3. Collect mappings during transformation
4. Generate and embed source maps in output

**Detection:**
- Snapshot comparison shows `None` vs `Some("...")`
- Browser DevTools can't show original source
- Stack traces reference generated code locations

**Phase:** Address during feature completion phase (explicitly called out in PROJECT.md as missing)

---

### Pitfall 5: Semantic Scope Mismatch

**What goes wrong:** Variables are incorrectly classified as captured/uncaptured, causing runtime errors or incorrect lazy-loading behavior.

**Why it happens:** SWC uses `SyntaxContext`-based `Id` tuples for symbol tracking while OXC uses `SymbolId`/`ReferenceId`. The semantic models differ:

- SWC: Tracks symbols via hygiene marks (syntax context)
- OXC: Tracks symbols via semantic analysis passes

Additionally, `Reference` now stores `scope_id` directly in recent OXC versions, changing how scope traversal works.

**Consequences:**
- Variables incorrectly hoisted out of QRL segments
- `useLexicalScope` calls have wrong parameters
- Captured variables not included in segment closures

**Prevention:**
1. Study OXC's `oxc_semantic` module thoroughly
2. Write explicit tests for capture detection
3. Compare `scoped_idents` in segment metadata between implementations
4. Verify `captures: true/false` matches expected behavior

**Detection:**
- Segment metadata shows `"captures": false` when it should be `true`
- Runtime error: "Cannot read property of undefined"
- `useLexicalScope()` returns wrong values

**Phase:** Address during test porting phase with capture-specific test cases

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or partial feature gaps.

### Pitfall 6: Import Ordering Differences

**What goes wrong:** Import statements appear in different order, causing snapshot failures and potentially affecting module initialization order.

**Why it happens:**
- OXC uses `BTreeSet` for imports (sorted)
- SWC may use different collection types
- Import hoisting occurs at different stages

**Example:**
```javascript
// SWC
import { qrl } from "@qwik.dev/core";
const i_hash = ()=>import("...");
import { component } from '@qwik.dev/core';

// OXC
import { component, qrl } from "@qwik.dev/core";
```

**Prevention:**
1. Normalize import order as post-processing step
2. Document expected import ordering behavior
3. Verify side-effect imports preserve order

**Detection:**
- Snapshot diffs show only import reordering
- Module initialization order changes

**Phase:** Address during test porting phase

---

### Pitfall 7: JSX Runtime Naming Differences

**What goes wrong:** Different JSX helper function names break runtime integration.

**Why it happens:**
- SWC uses: `_jsxQ`, Qwik-specific JSX runtime
- OXC uses: `_jsxSorted`, `_jsxSplit`

**Prevention:**
1. Verify runtime exports match expected names
2. Update `@qwik.dev/core` to export both name variants, or
3. Align on single naming convention

**Detection:**
- Runtime error: "_jsxSorted is not a function"
- JSX doesn't render correctly

**Phase:** Address during JSX transformation phase

---

### Pitfall 8: Allocator Memory Pressure

**What goes wrong:** Large projects cause OOM errors or severe slowdowns.

**Why it happens:**
- New `Allocator::default()` created per component in OXC
- OXC recently increased chunk size from 512B to 16KB (good for throughput, bad for memory)
- No allocator pooling between component generations

**Prevention:**
1. Reuse allocators across component batch processing
2. Profile memory usage with large test projects
3. Consider arena size limits for embedded contexts (WASM)

**Detection:**
- Process memory grows unbounded during large builds
- WASM out-of-memory errors
- Build time increases non-linearly with project size

**Phase:** Address during optimization phase (post-correctness)

---

### Pitfall 9: TypeScript Decorator Support Gap

**What goes wrong:** Modern decorator syntax fails to transform.

**Why it happens:** OXC transformer does not support lowering native decorators (waiting for spec to stabilize). SWC has more mature decorator support.

**Prevention:**
1. Document decorator limitations
2. Recommend Babel/SWC fallback for decorator-heavy code
3. Test decorator scenarios explicitly

**Detection:**
- Build errors on decorator syntax
- Untransformed decorators in output

**Phase:** Document as known limitation

---

### Pitfall 10: CTX Name and Parent Segment Tracking

**What goes wrong:** Segment metadata has wrong `ctxName` or `parent` values, breaking debugging and source attribution.

**Example from snapshots:**
```javascript
// SWC
"ctxName": "$",
"parent": "renderHeader_zBbHWn4e8Cg",

// OXC
"ctxName": "renderHeader_div_onClick_vU0qgjVefds",
"parent": null,
```

**Why it happens:**
- Different segment stack tracking logic
- Context name derived differently (SWC uses marker function name, OXC uses generated symbol name)

**Prevention:**
1. Align `ctxName` to use marker function name (`$`, `component`, etc.)
2. Track parent segment during nested QRL extraction
3. Add explicit parent-child relationship tests

**Detection:**
- Segment metadata comparison shows different `ctxName`/`parent` values
- Debugging tools show wrong context

**Phase:** Address during test porting phase

---

## Minor Pitfalls

Mistakes that cause annoyance but are easily fixable.

### Pitfall 11: File Extension Handling

**What goes wrong:** Output files have `.js` extension when source was `.tsx`.

**Example:**
```
// SWC: test.tsx_renderHeader_zBbHWn4e8Cg.tsx
// OXC: test_example_1.tsx_renderHeader_ZgC5rsivXF0.js
```

**Prevention:**
1. Carry source extension through to output
2. Or explicitly configure output extension

**Detection:** File extension mismatch in snapshots

---

### Pitfall 12: Path Separator Normalization

**What goes wrong:** Windows vs Unix path separators cause test failures on different platforms.

**Prevention:**
1. Use `path-slash` or similar for normalization
2. Test on both Windows and Unix CI

**Detection:** Tests pass locally but fail in CI

---

### Pitfall 13: Location Span Differences

**What goes wrong:** `loc` array values differ, affecting source map accuracy and debugging.

**Example:**
```javascript
// SWC: "loc": [90, 161]
// OXC: "loc": [0, 0]
```

**Prevention:**
1. Preserve original spans during transformation
2. Track span mapping through allocator cloning

**Detection:** Snapshot comparison shows `[0, 0]` for all locations

---

### Pitfall 14: Parameter Name Preservation

**What goes wrong:** `paramNames` field missing from segment metadata.

**Example:**
```javascript
// SWC: "paramNames": ["ctx"]
// OXC: (field absent)
```

**Prevention:** Extract parameter names during function processing

**Detection:** Missing `paramNames` in segment JSON

---

## Phase-Specific Warnings

| Phase | Likely Pitfall | Mitigation |
|-------|----------------|------------|
| OXC Update | API breaking changes (Pitfall 1) | Incremental updates, full test runs |
| OXC Update | Allocator API changes | Review `oxc_allocator` changelog |
| Core Implementation | Hash divergence (Pitfall 3) | Normalize inputs before hashing |
| Core Implementation | Scope tracking differences (Pitfall 5) | Study `oxc_semantic` internals |
| Test Porting | Output format differences (Pitfall 2) | Define parity criteria early |
| Test Porting | Import ordering (Pitfall 6) | Post-process or accept differences |
| Feature Completion | Source maps (Pitfall 4) | Implement `oxc_sourcemap` integration |
| Feature Completion | Captures not implemented | Port `useLexicalScope` logic |
| Optimization | Memory pressure (Pitfall 8) | Profile with large projects |
| All Phases | Platform-specific paths (Pitfall 12) | Test on multiple platforms |

---

## Prevention Strategy Summary

### Before Starting Migration

1. **Freeze OXC version target** - Pick 0.111.0 and document any API gaps
2. **Define parity criteria** - Exact match vs semantic equivalence
3. **Create comparison tooling** - Automated diff that ignores expected differences
4. **Set up multi-platform CI** - Test on Windows, Mac, Linux

### During Migration

1. **Update OXC incrementally** - Don't jump 17 versions at once
2. **Run tests after each change** - Catch regressions early
3. **Document accepted differences** - Whitespace, import order, etc.
4. **Track hash stability** - Any hash change is a potential bug

### After Migration

1. **Validate production builds** - Test with real projects
2. **Compare bundle sizes** - Should be within 5% of SWC output
3. **Test source maps in browser** - Verify debugging works
4. **Profile memory usage** - Ensure no regression vs SWC

---

## Sources

- OXC Releases: https://github.com/oxc-project/oxc/releases
- OXC Documentation: https://oxc.rs/
- Direct codebase analysis:
  - `/Users/jackshelton/dev/open-source/qwik-optimizer/optimizer/src/`
  - `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/`
- Snapshot comparison between SWC and OXC test outputs
- OXC vs SWC benchmarks: https://github.com/oxc-project/bench-transformer
- Vite migration guide: https://main.vite.dev/guide/migration
