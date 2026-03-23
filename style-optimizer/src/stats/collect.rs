//! `MBTiles`/MVT reading and statistics accumulation.
//!
//! Reads vector tiles from an `MBTiles` (`SQLite`) file, decodes MVT protobuf data,
//! and produces [`TileStatistics`].

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, ensure};
use indexmap::IndexMap;
use rand::Rng;
use rusqlite::Connection;

use super::{
    CARDINALITY_THRESHOLD, GeometryTypeStats, LayerStats, PropertyStats, SourceStats,
    TileStatistics,
};

use crate::mvt;

/// Open an `MBTiles` file and validate it has the expected `tiles` table.
pub fn open_mbtiles(path: &Path) -> anyhow::Result<Connection> {
    let conn =
        Connection::open(path).with_context(|| format!("open MBTiles {}", path.display()))?;

    // Validate that the tiles table exists.
    let has_tiles: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='tiles'",
            [],
            |row| row.get(0),
        )
        .context("check tiles table")?;
    ensure!(has_tiles, "MBTiles file missing 'tiles' table");
    Ok(conn)
}

/// Query distinct zoom levels present in the `MBTiles` file.
pub fn available_zoom_levels(conn: &Connection) -> anyhow::Result<Vec<u8>> {
    let mut stmt = conn.prepare("SELECT DISTINCT zoom_level FROM tiles ORDER BY zoom_level")?;
    let rows = stmt.query_map([], |row| {
        let z: i32 = row.get(0)?;
        Ok(u8::try_from(z).unwrap_or(u8::MAX))
    })?;
    let mut levels = Vec::new();
    for row in rows {
        levels.push(row?);
    }
    Ok(levels)
}

/// Decode a raw tile blob (possibly gzip-compressed) into an MVT `Tile`.
pub fn decode_tile(data: &[u8]) -> anyhow::Result<mvt::Tile> {
    use prost::Message;

    let decompressed;
    let bytes = if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
        // Gzip-compressed
        let mut decoder = flate2::read::GzDecoder::new(data);
        decompressed = {
            let mut buf = Vec::new();
            decoder
                .read_to_end(&mut buf)
                .context("decompress gzip tile")?;
            buf
        };
        &decompressed[..]
    } else {
        data
    };

    mvt::Tile::decode(bytes).context("decode MVT protobuf")
}

/// Collect statistics from an `MBTiles` file for a given source name.
///
/// `zoom_levels` specifies which zoom levels to scan. `sample_rate` (0.0–1.0) controls
/// random sampling: 1.0 means scan all tiles.
pub fn collect_statistics(
    conn: &Connection,
    source_name: &str,
    zoom_levels: &[u8],
    sample_rate: f64,
) -> anyhow::Result<TileStatistics> {
    let mut layers: BTreeMap<String, LayerStatsAccumulator> = BTreeMap::new();
    let mut rng = rand::rng();

    for &zoom in zoom_levels {
        let mut stmt = conn.prepare("SELECT tile_data FROM tiles WHERE zoom_level = ?1")?;
        let rows = stmt.query_map([i32::from(zoom)], |row| {
            let data: Vec<u8> = row.get(0)?;
            Ok(data)
        })?;

        for row in rows {
            let data = row?;

            if sample_rate < 1.0 && rng.random::<f64>() > sample_rate {
                continue;
            }

            let tile = match decode_tile(&data) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("warning: skipping tile at z{zoom}: {e}");
                    continue;
                }
            };

            for mvt_layer in &tile.layers {
                let acc = layers.entry(mvt_layer.name.clone()).or_default();

                for feature in &mvt_layer.features {
                    acc.total_features += 1;
                    *acc.features_by_zoom.entry(zoom).or_insert(0) += 1;

                    // Geometry type
                    match feature.r#type() {
                        mvt::tile::GeomType::Point => acc.geometry_types.point += 1,
                        mvt::tile::GeomType::Linestring => acc.geometry_types.linestring += 1,
                        mvt::tile::GeomType::Polygon => acc.geometry_types.polygon += 1,
                        mvt::tile::GeomType::Unknown => acc.geometry_types.unknown += 1,
                    }

                    // Feature ID
                    if feature.id.unwrap_or(0) != 0 {
                        acc.has_feature_ids = true;
                    }

                    // Properties from tags (alternating key_index, value_index)
                    let tags = &feature.tags;
                    let mut i = 0;
                    while i + 1 < tags.len() {
                        let key_idx = tags[i] as usize;
                        let val_idx = tags[i + 1] as usize;
                        i += 2;

                        let Some(key) = mvt_layer.keys.get(key_idx) else {
                            continue;
                        };
                        let Some(value) = mvt_layer.values.get(val_idx) else {
                            continue;
                        };

                        let prop_acc = acc
                            .properties
                            .entry(key.clone())
                            .or_insert_with(PropertyStatsAccumulator::new);
                        prop_acc.observe(value);
                    }
                }
            }
        }
    }

    let mut source_layers = BTreeMap::new();
    for (name, acc) in layers {
        source_layers.insert(name, acc.finish());
    }

    let mut sources = BTreeMap::new();
    sources.insert(
        source_name.to_string(),
        SourceStats {
            layers: source_layers,
        },
    );

    Ok(TileStatistics { sources })
}

