# What This Is

A research prototype that optimizes MapLibre style JSON documents and acoompaniying data.
It parses styles into typed Rust structs (generated from the upstream `v8.json` spec), applies optimization passes, and serializes back to JSON.
The optimizer does **not** support legacy MapLibre filter syntax.

## Build & Development Commands

This project uses `just` as task runner. Run `just` to see all available recipes.

| Task | Command |
|---|---|
| Build / type-check | `just check` |
| Run all tests | `just test` |
| Clippy | `just clippy` |
| Format | `just fmt` |
| Clippy | `just clippy` |
| Full CI suite locally | `CI=true just ci-test` |
| Regenerate all snapshots and artefacts | `just bless` |
| Fuzz testing | `just fuzz` |
| Render tests | `just render-test-setup` then `just render-test` |
| Benchmarks | `just bench`, `just bench-plot` |

## Workspace Structure

Three crates in the workspace:

- **`maplibre-style-spec`** — Spec types and code generator. Has a `full` feature flag that gates the generated `spec` module, `validate`, and `expression_validate`. When regenerating (`just gen`), the binary runs with `--no-default-features` so it compiles without the (possibly stale) generated code.
- **`maplibre-style-optimizer`** — The optimizer binary and library. Depends on `maplibre-style-spec` with `full`. Entry point: `optimize_style()`.
- **`codegen2`** — Shared code-generation utilities used by the spec generator.

The `upstream/` directory is a git submodule of the MapLibre GL JS repo (contains `v8.json`).

## Code Generation Pipeline (maplibre-style-spec)

`just gen` drives this flow:

1. **Decode**: `decoder/` deserializes the upstream `v8.json` (`StyleReference`).
2. **MIR**: `mir/` normalizes the decoded data into `IntermediateSpec` — a stable compiler IR that resolves cross-cutting concerns (layers combine data from multiple spec sections, expressions need special handling, etc.). Preprocessing passes live in `mir/preprocessing/`.
3. **Generate**: `generator/` consumes `IntermediateSpec` and emits Rust source files into `src/spec/`. These generated types derive `Serialize`/`Deserialize` and optionally `Arbitrary` (behind the `fuzz` feature).

The generated `spec/` directory is checked into git but can be fully regenerated. After regeneration, run `just fmt` and update snapshots with `just bless`.

# Optimizer Architecture (maplibre-style-optimizer)

`optimize/mod.rs` defines `OptPasses` and orchestrates passes over typed `MaplibreStyleSpecification`. Individual passes in `optimize/`:

- `dead.rs` — dead layer/source elimination
- `defaults.rs` — strip default values
- `expr.rs` — expression simplification
- `metadata.rs` — metadata stripping
- `selectivity.rs` — selectivity-based optimizations
- `source_util.rs` — source utilities
- `strip.rs` — general stripping
- `walk.rs` — tree-walking utilities

`stats.rs` / `TileStatistics` provides tile-level statistics used by some passes.

# Key Conventions

- Rust edition 2024, MSRV 1.92. `unsafe` is forbidden workspace-wide.
- Treat warnings as errors. Clippy pedantic is enabled.
- Always use `uv` for Python tooling (benchmarks).
- Use `#[expect(...)]` instead of `#[allow(...)]` for suppressing lints.
- Use inline comments to explain "why", not "what". Only comment non-obvious logic.
- Handle all edge cases. Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.
- Snapshot tests use `insta` — update with `just bless`.
- Avoid `#[serde(untagged)]` for deserializers (poor error messages); write custom visitors instead.
