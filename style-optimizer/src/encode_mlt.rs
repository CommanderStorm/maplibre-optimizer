//! MVT → MLT encoding bridge.
//!
//! Converts raw MVT protobuf bytes into MLT (`MapLibre` Tiles) binary format
//! using `mlt-core`.

use anyhow::Context;
use mlt_core::encoder::EncoderConfig;

/// Convert raw MVT protobuf bytes into MLT binary format.
///
/// Pipeline:
/// 1. Parse MVT via `mlt_core::mvt::mvt_to_tile_layers`
/// 2. For each layer, encode to MLT using the automatic optimizer
/// 3. Concatenate all layer bytes
pub fn mvt_to_mlt(mvt_bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let layers =
        mlt_core::mvt::mvt_to_tile_layers(mvt_bytes).context("parse MVT for MLT conversion")?;

    let cfg = EncoderConfig::default();
    let mut output = Vec::new();

    for layer in layers {
        let layer_bytes = layer.encode(cfg).context("MLT layer encoding")?;
        output.extend_from_slice(&layer_bytes);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::*;
    use crate::mvt;

    /// Build a minimal MVT tile, encode to bytes, convert to MLT, and verify
    /// the result can be parsed back by `mlt-core`.
    #[test]
    fn roundtrip_simple_tile() {
        let tile = mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "test".to_string(),
                features: vec![mvt::tile::Feature {
                    id: Some(1),
                    tags: vec![0, 0],
                    r#type: Some(mvt::tile::GeomType::Point.into()),
                    // MoveTo(1) at (10, 20) in MVT command encoding.
                    geometry: vec![
                        (1 << 3) | 1, // command: MoveTo, count=1
                        20,           // zigzag(10)
                        40,           // zigzag(20)
                    ],
                }],
                keys: vec!["name".to_string()],
                values: vec![mvt::tile::Value {
                    string_value: Some("hello".to_string()),
                    ..Default::default()
                }],
                extent: Some(4096),
            }],
        };

        let encoded = tile.encode_to_vec();
        let mlt_result = mvt_to_mlt(encoded).unwrap();

        // Verify we can parse the MLT output.
        assert!(!mlt_result.is_empty());
        let mut parser = mlt_core::Parser::default();
        let layers = parser.parse_layers(&mlt_result).unwrap();
        assert_eq!(layers.len(), 1);
    }
}
