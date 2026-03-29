//! Property: applying all optimizer passes twice produces the same JSON output as once.
//!
//! Compares JSON-serialized output (the format that is actually written to disk)
//! rather than typed structs, because some complex expression forms have multiple
//! valid typed representations that are semantically equivalent.

use std::path::Path;
use std::sync::OnceLock;

use arbitrary::Arbitrary;
use maplibre_style_optimizer::{OptPasses, load_intermediate_spec_from_v8_path, optimize_style};
use maplibre_style_spec::mir::MirSpec;
use maplibre_style_spec::spec::MaplibreStyleSpecification;
use proptest::prelude::*;

fn mir() -> &'static MirSpec {
    static MIR: OnceLock<MirSpec> = OnceLock::new();
    MIR.get_or_init(|| {
        let v8 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
        load_intermediate_spec_from_v8_path(&v8).expect("load v8.json")
    })
}

/// Generate a `MaplibreStyleSpecification` from a raw byte buffer.
///
/// Uses `filter_map` (graceful reject) rather than abort when the byte buffer is too small
/// to produce a complete value — this avoids aborting the whole proptest run.
fn arbitrary_style() -> impl Strategy<Value = MaplibreStyleSpecification> {
    proptest::collection::vec(any::<u8>(), 0..2048).prop_filter_map(
        "insufficient bytes or NaN/infinity in generated style",
        |bytes| {
            let style =
                MaplibreStyleSpecification::arbitrary(&mut arbitrary::Unstructured::new(&bytes))
                    .ok()?;
            // Reject styles containing NaN or infinity: JSON has no representation for them,
            // and NaN != NaN under IEEE 754 causes false PartialEq failures.
            serde_json::to_string(&style).ok()?;
            Some(style)
        },
    )
}

proptest! {
    // Store failure cases alongside this test file rather than searching for main.rs/lib.rs.
    #![proptest_config(ProptestConfig::with_cases(4096))]

    #[test]
    fn optimizer_is_idempotent(mut style in arbitrary_style()) {
        let passes = OptPasses::all();
        optimize_style(&mut style, mir(), &passes, None);
        let after_first = serde_json::to_value(&style).unwrap();
        optimize_style(&mut style, mir(), &passes, None);
        let after_second = serde_json::to_value(&style).unwrap();
        prop_assert_eq!(after_first, after_second);
    }
}
