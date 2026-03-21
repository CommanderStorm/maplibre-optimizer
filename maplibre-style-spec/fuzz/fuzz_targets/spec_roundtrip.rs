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

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(v) = MaplibreStyleSpecification::arbitrary(&mut u) else {
        return;
    };
    let json = serde_json::to_value(&v).expect("serialize must not fail");
    let w: MaplibreStyleSpecification = serde_json::from_value(json.clone())
        .unwrap_or_else(|e| panic!("deserialize of own serialization failed: {e}\njson: {json}"));
    pretty_assertions::assert_eq!(v, w);
});
