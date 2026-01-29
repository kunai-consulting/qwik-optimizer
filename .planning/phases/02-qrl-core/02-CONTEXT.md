# Phase 2: QRL Core - Context

**Gathered:** 2026-01-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement QRL extraction and transformation for all function types with correct hash generation. This includes arrow functions, function declarations, Component$ wrappers, nested QRLs, ternary expressions, and lexical scope capture tracking.

</domain>

<decisions>
## Implementation Decisions

### Parity Constraint
- **All behavior defined by SWC reference implementation** — match exactly
- Output format, hash generation, display names: replicate SWC behavior
- No design decisions needed — the existing implementation IS the specification

### Validation Approach
- Use existing qwik-core snapshot tests as ground truth
- Parity verified when OXC output matches SWC snapshots exactly

### Claude's Discretion
- Internal implementation structure and code organization
- Test ordering during development
- Error message wording (while matching error detection behavior)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — SWC parity defines all behavior.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-qrl-core*
*Context gathered: 2026-01-29*
