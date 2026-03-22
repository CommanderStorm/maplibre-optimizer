//! Property: applying all optimizer passes twice produces the same result as applying them once.
//!
//! This catches convergence bugs in the fixpoint loop and any pass that fails to reach a
//! stable output on its first application.

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
        "insufficient bytes for arbitrary generation",
        |bytes| {
            MaplibreStyleSpecification::arbitrary(&mut arbitrary::Unstructured::new(&bytes)).ok()
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
        let after_first = style.clone();
        optimize_style(&mut style, mir(), &passes, None);
        prop_assert_eq!(after_first, style);
    }
}
