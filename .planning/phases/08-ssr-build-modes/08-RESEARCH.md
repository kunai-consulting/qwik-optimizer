# Phase 8: SSR & Build Modes - Research

**Researched:** 2026-01-29
**Domain:** Server/client code handling, build mode const replacement, dead code elimination
**Confidence:** HIGH

## Summary

Phase 8 implements server/client code handling and build mode transformations for the Qwik optimizer. This involves three main capabilities:

1. **Const Replacement**: Replace `isServer`, `isBrowser`, and `isDev` identifiers with their boolean values based on build configuration, enabling bundlers to perform dead code elimination.

2. **Segment Emission Control**: Control which QRL segments are emitted based on build mode, using `strip_ctx_name` and `strip_event_handlers` configuration to skip server-only or client-only code.

3. **Mode-Specific Transformations**: Different behavior for `Prod`, `Dev`, `Lib`, and `Test` modes - affecting symbol naming, debug info, and transformation behavior.

The SWC implementation provides a clear reference: `const_replace.rs` is a simple VisitMut that replaces identifiers imported from `@qwik.dev/core` or `@qwik.dev/core/build` with boolean literals. Dead code elimination is handled by the downstream bundler (Vite/Rollup) after const replacement makes branches statically analyzable.

**Primary recommendation:** Implement ConstReplacerVisitor as a separate traversal pass using OXC's VisitMut pattern, similar to the SWC implementation, to replace `isServer`/`isBrowser`/`isDev` identifiers with boolean literals before main transformation.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| OXC | 0.111.0 | AST traversal and mutation | Already in use, VisitMut for identifier replacement |
| TransformOptions | - | Build configuration | Already has `target` field for mode |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| oxc_traverse | 0.111.0 | Full traversal with context | For identifier replacement pass |
| oxc_ast_visit | 0.111.0 | Simpler visit pattern | Alternative for const replacement |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate visitor pass | Inline during main transform | SWC uses separate pass; cleaner separation |
| Full traverse_mut | Simple VisitMut | VisitMut simpler for identifier-only changes |

**Installation:**
No new dependencies needed - uses existing OXC infrastructure.

## Architecture Patterns

### Recommended Project Structure
```
optimizer/src/
├── const_replace.rs     # NEW: ConstReplacerVisitor for isServer/isDev replacement
├── transform.rs         # Existing: Add is_server, is_dev to TransformOptions
├── component/shared.rs  # Existing: Add BUILDER_IO_QWIK_BUILD source constant
└── collector.rs         # Existing: Track isServer/isBrowser/isDev imports
```

### Pattern 1: Const Replacement Visitor
**What:** Separate AST visitor that replaces imported identifiers with boolean literals
**When to use:** Before main QRL transformation, after import collection
**Example:**
```rust
// Source: qwik-core/src/optimizer/core/src/const_replace.rs
pub struct ConstReplacerVisitor {
    pub is_server: bool,
    pub is_dev: bool,
    // Tracks which local identifiers map to isServer/isBrowser/isDev
    pub is_server_ident: Option<Id>,    // from @qwik.dev/core/build
    pub is_browser_ident: Option<Id>,   // from @qwik.dev/core/build
    pub is_dev_ident: Option<Id>,       // from @qwik.dev/core/build
    pub is_core_server_ident: Option<Id>,   // from @qwik.dev/core
    pub is_core_browser_ident: Option<Id>,  // from @qwik.dev/core
    pub is_core_dev_ident: Option<Id>,      // from @qwik.dev/core
}

impl VisitMut for ConstReplacerVisitor {
    fn visit_mut_expr(&mut self, node: &mut ast::Expr) {
        // Match identifier against tracked imports
        // Replace with boolean literal
        // isBrowser is !isServer
    }
}
```

### Pattern 2: Segment Emission Control
**What:** Skip emitting segments based on strip_ctx_name and strip_event_handlers options
**When to use:** During QRL creation, check should_emit_segment()
**Example:**
```rust
// Source: qwik-core/src/optimizer/core/src/transform.rs
fn should_emit_segment(&self, segment_data: &SegmentData) -> bool {
    // Check strip_ctx_name (e.g., ["server"] to strip server$ QRLs)
    if let Some(strip_ctx_name) = self.options.strip_ctx_name {
        if strip_ctx_name.iter().any(|v| segment_data.ctx_name.starts_with(v.as_ref())) {
            return false;
        }
    }
    // Check strip_event_handlers for client-only builds
    if self.options.strip_event_handlers && segment_data.ctx_kind == SegmentKind::EventHandler {
        return false;
    }
    true
}
```