// ── Accumulators ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct LayerStatsAccumulator {
    total_features: u64,
    features_by_zoom: BTreeMap<u8, u64>,
    geometry_types: GeometryTypeStats,
    has_feature_ids: bool,
    properties: BTreeMap<String, PropertyStatsAccumulator>,
}

impl LayerStatsAccumulator {
    fn finish(self) -> LayerStats {
        let mut properties = BTreeMap::new();
        for (key, acc) in self.properties {
            properties.insert(key, acc.finish());
        }
        LayerStats {
            total_features: self.total_features,
            features_by_zoom: self.features_by_zoom,
            geometry_types: self.geometry_types,
            has_feature_ids: self.has_feature_ids,
            properties,
        }
    }
}

/// Detected MVT value type for accumulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectedType {
    Bool,
    Integer,
    UnsignedInteger,
    Double,
    String,
    Mixed,
}

/// Accumulates statistics for a single property key across all features.
pub(crate) struct PropertyStatsAccumulator {
    detected_type: Option<DetectedType>,
    present_count: u64,
    // Bool tracking
    true_count: u64,
    // Integer tracking
    int_min: i64,
    int_max: i64,
    int_values: Option<BTreeMap<i64, u64>>,
    int_cardinality: u64,
    // Unsigned integer tracking
    uint_min: u64,
    uint_max: u64,
    uint_values: Option<BTreeMap<u64, u64>>,
    uint_cardinality: u64,
    // Double tracking
    double_min: f64,
    double_max: f64,
    double_cardinality: u64,
    // String tracking
    string_values: Option<BTreeMap<String, u64>>,
    string_cardinality: u64,
    // Mixed tracking
    mixed_cardinality: u64,
}

