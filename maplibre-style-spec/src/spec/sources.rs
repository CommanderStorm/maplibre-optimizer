#![allow(clippy::large_enum_variant)]
#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use crate::{boolean_prop, color_prop, numeric_prop, string_prop};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<GeojsonSourceAttribution>,
    /// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
    ///
    /// Range: 0..=512
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffer: Option<GeojsonSourceBuffer>,
    /// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
    ///
    ///  * `cluster` Is `true` if the point is a cluster
    ///
    ///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
    ///
    ///  * `point_count` Number of original points grouped into this cluster
    ///
    ///  * `point_count_abbreviated` An abbreviated point count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<GeojsonSourceCluster>,
    /// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
    #[serde(rename = "clusterMaxZoom")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_max_zoom: Option<GeojsonSourceClusterMaxZoom>,
    /// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
    #[serde(rename = "clusterMinPoints")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_min_points: Option<GeojsonSourceClusterMinPoints>,
    /// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
    ///
    /// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
    ///
    /// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
    ///
    /// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
    #[serde(rename = "clusterProperties")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_properties: Option<GeojsonSourceClusterProperties>,
    /// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
    ///
    /// Range: 0..
    #[serde(rename = "clusterRadius")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_radius: Option<GeojsonSourceClusterRadius>,
    /// A URL to a GeoJSON file, or inline GeoJSON.
    pub data: GeojsonSourceData,
    /// An expression for filtering features prior to processing them for rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<GeojsonSourceFilter>,
    /// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
    #[serde(rename = "generateId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generate_id: Option<GeojsonSourceGenerateId>,
    /// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
    #[serde(rename = "lineMetrics")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_metrics: Option<GeojsonSourceLineMetrics>,
    /// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<GeojsonSourceMaxzoom>,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
    #[serde(rename = "promoteId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<GeojsonSourcePromoteId>,
    /// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<GeojsonSourceTolerance>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceAttribution(std::string::String);

/// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
///
/// Range: 0..=512
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceBuffer(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceBuffer {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(128)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
///
///  * `cluster` Is `true` if the point is a cluster
///
///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
///
///  * `point_count` Number of original points grouped into this cluster
///
///  * `point_count_abbreviated` An abbreviated point count
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceCluster(bool);

/// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterMaxZoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterMinPoints(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

/// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
///
/// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
///
/// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
///
/// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterProperties(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
///
/// Range: 0..
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceClusterRadius(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceClusterRadius {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(50)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A URL to a GeoJSON file, or inline GeoJSON.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceData(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    serde_json::Value,
);

/// An expression for filtering features prior to processing them for rendering.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceFilter(Filter);

/// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceGenerateId(bool);

/// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceLineMetrics(bool);

/// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(18)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourcePromoteId(std::string::String);

/// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct GeojsonSourceTolerance(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for GeojsonSourceTolerance {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.375)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSource {
    /// Corners of image specified in longitude, latitude pairs.
    pub coordinates: ImageSourceCoordinates,
    /// URL that points to an image.
    pub url: ImageSourceUrl,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceCoordinatesValue(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

/// Corners of image specified in longitude, latitude pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceCoordinates(Box<[ImageSourceCoordinatesValue; 4]>);

/// URL that points to an image.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct ImageSourceUrl(std::string::String);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<RasterSourceAttribution>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<RasterSourceBounds>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<RasterSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<RasterSourceMinzoom>,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<RasterSourceScheme>,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_size: Option<RasterSourceTileSize>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiles: Option<RasterSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<RasterSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volatile: Option<RasterSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceAttribution(std::string::String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for RasterSourceBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterSourceScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    #[default]
    Xyz,
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceTileSize(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterSourceTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterSourceVolatile(bool);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<RasterDemSourceAttribution>,
    /// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
    #[serde(rename = "baseShift")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_shift: Option<RasterDemSourceBaseShift>,
    /// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
    #[serde(rename = "blueFactor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blue_factor: Option<RasterDemSourceBlueFactor>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<RasterDemSourceBounds>,
    /// The encoding used by this source. Mapbox Terrain RGB is used by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<RasterDemSourceEncoding>,
    /// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
    #[serde(rename = "greenFactor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub green_factor: Option<RasterDemSourceGreenFactor>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<RasterDemSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<RasterDemSourceMinzoom>,
    /// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
    #[serde(rename = "redFactor")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub red_factor: Option<RasterDemSourceRedFactor>,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tile_size: Option<RasterDemSourceTileSize>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiles: Option<RasterDemSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<RasterDemSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volatile: Option<RasterDemSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceAttribution(std::string::String);

/// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBaseShift(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceBaseShift {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(0.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBlueFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceBlueFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for RasterDemSourceBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// The encoding used by this source. Mapbox Terrain RGB is used by default.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum RasterDemSourceEncoding {
    /// Decodes tiles using the redFactor, blueFactor, greenFactor, baseShift parameters.
    #[serde(rename = "custom")]
    Custom,
    /// Mapbox Terrain RGB tiles. See https://www.mapbox.com/help/access-elevation-data/#mapbox-terrain-rgb for more info.
    #[serde(rename = "mapbox")]
    #[default]
    Mapbox,
    /// Terrarium format PNG tiles. See https://aws.amazon.com/es/public-datasets/terrain/ for more info.
    #[serde(rename = "terrarium")]
    Terrarium,
}

/// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceGreenFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceGreenFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceRedFactor(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceRedFactor {
    fn default() -> Self {
        Self(
            serde_json::Number::from_f64(1.0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceTileSize(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for RasterDemSourceTileSize {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(512)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct RasterDemSourceVolatile(bool);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSource {
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<VectorSourceAttribution>,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<VectorSourceBounds>,
    /// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<VectorSourceEncoding>,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<VectorSourceMaxzoom>,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<VectorSourceMinzoom>,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
    #[serde(rename = "promoteId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<VectorSourcePromoteId>,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme: Option<VectorSourceScheme>,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiles: Option<VectorSourceTiles>,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<VectorSourceUrl>,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volatile: Option<VectorSourceVolatile>,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceAttribution(std::string::String);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceBounds(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]
     Box<[serde_json::Number; 4]>,
);

impl Default for VectorSourceBounds {
    fn default() -> Self {
        Self(Box::new([
            serde_json::Number::from_i128(-180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(-85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_i128(180)
                .expect("the number is serialised from a number and is thus always valid"),
            serde_json::Number::from_f64(85.051129)
                .expect("the number is serialised from a number and is thus always valid"),
        ]))
    }
}

/// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum VectorSourceEncoding {
    /// MapLibre Vector Tiles. See https://github.com/maplibre/maplibre-tile-spec for more info.
    #[serde(rename = "mlt")]
    Mlt,
    /// Mapbox Vector Tiles. See http://github.com/mapbox/vector-tile-spec for more info.
    #[serde(rename = "mvt")]
    #[default]
    Mvt,
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceMaxzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for VectorSourceMaxzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(22)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceMinzoom(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
    serde_json::Number,
);

impl Default for VectorSourceMinzoom {
    fn default() -> Self {
        Self(
            serde_json::Number::from_i128(0)
                .expect("the number is serialised from a number and is thus always valid"),
        )
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourcePromoteId(std::string::String);

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum VectorSourceScheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    #[default]
    Xyz,
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceTiles(Vec<std::string::String>);

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceUrl(std::string::String);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VectorSourceVolatile(bool);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSource {
    /// Corners of video specified in longitude, latitude pairs.
    pub coordinates: VideoSourceCoordinates,
    /// URLs to video content in order of preferred format.
    pub urls: VideoSourceUrls,
}

/// A single longitude, latitude pair.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceCoordinatesValue(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]
     Box<[serde_json::Number; 2]>,
);

/// Corners of video specified in longitude, latitude pairs.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceCoordinates(Box<[VideoSourceCoordinatesValue; 4]>);

/// URLs to video content in order of preferred format.
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct VideoSourceUrls(Vec<std::string::String>);

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename = "geojson")]
    Geojson(GeojsonSource),
    #[serde(rename = "image")]
    Image(ImageSource),
    #[serde(rename = "raster")]
    Raster(RasterSource),
    #[serde(rename = "raster-dem")]
    RasterDem(RasterDemSource),
    #[serde(rename = "vector")]
    Vector(VectorSource),
    #[serde(rename = "video")]
    Video(VideoSource),
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
}
