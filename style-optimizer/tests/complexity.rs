//! Snapshot tests for the complexity report.

use std::path::Path;

use insta::assert_yaml_snapshot;
use maplibre_style_optimizer::complexity::complexity_report;
use maplibre_style_optimizer::load_intermediate_spec_from_v8_path;
use maplibre_style_spec::mir::MirSpec;
use serde_json::json;

fn sample_mir() -> MirSpec {
    let v8 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/src/reference/v8.json");
    load_intermediate_spec_from_v8_path(&v8).expect("load v8.json")
}

#[test]
fn complexity_report_basic() {
    let mir = sample_mir();
    let mut style = json!({
        "version": 8,
        "sources": {},
        "layers": [
            {
                "id": "water",
                "type": "fill",
                "source": "openmaptiles",
                "source-layer": "water",
                "filter": ["==", ["get", "class"], "ocean"],
                "paint": {
                    "fill-color": ["interpolate", ["linear"], ["zoom"], 0, "#aad", 10, "#88b"]
                }
            },
            {
                "id": "road",
                "type": "line",
                "source": "openmaptiles",
                "source-layer": "transportation",
                "filter": ["all",
                    ["==", ["geometry-type"], "LineString"],
                    ["match", ["get", "class"], ["motorway", "trunk"], true, false]
                ],
                "paint": {
                    "line-width": ["step", ["zoom"], 1, 10, 2, 14, 4]
                },
                "layout": {
                    "line-cap": "round"
                }
            },
            {
                "id": "background",
                "type": "background",
                "paint": {
                    "background-color": "#f8f4f0"
                }
            }
        ]
    });

    let report = complexity_report(&mut style, &mir);
    assert_yaml_snapshot!(report);
}

#[test]
fn complexity_report_empty_style() {
    let mir = sample_mir();
    let mut style = json!({
        "version": 8,
        "sources": {},
        "layers": []
    });

    let report = complexity_report(&mut style, &mir);
    assert_yaml_snapshot!(report);
}