impl PropertyStatsAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            detected_type: None,
            present_count: 0,
            true_count: 0,
            int_min: i64::MAX,
            int_max: i64::MIN,
            int_values: Some(BTreeMap::new()),
            int_cardinality: 0,
            uint_min: u64::MAX,
            uint_max: u64::MIN,
            uint_values: Some(BTreeMap::new()),
            uint_cardinality: 0,
            double_min: f64::INFINITY,
            double_max: f64::NEG_INFINITY,
            double_cardinality: 0,
            string_values: Some(BTreeMap::new()),
            string_cardinality: 0,
            mixed_cardinality: 0,
        }
    }

    /// Observe a single MVT `Value` for this property.
    pub(crate) fn observe(&mut self, value: &mvt::tile::Value) {
        self.present_count += 1;

        let new_type = if value.string_value.is_some() {
            DetectedType::String
        } else if value.bool_value.is_some() {
            DetectedType::Bool
        } else if value.int_value.is_some() || value.sint_value.is_some() {
            DetectedType::Integer
        } else if value.uint_value.is_some() {
            DetectedType::UnsignedInteger
        } else if value.float_value.is_some() || value.double_value.is_some() {
            DetectedType::Double
        } else {
            // No known value field set; treat as string with empty value.
            DetectedType::String
        };

        match self.detected_type {
            None => self.detected_type = Some(new_type),
            Some(current) if current != new_type && current != DetectedType::Mixed => {
                self.promote_to_mixed();
            }
            _ => {}
        }

        match self.detected_type.unwrap_or(DetectedType::Mixed) {
            DetectedType::Bool => {
                if value.bool_value == Some(true) {
                    self.true_count += 1;
                }
            }
            DetectedType::Integer => {
                let v = value.int_value.or(value.sint_value).unwrap_or(0);
                self.int_min = self.int_min.min(v);
                self.int_max = self.int_max.max(v);
                if let Some(ref mut counts) = self.int_values {
                    let entry = counts.entry(v).or_insert(0);
                    if *entry == 0 {
                        self.int_cardinality += 1;
                    }
                    *entry += 1;
                    if self.int_cardinality > CARDINALITY_THRESHOLD {
                        self.int_values = None;
                    }
                } else {
                    self.int_cardinality += 1; // approximate
                }
            }
            DetectedType::UnsignedInteger => {
                let v = value.uint_value.unwrap_or(0);
                self.uint_min = self.uint_min.min(v);
                self.uint_max = self.uint_max.max(v);
                if let Some(ref mut counts) = self.uint_values {
                    let entry = counts.entry(v).or_insert(0);
                    if *entry == 0 {
                        self.uint_cardinality += 1;
                    }
                    *entry += 1;
                    if self.uint_cardinality > CARDINALITY_THRESHOLD {
                        self.uint_values = None;
                    }
                } else {
                    self.uint_cardinality += 1;
                }
            }
            DetectedType::Double => {
                let v = value
                    .double_value
                    .or_else(|| value.float_value.map(f64::from))
                    .unwrap_or(0.0);
                self.double_min = self.double_min.min(v);
                self.double_max = self.double_max.max(v);
                self.double_cardinality += 1; // approximate for doubles
            }
            DetectedType::String => {
                let v = value.string_value.as_deref().unwrap_or("");
                if let Some(ref mut counts) = self.string_values {
                    let entry = counts.entry(v.to_string()).or_insert(0);
                    if *entry == 0 {
                        self.string_cardinality += 1;
                    }
                    *entry += 1;
                    if self.string_cardinality > CARDINALITY_THRESHOLD {
                        self.string_values = None;
                    }
                } else {
                    self.string_cardinality += 1;
                }
            }
            DetectedType::Mixed => {
                self.mixed_cardinality += 1;
            }
        }
    }

    fn promote_to_mixed(&mut self) {
        // Sum up cardinalities from previous type tracking
        let prev_card = self.int_cardinality
            + self.uint_cardinality
            + self.string_cardinality
            + self.double_cardinality
            + if self.detected_type == Some(DetectedType::Bool) {
                2
            } else {
                0
            };
        self.mixed_cardinality = prev_card;
        self.detected_type = Some(DetectedType::Mixed);
        // Drop type-specific tracking
        self.int_values = None;
        self.uint_values = None;
        self.string_values = None;
    }

    /// Finalize into a [`PropertyStats`].
    pub(crate) fn finish(self) -> PropertyStats {
        match self.detected_type.unwrap_or(DetectedType::Mixed) {
            DetectedType::Bool => PropertyStats::Bool {
                present_count: self.present_count,
                true_count: self.true_count,
            },
            DetectedType::Integer => PropertyStats::Integer {
                present_count: self.present_count,
                min: self.int_min,
                max: self.int_max,
                cardinality: self.int_cardinality,
                value_counts: self.int_values,
            },
            DetectedType::UnsignedInteger => PropertyStats::UnsignedInteger {
                present_count: self.present_count,
                min: self.uint_min,
                max: self.uint_max,
                cardinality: self.uint_cardinality,
                value_counts: self.uint_values,
            },
            DetectedType::Double => PropertyStats::Double {
                present_count: self.present_count,
                min: self.double_min,
                max: self.double_max,
                cardinality: self.double_cardinality,
            },
            DetectedType::String => {
                // Sort string value_counts by descending frequency.
                let value_counts = self.string_values.map(|btree| {
                    let mut pairs: Vec<(String, u64)> = btree.into_iter().collect();
                    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                    pairs.into_iter().collect::<IndexMap<String, u64>>()
                });
                PropertyStats::String {
                    present_count: self.present_count,
                    cardinality: self.string_cardinality,
                    value_counts,
                }
            }
            DetectedType::Mixed => PropertyStats::Mixed {
                present_count: self.present_count,
                cardinality: self.mixed_cardinality,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_string_value(s: &str) -> mvt::tile::Value {
        mvt::tile::Value {
            string_value: Some(s.to_string()),
            ..Default::default()
        }
    }

    fn make_int_value(v: i64) -> mvt::tile::Value {
        mvt::tile::Value {
            int_value: Some(v),
            ..Default::default()
        }
    }

    fn make_bool_value(v: bool) -> mvt::tile::Value {
        mvt::tile::Value {
            bool_value: Some(v),
            ..Default::default()
        }
    }

    fn make_double_value(v: f64) -> mvt::tile::Value {
        mvt::tile::Value {
            double_value: Some(v),
            ..Default::default()
        }
    }

    #[test]
    fn accumulator_single_type_string() {
        let mut acc = PropertyStatsAccumulator::new();
        acc.observe(&make_string_value("a"));
        acc.observe(&make_string_value("b"));
        acc.observe(&make_string_value("a"));

        let stats = acc.finish();
        match &stats {
            PropertyStats::String {
                present_count,
                cardinality,
                value_counts,
            } => {
                assert_eq!(*present_count, 3);
                assert_eq!(*cardinality, 2);
                let vc = value_counts.as_ref().unwrap();
                assert_eq!(vc["a"], 2);
                assert_eq!(vc["b"], 1);
                // Sorted by descending frequency: "a" first.
                assert_eq!(vc.keys().next().unwrap(), "a");
            }
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn accumulator_single_type_integer() {
        let mut acc = PropertyStatsAccumulator::new();
        acc.observe(&make_int_value(10));
        acc.observe(&make_int_value(-5));
        acc.observe(&make_int_value(10));

        let stats = acc.finish();
        match &stats {
            PropertyStats::Integer {
                present_count,
                min,
                max,
                cardinality,
                value_counts,
            } => {
                assert_eq!(*present_count, 3);
                assert_eq!(*min, -5);
                assert_eq!(*max, 10);
                assert_eq!(*cardinality, 2);
                let vc = value_counts.as_ref().unwrap();
                assert_eq!(vc[&10], 2);
                assert_eq!(vc[&-5], 1);
            }
            other => panic!("expected Integer, got {other:?}"),
        }
    }

    #[test]
    fn accumulator_single_type_bool() {
        let mut acc = PropertyStatsAccumulator::new();
        acc.observe(&make_bool_value(true));
        acc.observe(&make_bool_value(false));
        acc.observe(&make_bool_value(true));

        let stats = acc.finish();
        match &stats {
            PropertyStats::Bool {
                present_count,
                true_count,
            } => {
                assert_eq!(*present_count, 3);
                assert_eq!(*true_count, 2);
            }
            other => panic!("expected Bool, got {other:?}"),
        }
    }

    #[test]
    fn accumulator_mixed_type() {
        let mut acc = PropertyStatsAccumulator::new();
        acc.observe(&make_string_value("hello"));
        acc.observe(&make_int_value(42));

        let stats = acc.finish();
        match &stats {
            PropertyStats::Mixed {
                present_count,
                cardinality,
            } => {
                assert_eq!(*present_count, 2);
                assert!(*cardinality >= 1);
            }
            other => panic!("expected Mixed, got {other:?}"),
        }
    }

    #[test]
    fn accumulator_cardinality_threshold() {
        let mut acc = PropertyStatsAccumulator::new();
        for i in 0..=CARDINALITY_THRESHOLD {
            acc.observe(&make_string_value(&format!("val_{i}")));
        }

        let stats = acc.finish();
        match &stats {
            PropertyStats::String {
                cardinality,
                value_counts,
                ..
            } => {
                assert!(*cardinality > CARDINALITY_THRESHOLD);
                assert!(
                    value_counts.is_none(),
                    "should drop value_counts above threshold"
                );
            }
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn accumulator_double() {
        let mut acc = PropertyStatsAccumulator::new();
        acc.observe(&make_double_value(1.5));
        acc.observe(&make_double_value(-0.5));

        let stats = acc.finish();
        match &stats {
            PropertyStats::Double {
                present_count,
                min,
                max,
                ..
            } => {
                assert_eq!(*present_count, 2);
                assert!((*min - (-0.5)).abs() < f64::EPSILON);
                assert!((*max - 1.5).abs() < f64::EPSILON);
            }
            other => panic!("expected Double, got {other:?}"),
        }
    }

    #[test]
    fn integration_mbtiles_roundtrip() {
        use prost::Message;

        // Create an in-memory MBTiles database with a test tile.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB);",
        ).unwrap();

        // Build a minimal MVT tile with one layer and two features.
        let tile = mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "roads".to_string(),
                features: vec![
                    mvt::tile::Feature {
                        id: Some(1),
                        tags: vec![0, 0], // key[0]="class", value[0]="primary"
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                    mvt::tile::Feature {
                        id: Some(2),
                        tags: vec![0, 1], // key[0]="class", value[1]="secondary"
                        r#type: Some(mvt::tile::GeomType::Linestring.into()),
                        geometry: vec![],
                    },
                ],
                keys: vec!["class".to_string()],
                values: vec![
                    mvt::tile::Value {
                        string_value: Some("primary".to_string()),
                        ..Default::default()
                    },
                    mvt::tile::Value {
                        string_value: Some("secondary".to_string()),
                        ..Default::default()
                    },
                ],
                extent: Some(4096),
            }],
        };

        let mut buf = Vec::new();
        tile.encode(&mut buf).unwrap();

        conn.execute(
            "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![10, 0, 0, buf],
        ).unwrap();

        let stats = collect_statistics(&conn, "test_source", &[10], 1.0).unwrap();

        let layer = stats.layer_stats("test_source", "roads").unwrap();
        assert_eq!(layer.total_features, 2);
        assert_eq!(layer.features_by_zoom[&10], 2);
        assert_eq!(layer.geometry_types.linestring, 2);
        assert!(layer.has_feature_ids);

        match &layer.properties["class"] {
            PropertyStats::String {
                present_count,
                cardinality,
                value_counts,
            } => {
                assert_eq!(*present_count, 2);
                assert_eq!(*cardinality, 2);
                let vc = value_counts.as_ref().unwrap();
                assert_eq!(vc["primary"], 1);
                assert_eq!(vc["secondary"], 1);
            }
            other => panic!("expected String, got {other:?}"),
        }
    }

    #[test]
    fn integration_gzip_tile() {
        use std::io::Write;

        use flate2::Compression;
        use flate2::write::GzEncoder;
        use prost::Message;

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB);",
        ).unwrap();

        let tile = mvt::Tile {
            layers: vec![mvt::tile::Layer {
                version: 2,
                name: "points".to_string(),
                features: vec![mvt::tile::Feature {
                    id: None,
                    tags: vec![],
                    r#type: Some(mvt::tile::GeomType::Point.into()),
                    geometry: vec![],
                }],
                keys: vec![],
                values: vec![],
                extent: Some(4096),
            }],
        };

        let mut raw = Vec::new();
        tile.encode(&mut raw).unwrap();

        // Gzip-compress the tile
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw).unwrap();
        let compressed = encoder.finish().unwrap();

        conn.execute(
            "INSERT INTO tiles (zoom_level, tile_column, tile_row, tile_data) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![5, 0, 0, compressed],
        ).unwrap();

        let stats = collect_statistics(&conn, "src", &[5], 1.0).unwrap();
        let layer = stats.layer_stats("src", "points").unwrap();
        assert_eq!(layer.total_features, 1);
        assert_eq!(layer.geometry_types.point, 1);
        assert!(!layer.has_feature_ids);
    }
}
