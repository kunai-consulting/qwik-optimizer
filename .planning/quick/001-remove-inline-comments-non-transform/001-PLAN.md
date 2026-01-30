---
phase: quick-001
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - optimizer/src/collector.rs
  - optimizer/src/const_replace.rs
  - optimizer/src/inlined_fn.rs
  - optimizer/src/props_destructuring.rs
  - optimizer/src/code_move.rs
  - optimizer/src/entry_strategy.rs
  - optimizer/src/is_const.rs
  - optimizer/src/js_lib_interface.rs
  - optimizer/src/component/component.rs
  - optimizer/src/component/id.rs
  - optimizer/src/component/language.rs
  - optimizer/src/component/mod.rs
  - optimizer/src/component/qrl.rs
  - optimizer/src/component/segment_data.rs
  - optimizer/src/component/shared.rs
  - optimizer/src/component/source_info.rs
autonomous: true

must_haves:
  truths:
    - "All inline comments (//) removed from non-transform modules"
    - "Doc comments (///) preserved on public API items"
    - "All 239 tests still pass"
  artifacts:
    - path: "optimizer/src/collector.rs"
      provides: "Identifier collection without inline comments"
    - path: "optimizer/src/const_replace.rs"
      provides: "SSR const replacement without inline comments"
    - path: "optimizer/src/inlined_fn.rs"
      provides: "Inlined function generation without inline comments"
    - path: "optimizer/src/props_destructuring.rs"
      provides: "Props destructuring without inline comments"
    - path: "optimizer/src/code_move.rs"
      provides: "Code movement without inline comments"
    - path: "optimizer/src/entry_strategy.rs"
      provides: "Entry strategy without inline comments"
    - path: "optimizer/src/is_const.rs"
      provides: "Const detection without inline comments"
    - path: "optimizer/src/js_lib_interface.rs"
      provides: "JS library interface without inline comments"
    - path: "optimizer/src/component/*.rs"
      provides: "Component modules without inline comments"
  key_links:
    - from: "all modules"
      to: "test suite"
      via: "cargo test"
      pattern: "239 passed"
---

<objective>
Remove all inline comments from non-transform modules to make code self-documenting.

Purpose: Complete the code reduction initiative by removing inline comments from modules outside the transform/ directory. This follows the pattern established in 12-02 and 12-03 where inline comments were removed from transform modules and JSX modules.

Output: 16 cleaned Rust files with only doc comments (///) on public API items.
</objective>

<execution_context>
@/Users/jackshelton/.claude/get-shit-done/workflows/execute-plan.md
@/Users/jackshelton/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove inline comments from utility modules</name>
  <files>
    optimizer/src/collector.rs
    optimizer/src/const_replace.rs
    optimizer/src/inlined_fn.rs
    optimizer/src/props_destructuring.rs
    optimizer/src/code_move.rs
    optimizer/src/entry_strategy.rs
    optimizer/src/is_const.rs
    optimizer/src/js_lib_interface.rs
  </files>
  <action>
    Remove all inline comments (//) from each file. Keep:
    - Module-level doc comments (//! at top of file)
    - Doc comments (///) on public structs, enums, traits, functions, and fields
    - Code logic should be self-documenting through clear naming

    Comment types to REMOVE:
    - `// explanation of what code does`
    - `// TODO`, `// FIXME`, `// NOTE` (unless critical)
    - `// SWC parity` or similar implementation notes
    - `// ---` section dividers
    - Comments after code on same line

    Do NOT remove:
    - `//!` module docs at file top
    - `///` doc comments on public items
    - `#[allow(...)]` or other attributes
  </action>
  <verify>
    Run `cargo check -p qwik-optimizer` to verify syntax is valid.
  </verify>
  <done>
    All 8 utility modules have inline comments removed while preserving doc comments.
  </done>
</task>

<task type="auto">
  <name>Task 2: Remove inline comments from component modules</name>
  <files>
    optimizer/src/component/component.rs
    optimizer/src/component/id.rs
    optimizer/src/component/language.rs
    optimizer/src/component/mod.rs
    optimizer/src/component/qrl.rs
    optimizer/src/component/segment_data.rs
    optimizer/src/component/shared.rs
    optimizer/src/component/source_info.rs
  </files>
  <action>
    Apply the same comment removal rules as Task 1 to the component/*.rs files:
    - Remove all inline comments (//)
    - Keep module-level doc comments (//!)
    - Keep doc comments (///) on public API items

    The component/ directory contains core data structures and their implementations.
    These files have fewer inline comments than transform modules.
  </action>
  <verify>
    Run `cargo check -p qwik-optimizer` to verify syntax is valid.
  </verify>
  <done>
    All 8 component modules have inline comments removed while preserving doc comments.
  </done>
</task>

<task type="auto">
  <name>Task 3: Verify all tests pass</name>
  <files>None (verification only)</files>
  <action>
    Run the full test suite to verify no functionality was broken by comment removal.
    All 239 tests must pass. If any test fails, it indicates a comment was incorrectly
    removed that contained actual code, or an edit introduced a syntax error.
  </action>
  <verify>
    Run `cargo test -p qwik-optimizer` and confirm 239 tests pass.
  </verify>
  <done>
    Test output shows "239 passed" with no failures.
  </done>
</task>

</tasks>

<verification>
1. `cargo check -p qwik-optimizer` compiles without errors
2. `cargo test -p qwik-optimizer` shows 239 passed tests
3. No inline comments remain in target files (only doc comments)
</verification>

<success_criteria>
- All 16 target files have inline comments removed
- Doc comments (///) preserved on public API items
- Module doc comments (//!) preserved at file tops
- All 239 tests pass
</success_criteria>

<output>
After completion, create `.planning/quick/001-remove-inline-comments-non-transform/001-SUMMARY.md`
</output>
