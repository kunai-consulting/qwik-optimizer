---
phase: quick-002
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - optimizer/src/code_move.rs
  - optimizer/src/collector.rs
  - optimizer/src/inlined_fn.rs
  - optimizer/src/entry_strategy.rs
  - optimizer/src/is_const.rs
  - optimizer/src/component/shared.rs
  - optimizer/src/component/segment_data.rs
  - optimizer/src/const_replace.rs
  - optimizer/src/props_destructuring.rs
  - optimizer/src/component/component.rs
  - optimizer/src/component/id.rs
  - optimizer/src/component/source_info.rs
  - optimizer/src/processing_failure.rs
  - optimizer/src/transform/options.rs
  - optimizer/src/transform/state.rs
  - optimizer/src/transform/generator.rs
  - optimizer/src/transform/qrl.rs
  - optimizer/src/import_clean_up.rs
autonomous: true

must_haves:
  truths:
    - "Doc comments provide value beyond function/struct names"
    - "No verbose multi-line doc comments with step-by-step narration"
    - "All 239 tests pass after changes"
  artifacts:
    - path: "optimizer/src/code_move.rs"
      provides: "useLexicalScope injection"
      contains: "transform_function_expr"
    - path: "optimizer/src/collector.rs"
      provides: "Identifier collection for QRL analysis"
      contains: "IdentCollector"
---

