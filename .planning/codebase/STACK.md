# Technology Stack

**Analysis Date:** 2026-01-29

## Languages

**Primary:**
- Rust 2021 edition - Core optimizer engine and transformation logic

**Secondary:**
- JavaScript/CommonJS - NAPI bindings and test harness for Node.js consumption

## Runtime

**Environment:**
- Rust stable (via Nix flake: `rust-bin.stable.latest.default`)
- Node.js (for NAPI module consumption)
- WebAssembly target support (wasm32-unknown-unknown included in Nix setup)

**Package Manager:**
- Cargo (Rust)
- No lockfile needed - Cargo.lock is auto-generated from Cargo.toml

## Frameworks

**Core:**
- Oxc (Oxide JavaScript Compiler) 0.94.0 - JavaScript/TypeScript parsing, transformation, and codegen
  - Includes: Parser, AST, Semantic Analysis, Codegen, Minifier, Transformer
  - Language support: TypeScript, JSX transpilation

**Bindings:**
- NAPI 2 - Node.js native bindings for exposing Rust functionality to JavaScript
- napi-derive 2 - Procedural macros for NAPI function binding
- tokio 1 - Async runtime with multi-threading support for CPU-intensive tasks

**Testing:**
- insta 1.42.1 - Snapshot testing with YAML output format
- glob 0.3.2 - Test file discovery

**Build/Dev:**
- napi-build 2 - Build helper for NAPI modules
- mimalloc 0.1.25 - Memory allocator optimization (Windows only)

## Key Dependencies

**Critical:**
- oxc_parser 0.94.0 - JavaScript/TypeScript parser foundation
- oxc_semantic 0.94.0 - Semantic analysis for scope and symbol resolution
- oxc_transformer 0.94.0 - Code transformation with JSX/TypeScript support
- oxc_codegen 0.94.0 - Code generation from AST back to JavaScript
- thiserror 2.0.11 - Error type derivation and handling
- serde 1.0 - Serialization framework for data interchange
- serde_json 1.0 - JSON serialization/deserialization

**Infrastructure:**
- base64 0.22.1 - Base64 encoding for source maps
- pathdiff 0.2.1 - Path relative difference calculation
- napi 2 - JavaScript/Rust interop layer with serde-json and tokio_rt features
- oxc-browserslist 2.1.2 - Browser target compatibility detection

**Development:**
- insta 1.42.1 (with yaml feature) - Snapshot testing for transform outputs
- glob 0.3.2 - Test project file pattern matching

## Workspace Structure

`Cargo.toml` workspace contains:
- `optimizer/` - Core Qwik optimization engine and AST transformation library
- `napi/` - Node.js NAPI binding that exposes optimizer to JavaScript

## Configuration

**Environment:**
- Configured via direnv and Nix flake
- `.envrc` - direnv integration (`use flake`)
- No environment variables required for operation

**Build:**
- `optimizer/Cargo.toml` - Core library configuration
- `napi/Cargo.toml` - NAPI binding with LTO optimization for release builds
- `napi/napi.config.json` - NAPI naming: exports as "qwik" module with default platform triples
- `napi/build.rs` - NAPI setup invocation

## Platform Requirements

**Development:**
- Rust stable toolchain (via `nix develop`)
- Git (minimal installation from nixpkgs)
- Bash shell
- Optional: direnv for automatic environment loading
- Target support: WASM (wasm32-unknown-unknown) included in Nix setup

**Production:**
- Node.js runtime (14+) - Required for NAPI module
- Platform-specific binaries: Compiled for x86_64, aarch64, and other platforms via NAPI multi-platform build
- Windows-specific: mimalloc allocator for performance

## Build Output

**Artifacts:**
- NAPI module: `qwik` (native binary module for Node.js)
- Rust library: Loadable as crate in `optimizer/`
- Source maps: Optional inline base64-encoded source maps in transform output

---

*Stack analysis: 2026-01-29*
