# Spec compliance after deserialization

## Goal

After `serde` deserialization into our style model, styles must be **spec-valid** in the same sense as upstream MapLibre style-spec validation:
invalid documents must be rejected, valid ones accepted.
This is **binary parity** with `upstream/test/integration/style-spec` fixtures (empty `*.output.json` array = valid; non-empty = invalid).
Error **messages** do not need to match upstream.

Upstream corpus: `../upstream/test/integration/style-spec/tests/*.input.json`, expectations in paired `*.output.json`.

Harness today: `tests/style_spec_reject_parity.rs` uses a temporary `ModeledStyleSpec` for bootstrapping. **Real compliance work** lands on the generated `src/spec.rs` (`MaplibreStyleSpecification` and nested types) and/or an explicit **validate** step run immediately after decode.

## Verification (prefer `just`)

Use the workspace [`justfile`](../justfile) from the repo root; prefer **`just`** over calling **`cargo`** directly so behavior stays aligned with local/CI conventions.

- **Full workspace tests:** `just test` — runs `test-cargo --all-targets` then doctests (`test-doc`).
- **Regenerate `spec.rs`:** `just gen` — stubs/cleans the generated file, runs `generate_spec` (`cargo run --no-default-features -p maplibre-style-spec --bin generate_spec`), then workspace fmt and `rustfmt` on `spec.rs`.

## Current blocker: `just test`

**`just test` fails today** in `maplibre-style-spec`: the generated **`spec::test`** module in [`src/spec.rs`](src/spec.rs) has on the order of **30+ failing** `test_example_*_decodes` cases (library tests, not the parity integration tests). Typical errors are serde “unknown variant …” when an example’s leading operator is not a variant of that output-type enum.

Rough causes:

1. **Per-output-type expression enums vs spec examples** — Codegen emits `rstest` decode checks from every `example` on each typedef ([`src/generator/autotest.rs`](src/generator/autotest.rs), used from [`src/generator/items/enum/syntax.rs`](src/generator/items/enum/syntax.rs)). The reference attaches examples whose operator belongs to **another** output bucket (e.g. `==`, comparisons, `+`, `interpolate`, `*`), so deserializing into `String` / `Boolean` / `Number` / … enums fails.
2. **Root composite examples** — Single-shot tests such as `test_example_root_light_decodes`, `test_example_root_projection_decodes`, and `test_example_root_sky_decodes` use upstream JSON that does not yet match the generated `RootLight` / `RootProjection` / `RootSky` shapes.

Treat **`just test-cargo '-p maplibre-style-spec --lib'`** as an early gate (fix codegen filtering, widen types, or adjust examples) so **`just test`** can succeed for the workspace.

## Test surfaces

Three layers; all should eventually be green:

1. **Library example tests** (generated in `spec.rs`) — Spec examples must deserialize into the generated types; failures indicate generator or modeling drift.
2. **`style_spec_reject_parity`** — Binary valid/invalid vs `upstream/test/integration/style-spec` fixtures.
3. **`expression_reject_parity`** — Expression compile success/failure vs the upstream expression integration corpus.

Until (1) passes, **`just test` fails in this crate** and the full workspace recipe does not complete cleanly.

## Modeling principles (Rust)

- **Tight, expressive interiors** — Prefer newtypes, enums, fixed-shape structs, and array-length semantics the spec allows. The goal is to model the **full problem space** in the type system so invalid states are unrepresentable or fail at decode/validate, not a loose `serde_json::Value` bucket.
- **Sum types for real alternatives** — Layers (`type` + paint/layout), sources, filters, curve specs, format sections, and expressions should use `enum` / tagged shapes that mirror the spec, not “anything goes” maps.
- **`Value` only at documented boundaries** — Reserve `serde_json::Value` (or similar) for places the spec truly admits arbitrary JSON (e.g. some `metadata` payloads), not for paint, layout, expressions, or filters because it hides bugs and fails parity tests.
- **Deserialize + validate** — Use custom `Deserialize` where it stays readable; use **`validate(&self)`** (or fallible `try_from`) for cross-field rules, ranges, URL templates, and constraints the reference encodes outside a single field.
- **Expressions** — Generated per-output-type syntax enums are the right direction; parity with upstream requires **type-checking** (and eventually evaluation) so interiors match compiled `type` / arity / domain rules, not only nested-array shape.
- **Integration tests** — `style_spec_reject_parity` / `expression_reject_parity` track parity; `ModeledStyleSpec` and the expression structural walker are **temporary** and should converge on the same generated + validated model.