### Pattern 3: Noop QRL for Stripped Segments
**What:** When segment is stripped, emit _noopQrl() instead of actual segment
**When to use:** When should_emit_segment() returns false
**Example:**
```rust
// Source: qwik-core/src/optimizer/core/src/transform.rs
fn create_noop_qrl(&mut self, symbol_name: &Atom, segment_data: SegmentData) -> ast::CallExpr {
    // Create _noopQrl("s_symbolName") or _noopQrlDEV(...) call
    // Preserves call site for SSR but doesn't emit segment file
}
```

### Anti-Patterns to Avoid
- **Inline replacement during main transform:** Keep const replacement as separate pass for cleaner code
- **Custom DCE implementation:** Let bundler handle dead code elimination after const replacement
- **Modifying is_server at runtime:** These are compile-time constants, not runtime values

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dead code elimination | Custom tree-shaker | Bundler DCE (via const replacement) | Bundlers do this better with minifier |
| Boolean constant folding | Simplify if(true)/if(false) | Bundler simplify pass | Part of standard minification |
| Import tracking | Manual tracking | Extend existing GlobalCollect pattern | Already tracks imports by source/specifier |

**Key insight:** The optimizer's job is to make code statically analyzable by replacing `isServer` with `true`/`false`. The bundler (Vite/Rollup/esbuild) then eliminates dead branches. Don't duplicate bundler work.

## Common Pitfalls

### Pitfall 1: Aliased Import Handling
**What goes wrong:** User imports `import { isServer as s } from '@qwik.dev/core/build'` and replacement fails
**Why it happens:** Only looking for exact identifier name "isServer" instead of tracking local alias
**How to avoid:** Track import by specifier AND local name, similar to GlobalCollect.get_imported_local()
**Warning signs:** Tests with aliased imports fail while direct imports work

### Pitfall 2: Missing Build Source
**What goes wrong:** Replacement works for `@qwik.dev/core` but not `@qwik.dev/core/build`
**Why it happens:** Only checking one import source
**How to avoid:** Check both sources - SWC tracks 6 different identifier possibilities (3 from each source)
**Warning signs:** Some isServer usages replaced, others not

### Pitfall 3: Wrong Default for is_server
**What goes wrong:** Client code runs on server, security issues
**Why it happens:** Defaulting is_server to false when unspecified
**How to avoid:** Default is_server to true (safe default - server code is safer to run on server than client code on server)
**Warning signs:** Server-only code appearing in client bundles

### Pitfall 4: Mode vs Target Confusion
**What goes wrong:** Using wrong mode for Dev/Prod behavior
**Why it happens:** Confusing EmitMode (Prod/Dev/Lib/Test) with is_server (true/false)
**How to avoid:**
- `target` (EmitMode): Affects symbol naming, dev info, optimization level
- `is_server`: Affects code targeting (server vs client build)
**Warning signs:** Debug info in production builds, or is_server affecting naming

### Pitfall 5: Skipping Const Replacement in Test Mode
**What goes wrong:** isServer not replaced in tests, causing test failures
**Why it happens:** SWC skips const replacement in Test mode
**How to avoid:** Match SWC behavior - only skip in EmitMode::Test
**Warning signs:** Test output differs from snapshot expectations

## Code Examples

Verified patterns from official sources:

### Const Replacement Core Logic
```rust
// Source: qwik-core/src/optimizer/core/src/const_replace.rs
impl VisitMut for ConstReplacerVisitor {
    fn visit_mut_expr(&mut self, node: &mut ast::Expr) {
        let mode = match node {
            ast::Expr::Ident(ref ident) => {
                if id_eq!(ident, &self.is_server_ident) {
                    ConstVariable::IsServer
                } else if id_eq!(ident, &self.is_browser_ident) {
                    ConstVariable::IsBrowser
                } else if id_eq!(ident, &self.is_dev_ident) {
                    ConstVariable::IsDev
                } else if id_eq!(ident, &self.is_core_server_ident) {
                    ConstVariable::IsServer
                } else if id_eq!(ident, &self.is_core_browser_ident) {
                    ConstVariable::IsBrowser
                } else if id_eq!(ident, &self.is_core_dev_ident) {
                    ConstVariable::IsDev
                } else {
                    ConstVariable::None
                }
            }
            _ => ConstVariable::None,
        };
        match mode {
            ConstVariable::IsServer => {
                *node = ast::Expr::Lit(ast::Lit::Bool(ast::Bool {
                    span: DUMMY_SP,
                    value: self.is_server,
                }))
            }
            ConstVariable::IsBrowser => {
                *node = ast::Expr::Lit(ast::Lit::Bool(ast::Bool {
                    span: DUMMY_SP,
                    value: !self.is_server,  // isBrowser = !isServer
                }))
            }
            ConstVariable::IsDev => {
                *node = ast::Expr::Lit(ast::Lit::Bool(ast::Bool {
                    span: DUMMY_SP,
                    value: self.is_dev,
                }))
            }
            ConstVariable::None => {
                node.visit_mut_children_with(self);
            }
        }
    }
}
```

