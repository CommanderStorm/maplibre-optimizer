//! JSON serialize → deserialize round-trip for [`MaplibreStyleSpecification`].
//!
//! Run with a nightly toolchain (required by cargo-fuzz), e.g.:
//! `cd maplibre-style-spec/fuzz && cargo +nightly fuzz run spec_roundtrip`
//!
//! If the default fuzz target fails to link on Linux (e.g. musl), see `fuzz/README.md`.
//!
//! Install: `cargo install cargo-fuzz`

#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use maplibre_style_spec::spec::MaplibreStyleSpecification;
use pretty_assertions::assert_eq;

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(v) = MaplibreStyleSpecification::arbitrary(&mut u) else {
        return;
    };
    let Ok(json) = serde_json::to_value(&v) else {
        return;
    };
    let Ok(w) = serde_json::from_value::<MaplibreStyleSpecification>(json) else {
        return;
    };
    assert_eq!(v, w);
});