## Current parity gap (as of last harness run)

**22 / 37** fixtures mismatch. The 15 fixtures not listed here already agree on valid vs invalid with the current harness.

Re-run parity after **`just test-cargo '-p maplibre-style-spec --lib'`** is green; the table reflects the harness state at last run, not necessarily current CI.

### Inventory (fixture → upstream expectation summary)

| Fixture | Upstream | Harness today | Upstream message (first error, or note) |
|--------|----------|---------------|----------------------------------------|
| `bad-color.input.json` | invalid | accepts | `layers[0].paint.fill-color: color expected, array found` |
| `center.input.json` | invalid | accepts | `center: array length 2 expected, length 3 found` |
| `extrakeys.input.json` | valid | rejects | Extra top-level keys must be allowed (unknown keys at root). |
| `filters.input.json` | invalid | accepts | `layers[0].filter: array expected, object found` |
| `font-faces-valid-array.input.json` | valid | rejects | Root must allow `font-faces`. |
| `font-faces-valid-empty.input.json` | valid | rejects | Same. |
| `font-faces-valid-string.input.json` | valid | rejects | Same. |
| `functions.input.json` | invalid | accepts | `layers[0].paint.line-width.base: number expected, string found` |
| `layers.input.json` | invalid | accepts | `layers[0]: either "type" or "ref" is required` |
| `light-arbitrary.input.json` | invalid | accepts | `foo: unknown property "foo"` |
| `light-malformed-color.input.json` | invalid | accepts | `color: color expected, "__proto__" found` |
| `light.input.json` | invalid | accepts | `anchor: expected one of [map, viewport], true found` |
| `malformed-glyphs-type.input.json` | invalid | accepts | `glyphs: string expected, boolean found` |
| `malformed-glyphs.input.json` | invalid | accepts | `glyphs: "glyphs" url must include a "{fontstack}" token` |
| `numbers.input.json` | invalid | accepts | `layers[2].paint.circle-radius: -1 is less than the minimum value 0` |
| `pitch.input.json` | invalid | accepts | `pitch: number expected, string found` |
| `projection.input.json` | valid | rejects | Root must allow `projection`. |
| `properties.input.json` | invalid | accepts | `layers[0].paint.fill-opacity: -1 is less than the minimum value 0` |
| `sources.input.json` | invalid | accepts | `sources.missing-type: "type" is required` |
| `terrain.input.json` | invalid | accepts | `source: string expected, number found` |
| `text-field-format.input.json` | invalid | accepts | `layers[1].layout.text-field: First argument must be an image or text section.` |
| `text-font.input.json` | invalid | accepts | `layers[4].layout.text-font: Invalid data expression for "text-font"… literals within the expression.` |

## Root causes (grouped)

### A — Root document shape ( serde + spec )

1. **Unknown top-level keys** must deserialize (and validate) per spec tolerance: e.g. `extrakeys` should stay valid.
2. **Missing root properties in the model** block valid styles: `font-faces`, `projection` (and any other v8 root keys the spec allows that we omit).

*Fix:* Ensure the generator emits **complete root fields** from MIR/reference. For **root-level** unknown keys, behavior must match upstream (see **Extra root keys (`extrakeys`)** below), not `deny_unknown_fields` on the whole style unless paired with an explicit extension map.

### Extra root keys (`extrakeys` fixture)

Integration test: `upstream/test/integration/style-spec/tests/extrakeys.input.json` adds a top-level object `extrakey: { "foo": "bar" }` beside normal fields. Upstream reports **no errors** (style is valid).

**In the reference (`v8.json`):**

- `$root` lists **named** root properties (`version`, `sources`, `layers`, `metadata`, …). There is **no** `"*"` entry under `$root` that means “any extra root key.”
- The **`metadata`** field is typed as `"type": "*"` in JSON — that means “arbitrary JSON object for this **property name**,” modelled in the decoder as [`PrimitiveType::Star`](src/decoder/mod.rs) (`#[serde(rename = "*")]`). That is **only** for the key `metadata`, not for arbitrary sibling keys.

**In upstream validation (maplibre-gl-style-spec):**

- [`validate_style.min.ts`](../upstream/src/validate_style.min.ts) validates the style with `valueSpec: styleSpec.$root` and passes **`objectElementValidators['*']`** which is `() => []` — i.e. **any root-level property that does not match a named `$root` entry is still “validated” by a no-op** and produces **no errors**.
- So permissive root keys are **wired in the validator**, not expressed as an extra `$root["*"]` field in `v8.json`. (Exception: `glyphs` uses a dedicated `validateGlyphsUrl` in the same map.)

