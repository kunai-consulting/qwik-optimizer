# External Integrations

**Analysis Date:** 2026-01-29

## APIs & External Services

**None detected**

This project does not integrate with external APIs or third-party services. It is a standalone compiler/optimizer with no network communication.

## Data Storage

**Databases:**
- None - Not applicable

**File Storage:**
- Local filesystem only
  - Input: Reads JavaScript/TypeScript source files from `src_dir` parameter
  - Output: Returns transformed code in memory, no persistent writes
  - Test data: `optimizer/src/test_input/` contains test projects

**Caching:**
- None - In-memory transformation only

## Authentication & Identity

**None** - No authentication or user identity handling required

## Monitoring & Observability

**Error Tracking:**
- None - Errors are returned as structured `Diagnostic` objects in TransformOutput

**Logs:**
- None - No logging framework integrated
- Error reporting via Rust error types (`thiserror`) and diagnostic structs

## CI/CD & Deployment

**Hosting:**
- GitHub repository (kunai-consulting/qwik-optimizer)
- No production deployment - This is a library/optimizer tool

**CI Pipeline:**
- None detected in codebase
- Build system: Cargo with NAPI multi-platform support (handles x86_64, aarch64, etc.)

## Environment Configuration

**Required env vars:**
- None - Configuration passed entirely via function parameters

**Configuration Input:**
- All configuration provided via `TransformModulesOptions` struct in function calls
- Options include: `src_dir`, `root_dir`, `entry_strategy`, `minify`, `transpile_ts`, `transpile_jsx`, `mode`, and various feature flags

**Secrets location:**
- Not applicable - No secrets required

## Webhooks & Callbacks

**Incoming:**
- None - Not applicable

**Outgoing:**
- None - Not applicable

## Node.js Integration

**Module Export:**
- NAPI module name: `qwik`
- Function: `transform_modules(options: TransformModulesOptions): Promise<TransformOutput>`
- Async via tokio with thread pool for CPU-intensive work (`task::spawn_blocking`)
- Return type: Serialized to JSON via serde_json

**Input/Output Format:**
- Input: TransformModuleInput array with path, code, optional dev_path
- Output: TransformOutput including:
  - `modules`: Transformed module code and metadata
  - `diagnostics`: Errors and warnings
  - `isTypeScript`: Language detection
  - `isJsx`: JSX usage detection
  - `segment`: Optional segment analysis for code splitting

## Source Maps

**Handling:**
- Optional base64-encoded inline source maps
- Controlled by `source_maps` boolean in TransformModulesOptions
- Generated using Oxc codegen infrastructure

## Code Analysis Features

**No external services for:**
- Bundle size analysis
- Performance metrics
- Usage telemetry
- Build optimization suggestions

All analysis is local and deterministic.

---

*Integration audit: 2026-01-29*
