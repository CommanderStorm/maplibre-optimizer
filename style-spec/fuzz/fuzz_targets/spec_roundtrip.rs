#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use maplibre_style_spec::spec::MaplibreStyleSpecification;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    let mut u = Unstructured::new(data);
    let Ok(v) = MaplibreStyleSpecification::arbitrary(&mut u) else {
        return;
    };
    let json = serde_json::to_value(&v).expect("serialize must not fail");
    let w: MaplibreStyleSpecification = serde_json::from_value(json.clone())
        .unwrap_or_else(|e| panic!("deserialize of own serialization failed: {e}\njson: {json}"));
    pretty_assertions::assert_eq!(v, w);
});