**In this repo’s decoder:**

- [`StyleReference`](src/decoder/mod.rs) parses the **spec document**: `#[serde(rename = "$root")] pub root: BTreeMap<String, ParsedItem>` plus `#[serde(flatten)] pub fields`. That is the **reference** layout, not a user style.
- [`IntermediateRootPrimitives`](src/mir/root.rs) is built only from keys present under `$root` in the reference; there is **no** generated “catch-all root field” from the spec JSON for **user styles**.

**In generated user style types (`spec.rs`):**

- [`MaplibreStyleSpecification`](src/spec.rs) has a **fixed set of fields** for known root keys. Serde’s default for structs is to **ignore unknown properties** on deserialize unless the struct opts into **`#[serde(deny_unknown_fields)]`**.
- So **unknown root keys are dropped at decode** and do not fail parsing — broadly aligned with “accept + no error,” but the **values are not retained** unless you add e.g. `#[serde(flatten)] pub extensions: BTreeMap<String, serde_json::Value>` (or a typed bag) when you need round-trip or tooling that preserves them.

**Parity note:** The temporary [`ModeledStyleSpec`](tests/style_spec_reject_parity.rs) used `deny_unknown_fields`, which **rejects** `extrakeys` — that is **harness-only** and does not match upstream or the default `spec.rs` derive.

### B — Weakly typed interiors (`Value`-like or too-wide enums)

Nested correctness is not enforced after decode, so invalid layer paint/layout, filters, light, terrain, glyphs URL rules, numeric ranges, color coercion, and expression constraints pass through.

*Fix:* Per **Modeling principles (Rust)** above: tighten **generator output** first (expressive types), then add **`validate`** only where the spec requires logic serde cannot encode. Prefer making illegal states **unrepresentable** over accepting and hoping a later pass catches errors.

### C — Harness skew (temporary)

`ModeledStyleSpec` in the integration test is **not** the shipping model. Some “valid” mismatches are **harness-only** (`extrakeys`, `font-faces`, `projection`) until either:

- the harness deserializes into `MaplibreStyleSpecification`, or
- a shared `parse_and_validate_style(json) -> Result<_, _>` API wraps codegen output.

Once `spec.rs` compiles cleanly in the test crate (or is re-exported from `lib.rs`), point the harness at the real type + validation.

## Recommended work order

1. **Unblock real type in tests** — Export or include generated `MaplibreStyleSpecification` in a way tests can compile against; fix generator issues if `spec.rs` fails to build in isolation.
2. **Root completeness + unknown keys** — Regenerate root with `font-faces`, `projection`, … and policy for extra keys consistent with upstream.
3. **High-leverage validators** — Sources (`type` required), layer shell (`type`/`ref`), root `center` arity, `pitch`/`glyphs` types, numeric min/max on common paint properties.
4. **Expressions & layout semantics** — `text-field` format sections, `text-font` data-expression literal rules, filter JSON shape, `light` object strictness.
5. **Re-run** `cargo test -p maplibre-style-spec --test style_spec_reject_parity` until **0 mismatches**.
6. **Expressions** — Extend validation / generated expression types; re-run `cargo test -p maplibre-style-spec --test expression_reject_parity` until **0 mismatches**.

## Expression integration harness

Corpus: `../upstream/test/integration/expression/tests/**/test.json` (same as `../upstream/test/integration/expression/expression.test.ts`).

Test: `tests/expression_reject_parity.rs` compares upstream **compile** success vs failure only (`expected.compiled.result`).
**Actual** side uses structural checks plus operator names from `IntermediateSpec` (and `error`), with special cases for `literal`, curve arrays like `["linear"]`, and JSON-valued operands (`collator`, `distance`, `within`, `format`).
It does **not** type-check or evaluate; remaining gaps show up as mismatches until the expression pipeline is spec-complete.

## References

- Upstream style-spec runner: `../upstream/test/integration/style-spec/validate_spec.test.ts`
- Upstream expression runner: `../upstream/test/integration/expression/expression.test.ts`
- Generated model: `src/spec.rs` (regenerate with **`just gen`** from repo root)
- Style fixture parity test: `tests/style_spec_reject_parity.rs`
- Expression fixture parity test: `tests/expression_reject_parity.rs`
