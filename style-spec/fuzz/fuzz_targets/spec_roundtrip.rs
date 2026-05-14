#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use maplibre_style_spec::spec::MaplibreStyleSpecification;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    let mut u = Unstructured::new(data);
    let Ok(raw) = MaplibreStyleSpecification::arbitrary(&mut u) else {
        return;
    };
    // Canonicalize: Arbitrary can produce non-canonical enum variants (e.g.
    // Interpolate(Step(…)) that deserialization would parse as Color(AnyExpr(Step(…)))),
    // or structurally invalid expressions (e.g. Match with zero arms) that serialize
    // to JSON the deserializer rejects.  A single roundtrip normalizes to the form
    // serde would naturally produce; if it fails, the input isn't roundtrippable.
    let json1 = serde_json::to_value(&raw).expect("serialize must not fail");
    let Ok(v) = serde_json::from_value::<MaplibreStyleSpecification>(json1) else {
        return;
    };
    // Now assert the canonical form roundtrips stably.
    let json2 = serde_json::to_value(&v).expect("re-serialize must not fail");
    let w: MaplibreStyleSpecification = serde_json::from_value(json2.clone())
        .unwrap_or_else(|e| {
            panic!("deserialize of canonical serialization failed: {e}\njson: {json2}")
        });
    pretty_assertions::assert_eq!(v, w);
});