<objective>
Trim verbose doc comments (///) to one-liners or remove entirely when function/struct names are self-explanatory.

Purpose: Reduce documentation noise that narrates implementation rather than documenting API behavior.
Output: Cleaner codebase with concise doc comments providing value beyond names.
</objective>

<context>
@.planning/STATE.md

Prior work: quick-001 removed all inline comments (//) from non-transform modules.
This task targets verbose doc comments (///) that narrate implementation steps.

Criteria for REMOVAL or TRIMMING:
- Remove `# Arguments` / `# Returns` sections (obvious from signature)
- Remove `This method:` numbered step lists (implementation narration)
- Remove code examples in doc comments (code should be obvious from tests)
- Remove `Ported from SWC` references (historical, not API documentation)
- Keep one-liner if it adds context beyond the function name
- Remove entirely if function name is self-explanatory

Examples:

BEFORE (verbose):
```rust
/// Transform a function expression to inject useLexicalScope at the start.
///
/// This is the main entry point for code movement transformation. It dispatches
/// based on expression type:
/// - Arrow functions: Converts expression body to block body if needed, then prepends useLexicalScope
/// - Function expressions: Prepends useLexicalScope to the body
/// - Other expressions: Returns unchanged
///
/// # Arguments
/// * `expr` - The function expression to transform
/// * `scoped_idents` - Variables captured from enclosing scope (sorted)
/// * `allocator` - OXC allocator for AST construction
///
/// # Returns
/// The transformed expression with useLexicalScope injection
pub fn transform_function_expr<'a>(...) -> Expression<'a>
```

AFTER (trimmed):
```rust
/// Inject `useLexicalScope` destructuring at function start for captured variables.
pub fn transform_function_expr<'a>(...) -> Expression<'a>
```

Or remove entirely if name is sufficient:
```rust
// BEFORE
/// Create a new IdentCollector
pub fn new() -> Self

// AFTER (remove - obvious from `new()`)
pub fn new() -> Self
```
</context>

<tasks>

<task type="auto">
  <name>Task 1: Trim verbose doc comments in utility modules</name>
  <files>
    optimizer/src/code_move.rs
    optimizer/src/collector.rs
    optimizer/src/inlined_fn.rs
    optimizer/src/entry_strategy.rs
    optimizer/src/is_const.rs
    optimizer/src/const_replace.rs
    optimizer/src/props_destructuring.rs
    optimizer/src/import_clean_up.rs
    optimizer/src/processing_failure.rs
  </files>
  <action>
For each file, apply these rules to all `///` doc comments:

1. Module-level `//!` comments: Trim to 1-2 lines max. Remove code examples, remove `Ported from SWC` notes.

2. Function doc comments:
   - Remove `# Arguments` sections entirely (obvious from signature)
   - Remove `# Returns` sections entirely (obvious from return type)
   - Remove `This method:` / `This function:` numbered step lists
   - Remove code examples (tests demonstrate usage)
   - Trim to one sentence if useful context remains
   - Remove entirely if function name is self-explanatory (e.g., `new()`, `default()`, getters)

3. Struct/enum doc comments:
   - Keep if explains PURPOSE beyond the name
   - Remove if just restates the struct name

4. Field doc comments:
   - Remove if field name is self-explanatory
   - Keep if non-obvious meaning

Specific changes by file:

**code_move.rs** (~31 lines -> ~5 lines):
- Module comment: Trim to "useLexicalScope injection for QRL segment files."
- `transform_function_expr`: One line about injecting useLexicalScope
- `transform_arrow_fn`: Remove (private, name is clear)
- `transform_fn`: Remove (private, name is clear)
- `create_use_lexical_scope`: Remove (name is self-explanatory)
- `create_return_stmt`: Remove (name is self-explanatory)

**collector.rs** (~22 lines -> ~8 lines):
- Module comment: Trim to "Identifier collector for QRL variable capture analysis."
- `ExportInfo`: Remove verbose field comments, struct doc is enough
- `IdentCollector`: Keep one-line description
- `new()`: Remove
- `get_words`: Remove (name is clear)
- `collect_exports`: Trim to one line

**inlined_fn.rs** (~21 lines -> ~5 lines):
- Module comment: Trim to "_fnSignal generation for computed signal expressions."
- Remove code examples
- `InlinedFnResult`: Keep one-liner
- `should_wrap_in_fn_signal`: Trim to one line
- `convert_inlined_fn`: Trim to one line
- Private helpers: Remove all doc comments

**entry_strategy.rs** (~18 lines -> ~6 lines):
- `EntryPolicy` trait: Trim to one line
- `get_entry_for_sym`: Remove verbose Args/Returns
- Strategy structs: Keep one-line descriptions

**is_const.rs** (~10 lines -> ~3 lines):
- Module comment: Trim to "Prop constness detection for JSX transformation."
- `is_const_expr`: Trim to one line
- `ConstCollector`: Remove (private implementation detail)

**const_replace.rs, props_destructuring.rs, import_clean_up.rs, processing_failure.rs**:
- Apply same rules as above
  </action>
  <verify>
Run `cargo build -p qwik-optimizer` - should compile
Run `cargo test -p qwik-optimizer` - all 239 tests pass
  </verify>
  <done>Utility modules have concise doc comments with no verbose narration</done>
</task>

<task type="auto">
  <name>Task 2: Trim verbose doc comments in component modules</name>
  <files>
    optimizer/src/component/shared.rs
    optimizer/src/component/segment_data.rs
    optimizer/src/component/component.rs
    optimizer/src/component/id.rs
    optimizer/src/component/source_info.rs
  </files>
  <action>
Apply same rules as Task 1 to component modules.

**component/shared.rs** (~17 lines -> ~8 lines):
- Keep constant doc comments (they document API values)
- Remove verbose import-related comments where name is obvious

**component/segment_data.rs** (~11 lines):
- Trim struct doc to one line
- Remove field comments if field names are clear

**component/component.rs, id.rs, source_info.rs**:
- Apply same trimming rules
  </action>
  <verify>
Run `cargo build -p qwik-optimizer` - should compile
Run `cargo test -p qwik-optimizer` - all 239 tests pass
  </verify>
  <done>Component modules have concise doc comments</done>
</task>

<task type="auto">
  <name>Task 3: Trim verbose doc comments in transform modules</name>
  <files>
    optimizer/src/transform/options.rs
    optimizer/src/transform/state.rs
    optimizer/src/transform/generator.rs
    optimizer/src/transform/qrl.rs
  </files>
  <action>
Apply same rules as Task 1 to transform modules.

These modules were cleaned in Phase 12 but may still have verbose doc comments.
Focus on:
- Removing `# Arguments` / `# Returns` sections
- Trimming multi-line descriptions to one line
- Removing doc comments on obvious methods
  </action>
  <verify>
Run `cargo build -p qwik-optimizer` - should compile
Run `cargo test -p qwik-optimizer` - all 239 tests pass
Verify total doc comment lines reduced by checking: `grep -c "^///" optimizer/src/**/*.rs`
  </verify>
  <done>Transform modules have concise doc comments, all tests pass</done>
</task>

</tasks>

<verification>
- `cargo build -p qwik-optimizer` compiles without errors
- `cargo test -p qwik-optimizer` shows 239 tests passing
- Doc comment line count reduced from ~161 to ~50-70
</verification>

<success_criteria>
- No verbose multi-line doc comments with step-by-step narration
- No `# Arguments` or `# Returns` sections
- All remaining doc comments provide value beyond function/struct names
- All 239 tests pass
</success_criteria>

<output>
After completion, create `.planning/quick/002-trim-verbose-doc-comments/002-SUMMARY.md`
</output>
