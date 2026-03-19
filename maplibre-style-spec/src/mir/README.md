# Why MIR exists

`v8.json` is an input format, not a compiler IR.

The decoder's job is to deserialize the upstream schema as-is, including irregularities.
Code generation, however, needs a normalized and internally consistent model.

The MIR layer is that model.

## What MIR gives us

- A stable shape for generation (`IntermediateSpec`) independent of decoder quirks.
- Centralized preprocessing for cross-cutting spec concepts (layers, expressions, sources).
- A single place to lower decoded values into codegen-oriented field/type structures.
- Clear ownership boundaries between "decode upstream JSON" and "generate Rust API".

Example: generating layer fields requires combining information spread across multiple
upstream sections (e.g. layer categories and layout/paint/property definitions).
MIR resolves those relationships before generation.

## Current architecture

Current flow:

1. `StyleReference` is decoded from upstream JSON.
2. MIR preprocessing extracts/normalizes complex sections:
   - expressions
   - layers
   - sources
3. Remaining items are lowered into MIR field/type definitions.
4. Generator consumes `IntermediateSpec` to emit `spec.rs`.

This is implemented as:

- `StyleReference -> IntermediateSpec` (`mir/mod.rs`)
- MIR lowering helpers (`mir/lower.rs`)
- MIR preprocessing passes (`mir/preprocessing/*`)
- Generator entrypoint consumes MIR (`generator/mod.rs`)