### TransformOptions Extension
```rust
// Source: qwik-core/src/optimizer/core/src/parse.rs (TransformCodeOptions)
pub struct TransformCodeOptions<'a> {
    // ... existing fields ...
    pub strip_ctx_name: Option<&'a [String]>,  // QRL context names to strip
    pub strip_event_handlers: bool,             // Strip event handler segments
    pub is_server: bool,                        // Server build (true) or client build (false)
}
```

### Integration Point in Parse Flow
```rust
// Source: qwik-core/src/optimizer/core/src/parse.rs
// After import collection, before QRL transformation:
if config.mode != EmitMode::Test {
    let mut const_replacer = ConstReplacerVisitor::new(config.is_server, is_dev, &collect);
    program.visit_mut_with(&mut const_replacer);
}
```

### Noop QRL Generation
```rust
// Source: qwik-core/src/optimizer/core/src/transform.rs
fn create_noop_qrl(&mut self, symbol_name: &Atom, segment_data: SegmentData) -> ast::CallExpr {
    let mut args = vec![ast::Expr::Lit(ast::Lit::Str(ast::Str {
        span: DUMMY_SP,
        value: symbol_name.clone(),
        raw: None,
    }))];

    let mut fn_name: &Atom = &_NOOP_QRL;
    if self.options.mode == EmitMode::Dev {
        args.push(get_qrl_dev_obj(/* dev info */));
        fn_name = &_NOOP_QRL_DEV;
    }

    // Include captured variables for SSR hydration tracking
    if !segment_data.scoped_idents.is_empty() {
        args.push(/* array of captured idents */);
    }
    self.create_internal_call(fn_name, args, true)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Runtime isServer check | Compile-time const replacement | Always in Qwik | Enables bundler DCE |
| Full segment strip | Noop QRL replacement | v1.0 | Preserves call sites for SSR |

**Deprecated/outdated:**
- `@builder.io/qwik` import path: Renamed to `@qwik.dev/core` - must handle both for compatibility
- `@builder.io/qwik/build` import path: Renamed to `@qwik.dev/core/build`

## Open Questions

None - the SWC implementation provides complete reference for all requirements.

## Sources

### Primary (HIGH confidence)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/const_replace.rs` - Full SWC const replacement implementation
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/parse.rs` - Integration point and flow
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/transform.rs` - should_emit_segment, create_noop_qrl
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/words.rs` - IS_SERVER, IS_BROWSER, IS_DEV, BUILDER_IO_QWIK_BUILD constants

### Secondary (MEDIUM confidence)
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/test.rs` - Test cases for is_server behavior
- `/Users/jackshelton/dev/open-source/qwik-optimizer/qwik-core/src/optimizer/core/src/snapshots/` - Expected output snapshots

### Tertiary (LOW confidence)
- None - all requirements verifiable from source code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using existing OXC infrastructure, pattern matches SWC
- Architecture: HIGH - Direct port of SWC patterns
- Pitfalls: HIGH - All pitfalls derived from SWC implementation analysis

**Research date:** 2026-01-29
**Valid until:** 60 days (stable pattern, no external dependencies)

---

## Implementation Summary for Planner

### Required Changes

1. **Add TransformOptions fields:**
   - `is_server: bool` (default: true)
   - `is_dev: bool` (derived from target == Dev)
   - `strip_ctx_name: Option<Vec<String>>`
   - `strip_event_handlers: bool`

2. **Create const_replace.rs module:**
   - `ConstReplacerVisitor` struct with import tracking
   - `VisitMut` implementation for identifier replacement
   - `new()` constructor that takes GlobalCollect reference to find imports

3. **Add to component/shared.rs:**
   - `BUILDER_IO_QWIK_BUILD: &str = "@qwik.dev/core/build"`
   - Constants for isServer, isBrowser, isDev identifiers

4. **Extend collector.rs:**
   - Method to find imported local name by source and specifier
   - Similar to existing `get_imported_local` pattern

5. **Integrate in transform pipeline:**
   - Apply const replacement after import collection, before QRL transformation
   - Skip in Test mode (match SWC behavior)

6. **Add should_emit_segment() logic:**
   - Check strip_ctx_name against segment_data.ctx_name
   - Check strip_event_handlers for EventHandler segments

7. **Implement create_noop_qrl():**
   - Generate _noopQrl("symbol_name") call
   - Include dev info in Dev mode
   - Preserve captured idents for SSR tracking

### Test Cases Needed
- isServer replacement (true for server build)
- isBrowser replacement (!isServer)
- isDev replacement (true for Dev mode)
- Aliased imports (`import { isServer as s }`)
- Both import sources (@qwik.dev/core and @qwik.dev/core/build)
- strip_ctx_name filtering
- strip_event_handlers filtering
- Noop QRL generation for stripped segments
