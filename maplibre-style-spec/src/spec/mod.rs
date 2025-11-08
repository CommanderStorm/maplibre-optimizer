/// This is a Maplibre Style Specification
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct MaplibreStyleSpecification {
    /// Default bearing, in degrees. The bearing is the compass direction that is "up"; for example, a bearing of 90° orients the map so that east is up. This value will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub bearing: Bearing,
    /// Default map center in longitude and latitude.  The style center will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub center: Center,
    /// Default map center altitude in meters above sea level. The style center altitude defines the altitude where the camera is looking at and will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    #[serde(rename = "centerAltitude")]
    pub center_altitude: Centeraltitude,
    /// The `font-faces` property can be used to specify what font files to use for rendering text. Font faces contain information needed to render complex texts such as [Devanagari](https://en.wikipedia.org/wiki/Devanagari), [Khmer](https://en.wikipedia.org/wiki/Khmer_script) among many others.<h2>Unicode range</h2>The optional `unicode-range` property can be used to only use a particular font file for characters within the specified unicode range(s). Its value should be an array of strings, each indicating a start and end of a unicode range, similar to the [CSS descriptor with the same name](https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face/unicode-range). This allows specifying multiple non-consecutive unicode ranges. When not specified, the default value is `U+0-10FFFF`, meaning the font file will be used for all unicode characters.
    ///
    /// Refer to the [Unicode Character Code Charts](https://www.unicode.org/charts/) to see ranges for scripts supported by Unicode. To see what unicode code-points are available in a font, use a tool like [FontDrop](https://fontdrop.info/).
    ///
    /// <h2>Font Resolution</h2>For every name in a symbol layer’s [`text-font`](./layers.md/#text-font) array, characters are matched if they are covered one of the by the font files in the corresponding entry of the `font-faces` map. Any still-unmatched characters then fall back to the [`glyphs`](./glyphs.md) URL if provided.
    ///
    /// <h2>Supported Fonts</h2>What type of fonts are supported is implementation-defined. Unsupported fonts are ignored.
    #[serde(rename = "font-faces")]
    pub font_faces: FontFaces,
    /// A URL template for loading signed-distance-field glyph sets in PBF format.
    ///
    /// If this property is set, any text in the `text-field` layout property is displayed in the font stack named by the `text-font` layout property based on glyphs located at the URL specified by this property. Otherwise, font faces will be determined by the `text-font` property based on the local environment.
    ///
    /// The URL must include:
    ///
    ///  - `{fontstack}` - When requesting glyphs, this token is replaced with a comma separated list of fonts from a font stack specified in the `text-font` property of a symbol layer.
    ///
    ///  - `{range}` - When requesting glyphs, this token is replaced with a range of 256 Unicode code points. For example, to load glyphs for the Unicode Basic Latin and Basic Latin-1 Supplement blocks, the range would be 0-255. The actual ranges that are loaded are determined at runtime based on what text needs to be displayed.
    ///
    /// The URL must be absolute, containing the [scheme, authority and path components](https://en.wikipedia.org/wiki/URL#Syntax).
    pub glyphs: Glyphs,
    /// A style's `layers` property lists all the layers available in that style. The type of layer is specified by the `type` property, and must be one of `background`, `fill`, `line`, `symbol`, `raster`, `circle`, `fill-extrusion`, `heatmap`, `hillshade`, `color-relief`.
    ///
    /// Except for layers of the `background` type, each layer needs to refer to a source. Layers take the data that they get from a source, optionally filter features, and then define how those features are styled.
    pub layers: Layers,
    /// The global light source.
    pub light: Light,
    /// Arbitrary properties useful to track with the stylesheet, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    pub metadata: Metadata,
    /// A human-readable name for the style.
    pub name: Name,
    /// Default pitch, in degrees. Zero is perpendicular to the surface, for a look straight down at the map, while a greater value like 60 looks ahead towards the horizon. The style pitch will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub pitch: Pitch,
    /// The projection configuration
    pub projection: Projection,
    /// Default roll, in degrees. The roll angle is measured counterclockwise about the camera boresight. The style roll will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub roll: Roll,
    /// The map's sky configuration. **Note:** this definition is still experimental and is under development in maplibre-gl-js.
    pub sky: Sky,
    /// Sources state which data the map should display. Specify the type of source with the `type` property. Adding a source isn't enough to make data appear on the map because sources don't contain styling details like color or width. Layers refer to a source and give it a visual representation. This makes it possible to style the same source in different ways, like differentiating between types of roads in a highways layer.
    ///
    /// Tiled sources (vector and raster) must specify their details according to the [TileJSON specification](https://github.com/mapbox/tilejson-spec).
    pub sources: Sources,
    /// An array of `{id: 'my-sprite', url: 'https://example.com/sprite'}` objects. Each object should represent a unique URL to load a sprite from and and a unique ID to use as a prefix when referencing images from that sprite (i.e. 'my-sprite:image'). All the URLs are internally extended to load both .json and .png files. If the `id` field is equal to 'default', the prefix is omitted (just 'image' instead of 'default:image'). All the IDs and URLs must be unique. For backwards compatibility, instead of an array, one can also provide a single string that represent a URL to load the sprite from. The images in this case won't be prefixed.
    pub sprite: Sprite,
    /// An object used to define default values when using the [`global-state`](https://maplibre.org/maplibre-style-spec/expressions/#global-state) expression.
    pub state: State,
    /// The terrain configuration.
    pub terrain: Terrain,
    /// A global transition definition to use as a default across properties, to be used for timing transitions between one value and the next when no property-specific transition is set. Collision-based symbol fading is controlled independently of the style's `transition` property.
    pub transition: Transition,
    /// Style specification version number. Must be 8.
    pub version: Version,
    /// Default zoom level.  The style zoom will be used only if the map has not been positioned by other means (e.g. map options or user interaction).
    pub zoom: Zoom,
}

/// An expression defines a function that can be used for data-driven style properties or feature filters.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Expression(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExpressionName {
    /// Logical negation. Returns `true` if the input is `false`, and `false` if the input is `true`.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "!")]
    Not,
    /// Returns `true` if the input values are not equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "!=")]
    NotEqual,
    /// Returns the remainder after integer division of the first input by the second.
    #[serde(rename = "%")]
    Percentage,
    /// Returns the product of the inputs.
    #[serde(rename = "*")]
    Star,
    /// Returns the sum of the inputs.
    #[serde(rename = "+")]
    Plus,
    /// For two inputs, returns the result of subtracting the second input from the first. For a single input, returns the result of subtracting it from 0.
    #[serde(rename = "-")]
    Minus,
    /// Returns the result of floating point division of the first input by the second.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "/")]
    Slash,
    /// Returns `true` if the first input is strictly less than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "<")]
    Less,
    /// Returns `true` if the first input is less than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    #[serde(rename = "<=")]
    Lessequal,
    /// Returns `true` if the input values are equal, `false` otherwise. The comparison is strictly typed: values of different runtime types are always considered unequal. Cases where the types are known to be different at parse time are considered invalid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Add multiple geometries from one GeoJSON source](https://maplibre.org/maplibre-gl-js/docs/examples/multiple-geometries/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/timeline-animation/)
    ///
    ///  - [Display buildings in 3D](https://maplibre.org/maplibre-gl-js/docs/examples/display-buildings-in-3d/)
    ///
    ///  - [Filter symbols by toggling a list](https://maplibre.org/maplibre-gl-js/docs/examples/filter-symbols-by-toggling-a-list/)
    #[serde(rename = "==")]
    Equal,
    /// Returns `true` if the first input is strictly greater than the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    #[serde(rename = ">")]
    Greater,
    /// Returns `true` if the first input is greater than or equal to the second, `false` otherwise. The arguments are required to be either both strings or both numbers; if during evaluation they are not, expression evaluation produces an error. Cases where this constraint is known not to hold at parse time are considered in valid and will produce a parse error. Accepts an optional `collator` argument to control locale-dependent string comparisons.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = ">=")]
    Greaterequal,
    /// Returns the result of raising the first input to the power specified by the second.
    #[serde(rename = "^")]
    Power,
    /// Returns the absolute value of the input.
    #[serde(rename = "abs")]
    Absolute,
    /// Gets the value of a cluster property accumulated so far. Can only be used in the `clusterProperties` option of a clustered GeoJSON source.
    #[serde(rename = "accumulated")]
    Accumulated,
    /// Returns the arccosine of the input.
    #[serde(rename = "acos")]
    Arccosine,
    /// Returns `true` if all the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `false`, the result is `false` and no further input expressions are evaluated.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "all")]
    All,
    /// Returns `true` if any of the inputs are `true`, `false` otherwise. The inputs are evaluated in order, and evaluation is short-circuiting: once an input expression evaluates to `true`, the result is `true` and no further input expressions are evaluated.
    #[serde(rename = "any")]
    Any,
    /// Asserts that the input is an array (optionally with a specific item type and length). If, when the input expression is evaluated, it is not of the asserted type or length, then this assertion will cause the whole expression to be aborted.
    #[serde(rename = "array")]
    Array,
    /// Returns the arcsine of the input.
    #[serde(rename = "asin")]
    Asin,
    /// Retrieves an item from an array.
    #[serde(rename = "at")]
    At,
    /// Returns the arctangent of the input.
    #[serde(rename = "atan")]
    Atan,
    /// Asserts that the input value is a boolean. If multiple values are provided, each one is evaluated in order until a boolean is obtained. If none of the inputs are booleans, the expression is an error.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    #[serde(rename = "boolean")]
    Boolean,
    /// Selects the first output whose corresponding test condition evaluates to true, or the fallback value otherwise.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "case")]
    Case,
    /// Returns the smallest integer that is greater than or equal to the input.
    #[serde(rename = "ceil")]
    Ceil,
    /// Evaluates each expression in turn until the first non-null value is obtained, and returns that value.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    #[serde(rename = "coalesce")]
    Coalesce,
    /// Returns a `collator` for use in locale-dependent comparison operations. The `case-sensitive` and `diacritic-sensitive` options default to `false`. The `locale` argument specifies the IETF language tag of the locale to use. If none is provided, the default locale is used. If the requested locale is not available, the `collator` will use a system-defined fallback locale. Use `resolved-locale` to test the results of locale fallback behavior.
    #[serde(rename = "collator")]
    Collator,
    /// Returns a `string` consisting of the concatenation of the inputs. Each input is converted to a string as if by `to-string`.
    ///
    ///  - [Add a generated icon to the map](https://maplibre.org/maplibre-gl-js/docs/examples/add-a-generated-icon-to-the-map/)
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/fallback-image/)
    ///
    ///  - [Variable label placement](https://maplibre.org/maplibre-gl-js/docs/examples/variable-label-placement/)
    #[serde(rename = "concat")]
    Concat,
    /// Returns the cosine of the input.
    #[serde(rename = "cos")]
    Cos,
    /// Returns the shortest distance in meters between the evaluated feature and the input geometry. The input value can be a valid GeoJSON of type `Point`, `MultiPoint`, `LineString`, `MultiLineString`, `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Distance values returned may vary in precision due to loss in precision from encoding geometries, particularly below zoom level 13.
    #[serde(rename = "distance")]
    Distance,
    /// Returns the input string converted to lowercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    #[serde(rename = "downcase")]
    Downcase,
    /// Returns the mathematical constant e.
    #[serde(rename = "e")]
    E,
    /// Gets the elevation of a pixel (in meters above the vertical datum reference of the `raster-dem` tiles) from a `raster-dem` source. Can only be used in the `color-relief-color` property of a `color-relief` layer.
    #[serde(rename = "elevation")]
    Elevation,
    /// Retrieves a property value from the current feature's state. Returns null if the requested property is not present on the feature's state. A feature's state is not part of the GeoJSON or vector tile data, and must be set programmatically on each feature. When `source.promoteId` is not provided, features are identified by their `id` attribute, which must be an integer or a string that can be cast to an integer. When `source.promoteId` is provided, features are identified by their `promoteId` property, which may be a number, string, or any primitive data type. Note that ["feature-state"] can only be used with paint properties that support data-driven styling.
    ///
    ///  - [Create a hover effect](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-hover-effect/)
    #[serde(rename = "feature-state")]
    FeatureState,
    /// Returns the largest integer that is less than or equal to the input.
    #[serde(rename = "floor")]
    Floor,
    /// Returns a `formatted` string for displaying mixed-format text in the `text-field` property. The input may contain a string literal or expression, including an [`'image'`](#image) expression. Strings may be followed by a style override object that supports the following properties:
    ///
    /// - `"text-font"`: Overrides the font stack specified by the root layout property.
    ///
    /// - `"text-color"`: Overrides the color specified by the root paint property.
    ///
    /// - `"font-scale"`: Applies a scaling factor on `text-size` as specified by the root layout property.
    ///
    /// - `"vertical-align"`: Aligns vertically text section or image in relation to the row it belongs to. Possible values are:
    /// 	- `"bottom"` *default*: align the bottom of this section with the bottom of other sections.
    /// <img alt="Visual representation of bottom alignment" src="https://github.com/user-attachments/assets/0474a2fd-a4b2-417c-9187-7a13a28695bc"/>
    /// 	- `"center"`: align the center of this section with the center of other sections.
    /// <img alt="Visual representation of center alignment" src="https://github.com/user-attachments/assets/92237455-be6d-4c5d-b8f6-8127effc1950"/>
    /// 	- `"top"`: align the top of this section with the top of other sections.
    /// <img alt="Visual representation of top alignment" src="https://github.com/user-attachments/assets/45dccb28-d977-4abb-a006-4ea9792b7c53"/>
    /// 	- Refer to [the design proposal](https://github.com/maplibre/maplibre-style-spec/issues/832) for more details.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    #[serde(rename = "format")]
    Format,
    /// Returns the feature's simple geometry type: `Point`, `LineString`, or `Polygon`. `MultiPoint`, `MultiLineString`, and `MultiPolygon` are returned as `Point`, `LineString`, and `Polygon`, respectively.
    #[serde(rename = "geometry-type")]
    GeometryType,
    /// Retrieves a property value from the current feature's properties, or from another object if a second argument is provided. Returns null if the requested property is missing.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    ///
    ///  - [Extrude polygons for 3D indoor mapping](https://maplibre.org/maplibre-gl-js/docs/examples/extrude-polygons-for-3d-indoor-mapping/)
    #[serde(rename = "get")]
    Get,
    /// Retrieves a property value from global state that can be set with platform-specific APIs. Defaults can be provided using the [`state`](https://maplibre.org/maplibre-style-spec/root/#state) root property. Returns `null` if no value nor default value is set for the retrieved property.
    #[serde(rename = "global-state")]
    GlobalState,
    /// Tests for the presence of a property value in the current feature's properties, or from another object if a second argument is provided.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "has")]
    Has,
    /// Gets the kernel density estimation of a pixel in a heatmap layer, which is a relative measure of how many data points are crowded around a particular pixel. Can only be used in the `heatmap-color` property.
    #[serde(rename = "heatmap-density")]
    HeatmapDensity,
    /// Gets the feature's id, if it has one.
    #[serde(rename = "id")]
    Id,
    /// Returns an `image` type for use in `icon-image`, `*-pattern` entries and as a section in the `format` expression. If set, the `image` argument will check that the requested image exists in the style and will return either the resolved image name or `null`, depending on whether or not the image is currently in the style. This validation process is synchronous and requires the image to have been added to the style before requesting it in the `image` argument.
    ///
    ///  - [Use a fallback image](https://maplibre.org/maplibre-gl-js/docs/examples/use-a-fallback-image/)
    #[serde(rename = "image")]
    Image,
    /// Determines whether an item exists in an array or a substring exists in a string.
    ///
    ///  - [Measure distances](https://maplibre.org/maplibre-gl-js/docs/examples/measure-distances/)
    #[serde(rename = "in")]
    In,
    /// Returns the first position at which an item can be found in an array or a substring can be found in a string, or `-1` if the input cannot be found. Accepts an optional index from where to begin the search. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "index-of")]
    IndexOf,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order. The output type must be `number`, `array<number>`, `color`, `array<color>`, or `projection`.
    ///
    /// Interpolation types:
    ///
    /// - `["linear"]`, or an expression returning one of those types: Interpolates linearly between the pair of stops just less than and just greater than the input.
    ///
    /// - `["exponential", base]`: Interpolates exponentially between the stops just less than and just greater than the input. `base` controls the rate at which the output increases: higher values make the output increase more towards the high end of the range. With values close to 1 the output increases linearly.
    ///
    /// - `["cubic-bezier", x1, y1, x2, y2]`: Interpolates using the cubic bezier curve defined by the given control points.
    ///
    ///  - [Animate map camera around a point](https://maplibre.org/maplibre-gl-js/docs/examples/animate-camera-around-point/)
    ///
    ///  - [Change building color based on zoom level](https://maplibre.org/maplibre-gl-js/docs/examples/change-building-color-based-on-zoom-level/)
    ///
    ///  - [Create a heatmap layer](https://maplibre.org/maplibre-gl-js/docs/examples/heatmap-layer/)
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "interpolate")]
    Interpolate,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the Hue-Chroma-Luminance color space.
    #[serde(rename = "interpolate-hcl")]
    InterpolateHcl,
    /// Produces continuous, smooth results by interpolating between pairs of input and output values ("stops"). Works like `interpolate`, but the output type must be `color` or `array<color>`, and the interpolation is performed in the CIELAB color space.
    #[serde(rename = "interpolate-lab")]
    InterpolateLab,
    /// Returns `true` if the input string is expected to render legibly. Returns `false` if the input string contains sections that cannot be rendered without potential loss of meaning (e.g. Indic scripts that require complex text shaping, or right-to-left scripts if the `mapbox-gl-rtl-text` plugin is not in use in MapLibre GL JS).
    #[serde(rename = "is-supported-script")]
    IsSupportedScript,
    /// Gets the length of an array or string. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "length")]
    Length,
    /// Binds expressions to named variables, which can then be referenced in the result expression using `["var", "variable_name"]`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "let")]
    Let,
    /// Gets the progress along a gradient line. Can only be used in the `line-gradient` property.
    #[serde(rename = "line-progress")]
    LineProgress,
    /// Provides a literal array or object value.
    ///
    ///  - [Display and style rich text labels](https://maplibre.org/maplibre-gl-js/docs/examples/display-and-style-rich-text-labels/)
    #[serde(rename = "literal")]
    Literal,
    /// Returns the natural logarithm of the input.
    #[serde(rename = "ln")]
    Ln,
    /// Returns the mathematical constant ln(2).
    #[serde(rename = "ln2")]
    Ln2,
    /// Returns the base-ten logarithm of the input.
    #[serde(rename = "log10")]
    Log10,
    /// Returns the base-two logarithm of the input.
    #[serde(rename = "log2")]
    Log2,
    /// Selects the output whose label value matches the input value, or the fallback value if no match is found. The input can be any expression (e.g. `["get", "building_type"]`). Each label must be either:
    ///
    ///  - a single literal value; or
    ///
    ///  - an array of literal values, whose values must be all strings or all numbers (e.g. `[100, 101]` or `["c", "b"]`). The input matches if any of the values in the array matches, similar to the `"in"` operator.
    ///
    /// Each label must be unique. If the input type does not match the type of the labels, the result will be the fallback value.
    #[serde(rename = "match")]
    Match,
    /// Returns the maximum value of the inputs.
    #[serde(rename = "max")]
    Max,
    /// Returns the minimum value of the inputs.
    #[serde(rename = "min")]
    Min,
    /// Asserts that the input value is a number. If multiple values are provided, each one is evaluated in order until a number is obtained. If none of the inputs are numbers, the expression is an error.
    #[serde(rename = "number")]
    Number,
    /// Converts the input number into a string representation using the providing formatting rules. If set, the `locale` argument specifies the locale to use, as a BCP 47 language tag. If set, the `currency` argument specifies an ISO 4217 code to use for currency-style formatting. If set, the `min-fraction-digits` and `max-fraction-digits` arguments specify the minimum and maximum number of fractional digits to include.
    ///
    ///  - [Display HTML clusters with custom properties](https://maplibre.org/maplibre-gl-js/docs/examples/display-html-clusters-with-custom-properties/)
    #[serde(rename = "number-format")]
    NumberFormat,
    /// Asserts that the input value is an object. If multiple values are provided, each one is evaluated in order until an object is obtained. If none of the inputs are objects, the expression is an error.
    #[serde(rename = "object")]
    Object,
    /// Returns the mathematical constant pi.
    #[serde(rename = "pi")]
    Pi,
    /// Gets the feature properties object.  Note that in some cases, it may be more efficient to use ["get", "property_name"] directly.
    #[serde(rename = "properties")]
    Properties,
    /// Returns the IETF language tag of the locale being used by the provided `collator`. This can be used to determine the default system locale, or to determine if a requested locale was successfully loaded.
    #[serde(rename = "resolved-locale")]
    ResolvedLocale,
    /// Creates a color value from red, green, and blue components, which must range between 0 and 255, and an alpha component of 1. If any component is out of range, the expression is an error.
    #[serde(rename = "rgb")]
    Rgb,
    /// Creates a color value from red, green, blue components, which must range between 0 and 255, and an alpha component which must range between 0 and 1. If any component is out of range, the expression is an error.
    #[serde(rename = "rgba")]
    Rgba,
    /// Rounds the input to the nearest integer. Halfway values are rounded away from zero. For example, `["round", -1.5]` evaluates to -2.
    #[serde(rename = "round")]
    Round,
    /// Returns the sine of the input.
    #[serde(rename = "sin")]
    Sin,
    /// Returns a subarray from an array or a substring from a string from a specified start index, or between a start index and an end index if set. The return value is inclusive of the start index but not of the end index. In a string, a UTF-16 surrogate pair counts as a single position.
    #[serde(rename = "slice")]
    Slice,
    /// Returns the square root of the input.
    #[serde(rename = "sqrt")]
    Sqrt,
    /// Produces discrete, stepped results by evaluating a piecewise-constant function defined by pairs of input and output values ("stops"). The `input` may be any numeric expression (e.g., `["get", "population"]`). Stop inputs must be numeric literals in strictly ascending order.
    ///
    /// Returns the output value of the stop just less than the input, or the first output if the input is less than the first stop.
    ///
    ///  - [Create and style clusters](https://maplibre.org/maplibre-gl-js/docs/examples/create-and-style-clusters/)
    #[serde(rename = "step")]
    Step,
    /// Asserts that the input value is a string. If multiple values are provided, each one is evaluated in order until a string is obtained. If none of the inputs are strings, the expression is an error.
    #[serde(rename = "string")]
    String,
    /// Returns the tangent of the input.
    #[serde(rename = "tan")]
    Tan,
    /// Converts the input value to a boolean. The result is `false` when the input is an empty string, 0, `false`, `null`, or `NaN`; otherwise it is `true`.
    #[serde(rename = "to-boolean")]
    ToBoolean,
    /// Converts the input value to a color. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "to-color")]
    ToColor,
    /// Converts the input value to a number, if possible. If the input is `null` or `false`, the result is 0. If the input is `true`, the result is 1. If the input is a string, it is converted to a number as specified by the ["ToNumber Applied to the String Type" algorithm](https://tc39.github.io/ecma262/#sec-tonumber-applied-to-the-string-type) of the ECMAScript Language Specification. If multiple values are provided, each one is evaluated in order until the first successful conversion is obtained. If none of the inputs can be converted, the expression is an error.
    #[serde(rename = "to-number")]
    ToNumber,
    /// Returns a four-element array containing the input color's red, green, blue, and alpha components, in that order.
    #[serde(rename = "to-rgba")]
    ToRgba,
    /// Converts the input value to a string. If the input is `null`, the result is `""`. If the input is a boolean, the result is `"true"` or `"false"`. If the input is a number, it is converted to a string as specified by the ["NumberToString" algorithm](https://tc39.github.io/ecma262/#sec-tostring-applied-to-the-number-type) of the ECMAScript Language Specification. If the input is a color, it is converted to a string of the form `"rgba(r,g,b,a)"`, where `r`, `g`, and `b` are numerals ranging from 0 to 255, and `a` ranges from 0 to 1. Otherwise, the input is converted to a string in the format specified by the [`JSON.stringify`](https://tc39.github.io/ecma262/#sec-json.stringify) function of the ECMAScript Language Specification.
    ///
    ///  - [Create a time slider](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-time-slider/)
    #[serde(rename = "to-string")]
    ToString,
    /// Returns a string describing the type of the given value.
    #[serde(rename = "typeof")]
    Typeof,
    /// Returns the input string converted to uppercase. Follows the Unicode Default Case Conversion algorithm and the locale-insensitive case mappings in the Unicode Character Database.
    ///
    ///  - [Change the case of labels](https://maplibre.org/maplibre-gl-js/docs/examples/change-case-of-labels/)
    #[serde(rename = "upcase")]
    Upcase,
    /// References variable bound using `let`.
    ///
    ///  - [Visualize population density](https://maplibre.org/maplibre-gl-js/docs/examples/visualize-population-density/)
    #[serde(rename = "var")]
    Var,
    /// Returns `true` if the evaluated feature is fully contained inside a boundary of the input geometry, `false` otherwise. The input value can be a valid GeoJSON of type `Polygon`, `MultiPolygon`, `Feature`, or `FeatureCollection`. Supported features for evaluation:
    ///
    /// - `Point`: Returns `false` if a point is on the boundary or falls outside the boundary.
    ///
    /// - `LineString`: Returns `false` if any part of a line falls outside the boundary, the line intersects the boundary, or a line's endpoint is on the boundary.
    #[serde(rename = "within")]
    Within,
    /// Gets the current zoom level.  Note that in style layout and paint properties, ["zoom"] may only appear as the input to a top-level "step" or "interpolate" expression.
    #[serde(rename = "zoom")]
    Zoom,
}

/// A filter selects specific features from a layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Filter(serde_json::Value);

/// The filter operator.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FilterOperator {
    /// `["!=", key, value]` inequality: `feature[key] ≠ value`
    #[serde(rename = "!=")]
    NotEqual,
    /// `["!has", key]` `feature[key]` does not exist
    #[serde(rename = "!has")]
    NotHas,
    /// `["!in", key, v0, ..., vn]` set exclusion: `feature[key] ∉ {v0, ..., vn}`
    #[serde(rename = "!in")]
    NotIn,
    /// `["<", key, value]` less than: `feature[key] < value`
    #[serde(rename = "<")]
    Less,
    /// `["<=", key, value]` less than or equal: `feature[key] ≤ value`
    #[serde(rename = "<=")]
    Lessequal,
    /// `["==", key, value]` equality: `feature[key] = value`
    #[serde(rename = "==")]
    Equal,
    /// `[">", key, value]` greater than: `feature[key] > value`
    #[serde(rename = ">")]
    Greater,
    /// `[">=", key, value]` greater than or equal: `feature[key] ≥ value`
    #[serde(rename = ">=")]
    Greaterequal,
    /// `["all", f0, ..., fn]` logical `AND`: `f0 ∧ ... ∧ fn`
    #[serde(rename = "all")]
    All,
    /// `["any", f0, ..., fn]` logical `OR`: `f0 ∨ ... ∨ fn`
    #[serde(rename = "any")]
    Any,
    /// `["has", key]` `feature[key]` exists
    #[serde(rename = "has")]
    Has,
    /// `["in", key, v0, ..., vn]` set inclusion: `feature[key] ∈ {v0, ..., vn}`
    #[serde(rename = "in")]
    In,
    /// `["none", f0, ..., fn]` logical `NOR`: `¬f0 ∧ ... ∧ ¬fn`
    #[serde(rename = "none")]
    None,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Function {
    /// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
    #[serde(rename = "base")]
    pub base: Base,
    /// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
    #[serde(rename = "colorSpace")]
    pub color_space: Colorspace,
    /// A value to serve as a fallback function result when a value isn't otherwise available. It is used in the following circumstances:
    ///
    /// * In categorical functions, when the feature value does not match any of the stop domain values.
    ///
    /// * In property and zoom-and-property functions, when a feature does not contain a value for the specified property.
    ///
    /// * In identity functions, when the feature value is not valid for the style property (for example, if the function is being used for a `circle-color` property but the feature property value is not a string or not a valid color).
    ///
    /// * In interval or exponential property and zoom-and-property functions, when the feature value is not numeric.
    ///
    /// If no default is provided, the style property's default is used in these circumstances.
    #[serde(rename = "default")]
    pub default: DefaultStruct,
    /// An expression.
    #[serde(rename = "expression")]
    pub expression: Expression,
    /// The name of a feature property to use as the function input.
    #[serde(rename = "property")]
    pub property: Property,
    /// An array of stops.
    #[serde(rename = "stops")]
    pub stops: Stops,
    /// The interpolation strategy to use in function evaluation.
    #[serde(rename = "type")]
    pub r#type: Type,
}

/// The exponential base of the interpolation curve. It controls the rate at which the result increases. Higher values make the result increase more towards the high end of the range. With `1` the stops are interpolated linearly.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Base(serde_json::Number);

impl Default for Base {
    fn default() -> Self {
        Self(1.into())
    }
}

/// The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Colorspace {
    /// Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually.
    #[serde(rename = "hcl")]
    Hcl,
    /// Use the LAB color space to interpolate color values.
    #[serde(rename = "lab")]
    Lab,
    /// Use the RGB color space to interpolate color values
    #[serde(rename = "rgb")]
    Rgb,
}

impl Default for Colorspace {
    fn default() -> Self {
        Self::Rgb
    }
}

/// A value to serve as a fallback function result when a value isn't otherwise available. It is used in the following circumstances:
///
/// * In categorical functions, when the feature value does not match any of the stop domain values.
///
/// * In property and zoom-and-property functions, when a feature does not contain a value for the specified property.
///
/// * In identity functions, when the feature value is not valid for the style property (for example, if the function is being used for a `circle-color` property but the feature property value is not a string or not a valid color).
///
/// * In interval or exponential property and zoom-and-property functions, when the feature value is not numeric.
///
/// If no default is provided, the style property's default is used in these circumstances.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct DefaultStruct(serde_json::Value);

/// An expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Expression(serde_json::Value);

/// The name of a feature property to use as the function input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Property(serde_json::Value);

impl Default for Property {
    fn default() -> Self {
        "$zoom".to_string()
    }
}

/// An array of stops.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Stops(serde_json::Value);

/// The interpolation strategy to use in function evaluation.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// Return the output value of the stop equal to the function input.
    #[serde(rename = "categorical")]
    Categorical,
    /// Generate an output by interpolating between stops just less than and just greater than the function input.
    #[serde(rename = "exponential")]
    Exponential,
    /// Return the input value as the output value.
    #[serde(rename = "identity")]
    Identity,
    /// Return the output value of the stop just less than the function input.
    #[serde(rename = "interval")]
    Interval,
}

impl Default for Type {
    fn default() -> Self {
        Self::Exponential
    }
}

/// Zoom level and value pair.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FunctionStop(serde_json::Value);

/// The geometry type for the filter to select.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum GeometryType {
    /// Filter to line geometries.
    #[serde(rename = "LineString")]
    Linestring,
    /// Filter to point geometries.
    Point,
    /// Filter to polygon geometries.
    Polygon,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Layer {
    /// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
    #[serde(rename = "filter")]
    pub filter: Filter,
    /// Unique layer name.
    #[serde(rename = "id")]
    pub id: Id,
    /// Layout properties for the layer.
    #[serde(rename = "layout")]
    pub layout: Layout,
    /// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
    #[serde(rename = "maxzoom")]
    pub maxzoom: Maxzoom,
    /// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
    #[serde(rename = "metadata")]
    pub metadata: Metadata,
    /// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
    #[serde(rename = "minzoom")]
    pub minzoom: Minzoom,
    /// Default paint properties for this layer.
    #[serde(rename = "paint")]
    pub paint: Paint,
    /// Name of a source description to be used for this layer. Required for all layer types except `background`.
    #[serde(rename = "source")]
    pub source: Source,
    /// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
    #[serde(rename = "source-layer")]
    pub source_layer: SourceLayer,
    /// Rendering type of this layer.
    #[serde(rename = "type")]
    pub r#type: Type,
}

/// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Filter(serde_json::Value);

/// Unique layer name.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Id(serde_json::Value);

/// Layout properties for the layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Layout(serde_json::Value);

/// The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Maxzoom(serde_json::Number);

/// Arbitrary properties useful to track with the layer, but do not influence rendering. Properties should be prefixed to avoid collisions, like 'maplibre:'.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Metadata(serde_json::Value);

/// The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
///
/// Range: 0..=24
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Minzoom(serde_json::Number);

/// Default paint properties for this layer.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Paint(serde_json::Value);

/// Name of a source description to be used for this layer. Required for all layer types except `background`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Source(serde_json::Value);

/// Layer to use from a vector tile source. Required for vector tile sources; prohibited for all other source types, including GeoJSON sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct SourceLayer(serde_json::Value);

/// Rendering type of this layer.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// The background color or pattern of the map.
    #[serde(rename = "background")]
    Background,
    /// A filled circle.
    #[serde(rename = "circle")]
    Circle,
    /// Client-side elevation coloring based on DEM data. The implementation supports Mapbox Terrain RGB, Mapzen Terrarium tiles and custom encodings.
    #[serde(rename = "color-relief")]
    ColorRelief,
    /// A filled polygon with an optional stroked border.
    #[serde(rename = "fill")]
    Fill,
    /// An extruded (3D) polygon.
    #[serde(rename = "fill-extrusion")]
    FillExtrusion,
    /// A heatmap.
    #[serde(rename = "heatmap")]
    Heatmap,
    /// Client-side hillshading visualization based on DEM data. The implementation supports Mapbox Terrain RGB, Mapzen Terrarium tiles and custom encodings.
    #[serde(rename = "hillshade")]
    Hillshade,
    /// A stroked line.
    #[serde(rename = "line")]
    Line,
    /// Raster map textures such as satellite imagery.
    #[serde(rename = "raster")]
    Raster,
    /// An icon or a text label.
    #[serde(rename = "symbol")]
    Symbol,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Layout {
    LayoutFill(LayoutFill),
    LayoutLine(LayoutLine),
    LayoutCircle(LayoutCircle),
    LayoutHeatmap(LayoutHeatmap),
    LayoutFillExtrusion(LayoutFillExtrusion),
    LayoutSymbol(LayoutSymbol),
    LayoutRaster(LayoutRaster),
    LayoutHillshade(LayoutHillshade),
    LayoutColorRelief(LayoutColorRelief),
    LayoutBackground(LayoutBackground),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutBackground {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutCircle {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "circle-sort-key")]
    pub circle_sort_key: CircleSortKey,
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutColorRelief {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutFill {
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "fill-sort-key")]
    pub fill_sort_key: FillSortKey,
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FillSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutFillExtrusion {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutHeatmap {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutHillshade {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutLine {
    /// The display of line endings.
    #[serde(rename = "line-cap")]
    pub line_cap: LineCap,
    /// The display of lines when joining.
    #[serde(rename = "line-join")]
    pub line_join: LineJoin,
    /// Used to automatically convert miter joins to bevel joins for sharp angles.
    #[serde(rename = "line-miter-limit")]
    pub line_miter_limit: LineMiterLimit,
    /// Used to automatically convert round joins to miter joins for shallow angles.
    #[serde(rename = "line-round-limit")]
    pub line_round_limit: LineRoundLimit,
    /// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
    #[serde(rename = "line-sort-key")]
    pub line_sort_key: LineSortKey,
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// The display of line endings.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineCap {
    /// A cap with a squared-off end which is drawn to the exact endpoint of the line.
    #[serde(rename = "butt")]
    Butt,
    /// A cap with a rounded end which is drawn beyond the endpoint of the line at a radius of one-half of the line's width and centered on the endpoint of the line.
    #[serde(rename = "round")]
    Round,
    /// A cap with a squared-off end which is drawn beyond the endpoint of the line at a distance of one-half of the line's width.
    #[serde(rename = "square")]
    Square,
}

impl Default for LineCap {
    fn default() -> Self {
        Self::Butt
    }
}

/// The display of lines when joining.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineJoin {
    /// A join with a squared-off end which is drawn beyond the endpoint of the line at a distance of one-half of the line's width.
    #[serde(rename = "bevel")]
    Bevel,
    /// A join with a sharp, angled corner which is drawn with the outer sides beyond the endpoint of the path until they meet.
    #[serde(rename = "miter")]
    Miter,
    /// A join with a rounded end which is drawn beyond the endpoint of the line at a radius of one-half of the line's width and centered on the endpoint of the line.
    #[serde(rename = "round")]
    Round,
}

impl Default for LineJoin {
    fn default() -> Self {
        Self::Miter
    }
}

/// Used to automatically convert miter joins to bevel joins for sharp angles.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineMiterLimit(serde_json::Number);

impl Default for LineMiterLimit {
    fn default() -> Self {
        Self(2.into())
    }
}

/// Used to automatically convert round joins to miter joins for shallow angles.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineRoundLimit(serde_json::Number);

impl Default for LineRoundLimit {
    fn default() -> Self {
        Self(1.05.into())
    }
}

/// Sorts features in ascending order based on this value. Features with a higher sort key will appear above features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineSortKey(serde_json::Number);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutRaster {
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LayoutSymbol {
    /// If true, the icon will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "icon-allow-overlap")]
    pub icon_allow_overlap: IconAllowOverlap,
    /// Part of the icon placed closest to the anchor.
    #[serde(rename = "icon-anchor")]
    pub icon_anchor: IconAnchor,
    /// If true, other symbols can be visible even if they collide with the icon.
    #[serde(rename = "icon-ignore-placement")]
    pub icon_ignore_placement: IconIgnorePlacement,
    /// Name of image in sprite to use for drawing an image background.
    #[serde(rename = "icon-image")]
    pub icon_image: IconImage,
    /// If true, the icon may be flipped to prevent it from being rendered upside-down.
    #[serde(rename = "icon-keep-upright")]
    pub icon_keep_upright: IconKeepUpright,
    /// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
    #[serde(rename = "icon-offset")]
    pub icon_offset: IconOffset,
    /// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
    #[serde(rename = "icon-optional")]
    pub icon_optional: IconOptional,
    /// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
    #[serde(rename = "icon-overlap")]
    pub icon_overlap: IconOverlap,
    /// Size of additional area round the icon bounding box used for detecting symbol collisions.
    #[serde(rename = "icon-padding")]
    pub icon_padding: IconPadding,
    /// Orientation of icon when map is pitched.
    #[serde(rename = "icon-pitch-alignment")]
    pub icon_pitch_alignment: IconPitchAlignment,
    /// Rotates the icon clockwise.
    #[serde(rename = "icon-rotate")]
    pub icon_rotate: IconRotate,
    /// In combination with `symbol-placement`, determines the rotation behavior of icons.
    #[serde(rename = "icon-rotation-alignment")]
    pub icon_rotation_alignment: IconRotationAlignment,
    /// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
    #[serde(rename = "icon-size")]
    pub icon_size: IconSize,
    /// Scales the icon to fit around the associated text.
    #[serde(rename = "icon-text-fit")]
    pub icon_text_fit: IconTextFit,
    /// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
    #[serde(rename = "icon-text-fit-padding")]
    pub icon_text_fit_padding: IconTextFitPadding,
    /// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
    #[serde(rename = "symbol-avoid-edges")]
    pub symbol_avoid_edges: SymbolAvoidEdges,
    /// Label placement relative to its geometry.
    #[serde(rename = "symbol-placement")]
    pub symbol_placement: SymbolPlacement,
    /// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
    #[serde(rename = "symbol-sort-key")]
    pub symbol_sort_key: SymbolSortKey,
    /// Distance between two symbol anchors.
    #[serde(rename = "symbol-spacing")]
    pub symbol_spacing: SymbolSpacing,
    /// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
    #[serde(rename = "symbol-z-order")]
    pub symbol_z_order: SymbolZOrder,
    /// If true, the text will be visible even if it collides with other previously drawn symbols.
    #[serde(rename = "text-allow-overlap")]
    pub text_allow_overlap: TextAllowOverlap,
    /// Part of the text placed closest to the anchor.
    #[serde(rename = "text-anchor")]
    pub text_anchor: TextAnchor,
    /// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
    #[serde(rename = "text-field")]
    pub text_field: TextField,
    /// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
    #[serde(rename = "text-font")]
    pub text_font: TextFont,
    /// If true, other symbols can be visible even if they collide with the text.
    #[serde(rename = "text-ignore-placement")]
    pub text_ignore_placement: TextIgnorePlacement,
    /// Text justification options.
    #[serde(rename = "text-justify")]
    pub text_justify: TextJustify,
    /// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
    #[serde(rename = "text-keep-upright")]
    pub text_keep_upright: TextKeepUpright,
    /// Text tracking amount.
    #[serde(rename = "text-letter-spacing")]
    pub text_letter_spacing: TextLetterSpacing,
    /// Text leading value for multi-line text.
    #[serde(rename = "text-line-height")]
    pub text_line_height: TextLineHeight,
    /// Maximum angle change between adjacent characters.
    #[serde(rename = "text-max-angle")]
    pub text_max_angle: TextMaxAngle,
    /// The maximum line width for text wrapping.
    #[serde(rename = "text-max-width")]
    pub text_max_width: TextMaxWidth,
    /// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
    #[serde(rename = "text-offset")]
    pub text_offset: TextOffset,
    /// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
    #[serde(rename = "text-optional")]
    pub text_optional: TextOptional,
    /// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
    #[serde(rename = "text-overlap")]
    pub text_overlap: TextOverlap,
    /// Size of the additional area around the text bounding box used for detecting symbol collisions.
    #[serde(rename = "text-padding")]
    pub text_padding: TextPadding,
    /// Orientation of text when map is pitched.
    #[serde(rename = "text-pitch-alignment")]
    pub text_pitch_alignment: TextPitchAlignment,
    /// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
    #[serde(rename = "text-radial-offset")]
    pub text_radial_offset: TextRadialOffset,
    /// Rotates the text clockwise.
    #[serde(rename = "text-rotate")]
    pub text_rotate: TextRotate,
    /// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
    #[serde(rename = "text-rotation-alignment")]
    pub text_rotation_alignment: TextRotationAlignment,
    /// Font size.
    #[serde(rename = "text-size")]
    pub text_size: TextSize,
    /// Specifies how to capitalize text, similar to the CSS `text-transform` property.
    #[serde(rename = "text-transform")]
    pub text_transform: TextTransform,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
    #[serde(rename = "text-variable-anchor")]
    pub text_variable_anchor: TextVariableAnchor,
    /// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position.
    ///
    ///  The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor.
    ///
    ///  When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`.
    ///
    ///  ```json
    ///
    ///  { "text-variable-anchor-offset": ["top", [0, 4], "left", [3,0], "bottom", [1, 1]] }
    ///
    ///  ```
    ///
    ///  When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems.
    ///
    ///  When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.
    #[serde(rename = "text-variable-anchor-offset")]
    pub text_variable_anchor_offset: TextVariableAnchorOffset,
    /// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
    #[serde(rename = "text-writing-mode")]
    pub text_writing_mode: TextWritingMode,
    /// Whether this layer is displayed.
    #[serde(rename = "visibility")]
    pub visibility: Visibility,
}

/// If true, the icon will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconAllowOverlap(serde_json::Value);

impl Default for IconAllowOverlap {
    fn default() -> Self {
        false
    }
}

/// Part of the icon placed closest to the anchor.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconAnchor {
    /// The bottom of the icon is placed closest to the anchor.
    #[serde(rename = "bottom")]
    Bottom,
    /// The bottom left corner of the icon is placed closest to the anchor.
    #[serde(rename = "bottom-left")]
    BottomLeft,
    /// The bottom right corner of the icon is placed closest to the anchor.
    #[serde(rename = "bottom-right")]
    BottomRight,
    /// The center of the icon is placed closest to the anchor.
    #[serde(rename = "center")]
    Center,
    /// The left side of the icon is placed closest to the anchor.
    #[serde(rename = "left")]
    Left,
    /// The right side of the icon is placed closest to the anchor.
    #[serde(rename = "right")]
    Right,
    /// The top of the icon is placed closest to the anchor.
    #[serde(rename = "top")]
    Top,
    /// The top left corner of the icon is placed closest to the anchor.
    #[serde(rename = "top-left")]
    TopLeft,
    /// The top right corner of the icon is placed closest to the anchor.
    #[serde(rename = "top-right")]
    TopRight,
}

impl Default for IconAnchor {
    fn default() -> Self {
        Self::Center
    }
}

/// If true, other symbols can be visible even if they collide with the icon.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconIgnorePlacement(serde_json::Value);

impl Default for IconIgnorePlacement {
    fn default() -> Self {
        false
    }
}

/// Name of image in sprite to use for drawing an image background.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconImage(serde_json::Value);

/// If true, the icon may be flipped to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconKeepUpright(serde_json::Value);

impl Default for IconKeepUpright {
    fn default() -> Self {
        false
    }
}

/// Offset distance of icon from its anchor. Positive values indicate right and down, while negative values indicate left and up. Each component is multiplied by the value of `icon-size` to obtain the final offset in pixels. When combined with `icon-rotate` the offset will be as if the rotated direction was up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconOffset(serde_json::Value);

impl Default for IconOffset {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// If true, text will display without their corresponding icons when the icon collides with other symbols and the text does not.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconOptional(serde_json::Value);

impl Default for IconOptional {
    fn default() -> Self {
        false
    }
}

/// Allows for control over whether to show an icon when it overlaps other symbols on the map. If `icon-overlap` is not set, `icon-allow-overlap` is used instead.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconOverlap {
    /// The icon will be visible even if it collides with any other previously drawn symbol.
    #[serde(rename = "always")]
    Always,
    /// If the icon collides with another previously drawn symbol, the overlap mode for that symbol is checked. If the previous symbol was placed using `never` overlap mode, the new icon is hidden. If the previous symbol was placed using `always` or `cooperative` overlap mode, the new icon is visible.
    #[serde(rename = "cooperative")]
    Cooperative,
    /// The icon will be hidden if it collides with any other previously drawn symbol.
    #[serde(rename = "never")]
    Never,
}

/// Size of additional area round the icon bounding box used for detecting symbol collisions.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconPadding(serde_json::Value);

impl Default for IconPadding {
    fn default() -> Self {
        vec![2]
    }
}

/// Orientation of icon when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconPitchAlignment {
    /// Automatically matches the value of `icon-rotation-alignment`.
    #[serde(rename = "auto")]
    Auto,
    /// The icon is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The icon is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for IconPitchAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Rotates the icon clockwise.
///
/// Range: ..every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct IconRotate(serde_json::Number);

impl Default for IconRotate {
    fn default() -> Self {
        Self(0.into())
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of icons.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconRotationAlignment {
    /// When `symbol-placement` is set to `point`, this is equivalent to `viewport`. When `symbol-placement` is set to `line` or `line-center`, this is equivalent to `map`.
    #[serde(rename = "auto")]
    Auto,
    /// When `symbol-placement` is set to `point`, aligns icons east-west. When `symbol-placement` is set to `line` or `line-center`, aligns icon x-axes with the line.
    #[serde(rename = "map")]
    Map,
    /// Produces icons whose x-axes are aligned with the x-axis of the viewport, regardless of the value of `symbol-placement`.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for IconRotationAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Scales the original size of the icon by the provided factor. The new pixel size of the image will be the original pixel size multiplied by `icon-size`. 1 is the original size; 3 triples the size of the image.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct IconSize(serde_json::Number);

impl Default for IconSize {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Scales the icon to fit around the associated text.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconTextFit {
    /// The icon is scaled in both x- and y-dimensions.
    #[serde(rename = "both")]
    Both,
    /// The icon is scaled in the y-dimension to fit the height of the text.
    #[serde(rename = "height")]
    Height,
    /// The icon is displayed at its intrinsic aspect ratio.
    #[serde(rename = "none")]
    None,
    /// The icon is scaled in the x-dimension to fit the width of the text.
    #[serde(rename = "width")]
    Width,
}

impl Default for IconTextFit {
    fn default() -> Self {
        Self::None
    }
}

/// Size of the additional area added to dimensions determined by `icon-text-fit`, in clockwise order: top, right, bottom, left.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconTextFitPadding(serde_json::Value);

impl Default for IconTextFitPadding {
    fn default() -> Self {
        vec![0, 0, 0, 0]
    }
}

/// If true, the symbols will not cross tile edges to avoid mutual collisions. Recommended in layers that don't have enough padding in the vector tile to prevent collisions, or if it is a point symbol layer placed after a line symbol layer. When using a client that supports global collision detection, like MapLibre GL JS version 0.42.0 or greater, enabling this property is not needed to prevent clipped labels at tile boundaries.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct SymbolAvoidEdges(serde_json::Value);

impl Default for SymbolAvoidEdges {
    fn default() -> Self {
        false
    }
}

/// Label placement relative to its geometry.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SymbolPlacement {
    /// The label is placed along the line of the geometry. Can only be used on `LineString` and `Polygon` geometries.
    #[serde(rename = "line")]
    Line,
    /// The label is placed at the center of the line of the geometry. Can only be used on `LineString` and `Polygon` geometries. Note that a single feature in a vector tile may contain multiple line geometries.
    #[serde(rename = "line-center")]
    LineCenter,
    /// The label is placed at the point where the geometry is located.
    #[serde(rename = "point")]
    Point,
}

impl Default for SymbolPlacement {
    fn default() -> Self {
        Self::Point
    }
}

/// Sorts features in ascending order based on this value. Features with lower sort keys are drawn and placed first.  When `icon-allow-overlap` or `text-allow-overlap` is `false`, features with a lower sort key will have priority during placement. When `icon-allow-overlap` or `text-allow-overlap` is set to `true`, features with a higher sort key will overlap over features with a lower sort key.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SymbolSortKey(serde_json::Number);

/// Distance between two symbol anchors.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SymbolSpacing(serde_json::Number);

impl Default for SymbolSpacing {
    fn default() -> Self {
        Self(250.into())
    }
}

/// Determines whether overlapping symbols in the same layer are rendered in the order that they appear in the data source or by their y-position relative to the viewport. To control the order and prioritization of symbols otherwise, use `symbol-sort-key`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SymbolZOrder {
    /// Sorts symbols by `symbol-sort-key` if set. Otherwise, sorts symbols by their y-position relative to the viewport if `icon-allow-overlap` or `text-allow-overlap` is set to `true` or `icon-ignore-placement` or `text-ignore-placement` is `false`.
    #[serde(rename = "auto")]
    Auto,
    /// Sorts symbols by `symbol-sort-key` if set. Otherwise, no sorting is applied; symbols are rendered in the same order as the source data.
    #[serde(rename = "source")]
    Source,
    /// Sorts symbols by their y-position relative to the viewport if `icon-allow-overlap` or `text-allow-overlap` is set to `true` or `icon-ignore-placement` or `text-ignore-placement` is `false`.
    #[serde(rename = "viewport-y")]
    ViewportY,
}

impl Default for SymbolZOrder {
    fn default() -> Self {
        Self::Auto
    }
}

/// If true, the text will be visible even if it collides with other previously drawn symbols.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextAllowOverlap(serde_json::Value);

impl Default for TextAllowOverlap {
    fn default() -> Self {
        false
    }
}

/// Part of the text placed closest to the anchor.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextAnchor {
    /// The bottom of the text is placed closest to the anchor.
    #[serde(rename = "bottom")]
    Bottom,
    /// The bottom left corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-left")]
    BottomLeft,
    /// The bottom right corner of the text is placed closest to the anchor.
    #[serde(rename = "bottom-right")]
    BottomRight,
    /// The center of the text is placed closest to the anchor.
    #[serde(rename = "center")]
    Center,
    /// The left side of the text is placed closest to the anchor.
    #[serde(rename = "left")]
    Left,
    /// The right side of the text is placed closest to the anchor.
    #[serde(rename = "right")]
    Right,
    /// The top of the text is placed closest to the anchor.
    #[serde(rename = "top")]
    Top,
    /// The top left corner of the text is placed closest to the anchor.
    #[serde(rename = "top-left")]
    TopLeft,
    /// The top right corner of the text is placed closest to the anchor.
    #[serde(rename = "top-right")]
    TopRight,
}

impl Default for TextAnchor {
    fn default() -> Self {
        Self::Center
    }
}

/// Value to use for a text label. If a plain `string` is provided, it will be treated as a `formatted` with default/inherited formatting options.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextField(serde_json::Value);

impl Default for TextField {
    fn default() -> Self {}
}

/// Fonts to use for displaying text. If the `glyphs` root property is specified, this array is joined together and interpreted as a font stack name. Otherwise, it is interpreted as a cascading fallback list of local font names.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextFont(serde_json::Value);

impl Default for TextFont {
    fn default() -> Self {
        vec!["Open Sans Regular", "Arial Unicode MS Regular"]
    }
}

/// If true, other symbols can be visible even if they collide with the text.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextIgnorePlacement(serde_json::Value);

impl Default for TextIgnorePlacement {
    fn default() -> Self {
        false
    }
}

/// Text justification options.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextJustify {
    /// The text is aligned towards the anchor position.
    #[serde(rename = "auto")]
    Auto,
    /// The text is centered.
    #[serde(rename = "center")]
    Center,
    /// The text is aligned to the left.
    #[serde(rename = "left")]
    Left,
    /// The text is aligned to the right.
    #[serde(rename = "right")]
    Right,
}

impl Default for TextJustify {
    fn default() -> Self {
        Self::Center
    }
}

/// If true, the text may be flipped vertically to prevent it from being rendered upside-down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextKeepUpright(serde_json::Value);

impl Default for TextKeepUpright {
    fn default() -> Self {
        true
    }
}

/// Text tracking amount.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextLetterSpacing(serde_json::Number);

impl Default for TextLetterSpacing {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Text leading value for multi-line text.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextLineHeight(serde_json::Number);

impl Default for TextLineHeight {
    fn default() -> Self {
        Self(1.2.into())
    }
}

/// Maximum angle change between adjacent characters.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextMaxAngle(serde_json::Number);

impl Default for TextMaxAngle {
    fn default() -> Self {
        Self(45.into())
    }
}

/// The maximum line width for text wrapping.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextMaxWidth(serde_json::Number);

impl Default for TextMaxWidth {
    fn default() -> Self {
        Self(10.into())
    }
}

/// Offset distance of text from its anchor. Positive values indicate right and down, while negative values indicate left and up. If used with text-variable-anchor, input values will be taken as absolute values. Offsets along the x- and y-axis will be applied automatically based on the anchor position.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextOffset(serde_json::Value);

impl Default for TextOffset {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// If true, icons will display without their corresponding text when the text collides with other symbols and the icon does not.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextOptional(serde_json::Value);

impl Default for TextOptional {
    fn default() -> Self {
        false
    }
}

/// Allows for control over whether to show symbol text when it overlaps other symbols on the map. If `text-overlap` is not set, `text-allow-overlap` is used instead
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextOverlap {
    /// The text will be visible even if it collides with any other previously drawn symbol.
    #[serde(rename = "always")]
    Always,
    /// If the text collides with another previously drawn symbol, the overlap mode for that symbol is checked. If the previous symbol was placed using `never` overlap mode, the new text is hidden. If the previous symbol was placed using `always` or `cooperative` overlap mode, the new text is visible.
    #[serde(rename = "cooperative")]
    Cooperative,
    /// The text will be hidden if it collides with any other previously drawn symbol.
    #[serde(rename = "never")]
    Never,
}

/// Size of the additional area around the text bounding box used for detecting symbol collisions.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextPadding(serde_json::Number);

impl Default for TextPadding {
    fn default() -> Self {
        Self(2.into())
    }
}

/// Orientation of text when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextPitchAlignment {
    /// Automatically matches the value of `text-rotation-alignment`.
    #[serde(rename = "auto")]
    Auto,
    /// The text is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The text is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for TextPitchAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Radial offset of text, in the direction of the symbol's anchor. Useful in combination with `text-variable-anchor`, which defaults to using the two-dimensional `text-offset` if present.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextRadialOffset(serde_json::Number);

impl Default for TextRadialOffset {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Rotates the text clockwise.
///
/// Range: ..every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextRotate(serde_json::Number);

impl Default for TextRotate {
    fn default() -> Self {
        Self(0.into())
    }
}

/// In combination with `symbol-placement`, determines the rotation behavior of the individual glyphs forming the text.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextRotationAlignment {
    /// When `symbol-placement` is set to `point`, this is equivalent to `viewport`. When `symbol-placement` is set to `line` or `line-center`, this is equivalent to `map`.
    #[serde(rename = "auto")]
    Auto,
    /// When `symbol-placement` is set to `point`, aligns text east-west. When `symbol-placement` is set to `line` or `line-center`, aligns text x-axes with the line.
    #[serde(rename = "map")]
    Map,
    /// Produces glyphs whose x-axes are aligned with the x-axis of the viewport, regardless of the value of `symbol-placement`.
    #[serde(rename = "viewport")]
    Viewport,
    /// When `symbol-placement` is set to `point`, aligns text to the x-axis of the viewport. When `symbol-placement` is set to `line` or `line-center`, aligns glyphs to the x-axis of the viewport and places them along the line.
    #[serde(rename = "viewport-glyph")]
    ViewportGlyph,
}

impl Default for TextRotationAlignment {
    fn default() -> Self {
        Self::Auto
    }
}

/// Font size.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextSize(serde_json::Number);

impl Default for TextSize {
    fn default() -> Self {
        Self(16.into())
    }
}

/// Specifies how to capitalize text, similar to the CSS `text-transform` property.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextTransform {
    /// Forces all letters to be displayed in lowercase.
    #[serde(rename = "lowercase")]
    Lowercase,
    /// The text is not altered.
    #[serde(rename = "none")]
    None,
    /// Forces all letters to be displayed in uppercase.
    #[serde(rename = "uppercase")]
    Uppercase,
}

impl Default for TextTransform {
    fn default() -> Self {
        Self::None
    }
}

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations: the renderer will attempt to place the label at each location, in order, before moving onto the next label. Use `text-justify: auto` to choose justification based on anchor position. To apply an offset, use the `text-radial-offset` or the two-dimensional `text-offset`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextVariableAnchor(serde_json::Value);

/// To increase the chance of placing high-priority labels on the map, you can provide an array of `text-anchor` locations, each paired with an offset value. The renderer will attempt to place the label at each location, in order, before moving on to the next location+offset. Use `text-justify: auto` to choose justification based on anchor position.
///
///  The length of the array must be even, and must alternate between enum and point entries. i.e., each anchor location must be accompanied by a point, and that point defines the offset when the corresponding anchor location is used. Positive offset values indicate right and down, while negative values indicate left and up. Anchor locations may repeat, allowing the renderer to try multiple offsets to try and place a label using the same anchor.
///
///  When present, this property takes precedence over `text-anchor`, `text-variable-anchor`, `text-offset`, and `text-radial-offset`.
///
///  ```json
///
///  { "text-variable-anchor-offset": ["top", [0, 4], "left", [3,0], "bottom", [1, 1]] }
///
///  ```
///
///  When the renderer chooses the `top` anchor, `[0, 4]` will be used for `text-offset`; the text will be shifted down by 4 ems.
///
///  When the renderer chooses the `left` anchor, `[3, 0]` will be used for `text-offset`; the text will be shifted right by 3 ems.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextVariableAnchorOffset(serde_json::Value);

/// The property allows control over a symbol's orientation. Note that the property values act as a hint, so that a symbol whose language doesn’t support the provided orientation will be laid out in its natural orientation. Example: English point symbol will be rendered horizontally even if array value contains single 'vertical' enum value. The order of elements in an array define priority order for the placement of an orientation variant.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextWritingMode(serde_json::Value);

/// Whether this layer is displayed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Visibility {
    /// The layer is not shown.
    #[serde(rename = "none")]
    None,
    /// The layer is shown.
    #[serde(rename = "visible")]
    Visible,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Visible
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Light {
    /// Whether extruded geometries are lit relative to the map or viewport.
    #[serde(rename = "anchor")]
    pub anchor: Anchor,
    /// Color tint for lighting extruded geometries.
    #[serde(rename = "color")]
    pub color: Color,
    /// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
    #[serde(rename = "intensity")]
    pub intensity: Intensity,
    /// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
    #[serde(rename = "position")]
    pub position: Position,
}

/// Whether extruded geometries are lit relative to the map or viewport.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Anchor {
    /// The position of the light source is aligned to the rotation of the map.
    #[serde(rename = "map")]
    Map,
    /// The position of the light source is aligned to the rotation of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for Anchor {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Color tint for lighting extruded geometries.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct Color(color::DynamicColor);

impl Default for Color {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// Intensity of lighting (on a scale from 0 to 1). Higher numbers will present as more extreme contrast.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Intensity(serde_json::Number);

impl Default for Intensity {
    fn default() -> Self {
        Self(0.5.into())
    }
}

/// Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Position(serde_json::Value);

impl Default for Position {
    fn default() -> Self {
        vec![1.15, 210, 30]
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Paint {
    PaintFill(PaintFill),
    PaintLine(PaintLine),
    PaintCircle(PaintCircle),
    PaintHeatmap(PaintHeatmap),
    PaintFillExtrusion(PaintFillExtrusion),
    PaintSymbol(PaintSymbol),
    PaintRaster(PaintRaster),
    PaintHillshade(PaintHillshade),
    PaintColorRelief(PaintColorRelief),
    PaintBackground(PaintBackground),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintBackground {
    /// The color with which the background will be drawn.
    #[serde(rename = "background-color")]
    pub background_color: BackgroundColor,
    /// The opacity at which the background will be drawn.
    #[serde(rename = "background-opacity")]
    pub background_opacity: BackgroundOpacity,
    /// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "background-pattern")]
    pub background_pattern: BackgroundPattern,
}

/// The color with which the background will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct BackgroundColor(color::DynamicColor);

impl Default for BackgroundColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity at which the background will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct BackgroundOpacity(serde_json::Number);

impl Default for BackgroundOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Name of image in sprite to use for drawing an image background. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct BackgroundPattern(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintCircle {
    /// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
    #[serde(rename = "circle-blur")]
    pub circle_blur: CircleBlur,
    /// The fill color of the circle.
    #[serde(rename = "circle-color")]
    pub circle_color: CircleColor,
    /// The opacity at which the circle will be drawn.
    #[serde(rename = "circle-opacity")]
    pub circle_opacity: CircleOpacity,
    /// Orientation of circle when map is pitched.
    #[serde(rename = "circle-pitch-alignment")]
    pub circle_pitch_alignment: CirclePitchAlignment,
    /// Controls the scaling behavior of the circle when the map is pitched.
    #[serde(rename = "circle-pitch-scale")]
    pub circle_pitch_scale: CirclePitchScale,
    /// Circle radius.
    #[serde(rename = "circle-radius")]
    pub circle_radius: CircleRadius,
    /// The stroke color of the circle.
    #[serde(rename = "circle-stroke-color")]
    pub circle_stroke_color: CircleStrokeColor,
    /// The opacity of the circle's stroke.
    #[serde(rename = "circle-stroke-opacity")]
    pub circle_stroke_opacity: CircleStrokeOpacity,
    /// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
    #[serde(rename = "circle-stroke-width")]
    pub circle_stroke_width: CircleStrokeWidth,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "circle-translate")]
    pub circle_translate: CircleTranslate,
    /// Controls the frame of reference for `circle-translate`.
    #[serde(rename = "circle-translate-anchor")]
    pub circle_translate_anchor: CircleTranslateAnchor,
}

/// Amount to blur the circle. 1 blurs the circle such that only the centerpoint is full opacity.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleBlur(serde_json::Number);

impl Default for CircleBlur {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The fill color of the circle.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct CircleColor(color::DynamicColor);

impl Default for CircleColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity at which the circle will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleOpacity(serde_json::Number);

impl Default for CircleOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Orientation of circle when map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CirclePitchAlignment {
    /// The circle is aligned to the plane of the map.
    #[serde(rename = "map")]
    Map,
    /// The circle is aligned to the plane of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for CirclePitchAlignment {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Controls the scaling behavior of the circle when the map is pitched.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CirclePitchScale {
    /// Circles are scaled according to their apparent distance to the camera.
    #[serde(rename = "map")]
    Map,
    /// Circles are not scaled.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for CirclePitchScale {
    fn default() -> Self {
        Self::Map
    }
}

/// Circle radius.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleRadius(serde_json::Number);

impl Default for CircleRadius {
    fn default() -> Self {
        Self(5.into())
    }
}

/// The stroke color of the circle.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct CircleStrokeColor(color::DynamicColor);

impl Default for CircleStrokeColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity of the circle's stroke.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleStrokeOpacity(serde_json::Number);

impl Default for CircleStrokeOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// The width of the circle's stroke. Strokes are placed outside of the `circle-radius`.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct CircleStrokeWidth(serde_json::Number);

impl Default for CircleStrokeWidth {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct CircleTranslate(serde_json::Value);

impl Default for CircleTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `circle-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CircleTranslateAnchor {
    /// The circle is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The circle is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for CircleTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintColorRelief {
    /// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
    #[serde(rename = "color-relief-color")]
    pub color_relief_color: ColorReliefColor,
    /// The opacity at which the color-relief will be drawn.
    #[serde(rename = "color-relief-opacity")]
    pub color_relief_opacity: ColorReliefOpacity,
}

/// Defines the color of each pixel based on its elevation. Should be an expression that uses `["elevation"]` as input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct ColorReliefColor(color::DynamicColor);

/// The opacity at which the color-relief will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct ColorReliefOpacity(serde_json::Number);

impl Default for ColorReliefOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFill {
    /// Whether or not the fill should be antialiased.
    #[serde(rename = "fill-antialias")]
    pub fill_antialias: FillAntialias,
    /// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
    #[serde(rename = "fill-color")]
    pub fill_color: FillColor,
    /// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
    #[serde(rename = "fill-opacity")]
    pub fill_opacity: FillOpacity,
    /// The outline color of the fill. Matches the value of `fill-color` if unspecified.
    #[serde(rename = "fill-outline-color")]
    pub fill_outline_color: FillOutlineColor,
    /// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-pattern")]
    pub fill_pattern: FillPattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "fill-translate")]
    pub fill_translate: FillTranslate,
    /// Controls the frame of reference for `fill-translate`.
    #[serde(rename = "fill-translate-anchor")]
    pub fill_translate_anchor: FillTranslateAnchor,
}

/// Whether or not the fill should be antialiased.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillAntialias(serde_json::Value);

impl Default for FillAntialias {
    fn default() -> Self {
        true
    }
}

/// The color of the filled part of this layer. This color can be specified as `rgba` with an alpha component and the color's opacity will not affect the opacity of the 1px stroke, if it is used.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FillColor(color::DynamicColor);

impl Default for FillColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The opacity of the entire fill layer. In contrast to the `fill-color`, this value will also affect the 1px stroke around the fill, if the stroke is used.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FillOpacity(serde_json::Number);

impl Default for FillOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// The outline color of the fill. Matches the value of `fill-color` if unspecified.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FillOutlineColor(color::DynamicColor);

/// Name of image in sprite to use for drawing image fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillPattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillTranslate(serde_json::Value);

impl Default for FillTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `fill-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FillTranslateAnchor {
    /// The fill is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The fill is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for FillTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintFillExtrusion {
    /// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
    #[serde(rename = "fill-extrusion-base")]
    pub fill_extrusion_base: FillExtrusionBase,
    /// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
    #[serde(rename = "fill-extrusion-color")]
    pub fill_extrusion_color: FillExtrusionColor,
    /// The height with which to extrude this layer.
    #[serde(rename = "fill-extrusion-height")]
    pub fill_extrusion_height: FillExtrusionHeight,
    /// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
    #[serde(rename = "fill-extrusion-opacity")]
    pub fill_extrusion_opacity: FillExtrusionOpacity,
    /// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "fill-extrusion-pattern")]
    pub fill_extrusion_pattern: FillExtrusionPattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
    #[serde(rename = "fill-extrusion-translate")]
    pub fill_extrusion_translate: FillExtrusionTranslate,
    /// Controls the frame of reference for `fill-extrusion-translate`.
    #[serde(rename = "fill-extrusion-translate-anchor")]
    pub fill_extrusion_translate_anchor: FillExtrusionTranslateAnchor,
    /// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
    #[serde(rename = "fill-extrusion-vertical-gradient")]
    pub fill_extrusion_vertical_gradient: FillExtrusionVerticalGradient,
}

/// The height with which to extrude the base of this layer. Must be less than or equal to `fill-extrusion-height`.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FillExtrusionBase(serde_json::Number);

impl Default for FillExtrusionBase {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The base color of the extruded fill. The extrusion's surfaces will be shaded differently based on this color in combination with the root `light` settings. If this color is specified as `rgba` with an alpha component, the alpha component will be ignored; use `fill-extrusion-opacity` to set layer opacity.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FillExtrusionColor(color::DynamicColor);

impl Default for FillExtrusionColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The height with which to extrude this layer.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FillExtrusionHeight(serde_json::Number);

impl Default for FillExtrusionHeight {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The opacity of the entire fill extrusion layer. This is rendered on a per-layer, not per-feature, basis, and data-driven styling is not available.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FillExtrusionOpacity(serde_json::Number);

impl Default for FillExtrusionOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Name of image in sprite to use for drawing images on extruded fills. For seamless patterns, image width and height must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillExtrusionPattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up (on the flat plane), respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillExtrusionTranslate(serde_json::Value);

impl Default for FillExtrusionTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `fill-extrusion-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum FillExtrusionTranslateAnchor {
    /// The fill extrusion is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The fill extrusion is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for FillExtrusionTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// Whether to apply a vertical gradient to the sides of a fill-extrusion layer. If true, sides will be shaded slightly darker farther down.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct FillExtrusionVerticalGradient(serde_json::Value);

impl Default for FillExtrusionVerticalGradient {
    fn default() -> Self {
        true
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHeatmap {
    /// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
    #[serde(rename = "heatmap-color")]
    pub heatmap_color: HeatmapColor,
    /// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
    #[serde(rename = "heatmap-intensity")]
    pub heatmap_intensity: HeatmapIntensity,
    /// The global opacity at which the heatmap layer will be drawn.
    #[serde(rename = "heatmap-opacity")]
    pub heatmap_opacity: HeatmapOpacity,
    /// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
    #[serde(rename = "heatmap-radius")]
    pub heatmap_radius: HeatmapRadius,
    /// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
    #[serde(rename = "heatmap-weight")]
    pub heatmap_weight: HeatmapWeight,
}

/// Defines the color of each pixel based on its density value in a heatmap.  Should be an expression that uses `["heatmap-density"]` as input.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct HeatmapColor(color::DynamicColor);

impl Default for HeatmapColor {
    fn default() -> Self {
        Self(
            color::parse_color([
                "interpolate",
                ["linear"],
                ["heatmap-density"],
                0,
                "rgba(0, 0, 255, 0)",
                0.1,
                "royalblue",
                0.3,
                "cyan",
                0.5,
                "lime",
                0.7,
                "yellow",
                1,
                "red",
            ])
            .expect("Invalid color specified as the default value"),
        )
    }
}

/// Similar to `heatmap-weight` but controls the intensity of the heatmap globally. Primarily used for adjusting the heatmap based on zoom level.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HeatmapIntensity(serde_json::Number);

impl Default for HeatmapIntensity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// The global opacity at which the heatmap layer will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HeatmapOpacity(serde_json::Number);

impl Default for HeatmapOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Radius of influence of one heatmap point in pixels. Increasing the value makes the heatmap smoother, but less detailed.
///
/// Range: 1..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HeatmapRadius(serde_json::Number);

impl Default for HeatmapRadius {
    fn default() -> Self {
        Self(30.into())
    }
}

/// A measure of how much an individual point contributes to the heatmap. A value of 10 would be equivalent to having 10 points of weight 1 in the same spot. Especially useful when combined with clustering.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HeatmapWeight(serde_json::Number);

impl Default for HeatmapWeight {
    fn default() -> Self {
        Self(1.into())
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintHillshade {
    /// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
    #[serde(rename = "hillshade-accent-color")]
    pub hillshade_accent_color: HillshadeAccentColor,
    /// Intensity of the hillshade
    #[serde(rename = "hillshade-exaggeration")]
    pub hillshade_exaggeration: HillshadeExaggeration,
    /// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-highlight-color")]
    pub hillshade_highlight_color: HillshadeHighlightColor,
    /// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-altitude")]
    pub hillshade_illumination_altitude: HillshadeIlluminationAltitude,
    /// Direction of light source when map is rotated.
    #[serde(rename = "hillshade-illumination-anchor")]
    pub hillshade_illumination_anchor: HillshadeIlluminationAnchor,
    /// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-illumination-direction")]
    pub hillshade_illumination_direction: HillshadeIlluminationDirection,
    /// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
    #[serde(rename = "hillshade-method")]
    pub hillshade_method: HillshadeMethod,
    /// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
    #[serde(rename = "hillshade-shadow-color")]
    pub hillshade_shadow_color: HillshadeShadowColor,
}

/// The shading color used to accentuate rugged terrain like sharp cliffs and gorges.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct HillshadeAccentColor(color::DynamicColor);

impl Default for HillshadeAccentColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Intensity of the hillshade
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HillshadeExaggeration(serde_json::Number);

impl Default for HillshadeExaggeration {
    fn default() -> Self {
        Self(0.5.into())
    }
}

/// The shading color of areas that faces towards the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
enum HillshadeHighlightColor {
    /// A color
    One(color::DynamicColor),
    /// A set of colors
    Multiple(Vec<color::DynamicColor>),
}

impl Default for HillshadeHighlightColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#FFFFFF").expect("Invalid color specified as the default value"),
        )
    }
}

/// The altitude of the light source(s) used to generate the hillshading with 0 as sunset and 90 as noon. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
///
/// Range: 0..=90
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct HillshadeIlluminationAltitude(serde_json::Value);

impl Default for HillshadeIlluminationAltitude {
    fn default() -> Self {
        45
    }
}

/// Direction of light source when map is rotated.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum HillshadeIlluminationAnchor {
    /// The hillshade illumination is relative to the north direction.
    #[serde(rename = "map")]
    Map,
    /// The hillshade illumination is relative to the top of the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for HillshadeIlluminationAnchor {
    fn default() -> Self {
        Self::Viewport
    }
}

/// The direction of the light source(s) used to generate the hillshading with 0 as the top of the viewport if `hillshade-illumination-anchor` is set to `viewport` and due north if `hillshade-illumination-anchor` is set to `map`. Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
///
/// Range: 0..=359
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct HillshadeIlluminationDirection(serde_json::Value);

impl Default for HillshadeIlluminationDirection {
    fn default() -> Self {
        335
    }
}

/// The hillshade algorithm to use, one of `standard`, `basic`, `combined`, `igor`, or `multidirectional`. ![image](assets/hillshade_methods.png)
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum HillshadeMethod {
    /// Basic hillshade. Uses a simple physics model where the reflected light intensity is proportional to the cosine of the angle between the incident light and the surface normal. Similar to GDAL's `gdaldem` default algorithm.
    #[serde(rename = "basic")]
    Basic,
    /// Hillshade algorithm whose intensity scales with slope. Similar to GDAL's `gdaldem` with `-combined` option.
    #[serde(rename = "combined")]
    Combined,
    /// Hillshade algorithm which tries to minimize effects on other map features beneath. Similar to GDAL's `gdaldem` with `-igor` option.
    #[serde(rename = "igor")]
    Igor,
    /// Hillshade with multiple illumination directions. Uses the basic hillshade model with multiple independent light sources.
    #[serde(rename = "multidirectional")]
    Multidirectional,
    /// The legacy hillshade method.
    #[serde(rename = "standard")]
    Standard,
}

impl Default for HillshadeMethod {
    fn default() -> Self {
        Self::Standard
    }
}

/// The shading color of areas that face away from the light source(s). Only when `hillshade-method` is set to `multidirectional` can you specify multiple light sources.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
enum HillshadeShadowColor {
    /// A color
    One(color::DynamicColor),
    /// A set of colors
    Multiple(Vec<color::DynamicColor>),
}

impl Default for HillshadeShadowColor {
    fn default() -> Self {
        Self::One(
            color::parse_color("#000000").expect("Invalid color specified as the default value"),
        )
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintLine {
    /// Blur applied to the line, in pixels.
    #[serde(rename = "line-blur")]
    pub line_blur: LineBlur,
    /// The color with which the line will be drawn.
    #[serde(rename = "line-color")]
    pub line_color: LineColor,
    /// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
    #[serde(rename = "line-dasharray")]
    pub line_dasharray: LineDasharray,
    /// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
    #[serde(rename = "line-gap-width")]
    pub line_gap_width: LineGapWidth,
    /// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
    #[serde(rename = "line-gradient")]
    pub line_gradient: LineGradient,
    /// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
    #[serde(rename = "line-offset")]
    pub line_offset: LineOffset,
    /// The opacity at which the line will be drawn.
    #[serde(rename = "line-opacity")]
    pub line_opacity: LineOpacity,
    /// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
    #[serde(rename = "line-pattern")]
    pub line_pattern: LinePattern,
    /// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
    #[serde(rename = "line-translate")]
    pub line_translate: LineTranslate,
    /// Controls the frame of reference for `line-translate`.
    #[serde(rename = "line-translate-anchor")]
    pub line_translate_anchor: LineTranslateAnchor,
    /// Stroke thickness.
    #[serde(rename = "line-width")]
    pub line_width: LineWidth,
}

/// Blur applied to the line, in pixels.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineBlur(serde_json::Number);

impl Default for LineBlur {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The color with which the line will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LineColor(color::DynamicColor);

impl Default for LineColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Specifies the lengths of the alternating dashes and gaps that form the dash pattern. The lengths are later scaled by the line width. To convert a dash length to pixels, multiply the length by the current line width. GeoJSON sources with `lineMetrics: true` specified won't render dashed lines to the expected scale. Zoom-dependent expressions will be evaluated only at integer zoom levels. The only way to create an array value is using `["literal", [...]]`; arrays cannot be read from or derived from feature properties.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct LineDasharray(serde_json::Value);

/// Draws a line casing outside of a line's actual path. Value indicates the width of the inner gap.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineGapWidth(serde_json::Number);

impl Default for LineGapWidth {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Defines a gradient with which to color a line feature. Can only be used with GeoJSON sources that specify `"lineMetrics": true`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct LineGradient(color::DynamicColor);

/// The line's offset. For linear features, a positive value offsets the line to the right, relative to the direction of the line, and a negative value to the left. For polygon features, a positive value results in an inset, and a negative value results in an outset.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineOffset(serde_json::Number);

impl Default for LineOffset {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The opacity at which the line will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineOpacity(serde_json::Number);

impl Default for LineOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Name of image in sprite to use for drawing image lines. For seamless patterns, image width must be a factor of two (2, 4, 8, ..., 512). Note that zoom-dependent expressions will be evaluated only at integer zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct LinePattern(serde_json::Value);

/// The geometry's offset. Values are [x, y] where negatives indicate left and up, respectively.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct LineTranslate(serde_json::Value);

impl Default for LineTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `line-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum LineTranslateAnchor {
    /// The line is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The line is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for LineTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// Stroke thickness.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct LineWidth(serde_json::Number);

impl Default for LineWidth {
    fn default() -> Self {
        Self(1.into())
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintRaster {
    /// Increase or reduce the brightness of the image. The value is the maximum brightness.
    #[serde(rename = "raster-brightness-max")]
    pub raster_brightness_max: RasterBrightnessMax,
    /// Increase or reduce the brightness of the image. The value is the minimum brightness.
    #[serde(rename = "raster-brightness-min")]
    pub raster_brightness_min: RasterBrightnessMin,
    /// Increase or reduce the contrast of the image.
    #[serde(rename = "raster-contrast")]
    pub raster_contrast: RasterContrast,
    /// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
    #[serde(rename = "raster-fade-duration")]
    pub raster_fade_duration: RasterFadeDuration,
    /// Rotates hues around the color wheel.
    #[serde(rename = "raster-hue-rotate")]
    pub raster_hue_rotate: RasterHueRotate,
    /// The opacity at which the image will be drawn.
    #[serde(rename = "raster-opacity")]
    pub raster_opacity: RasterOpacity,
    /// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
    #[serde(rename = "raster-resampling")]
    pub raster_resampling: RasterResampling,
    /// Increase or reduce the saturation of the image.
    #[serde(rename = "raster-saturation")]
    pub raster_saturation: RasterSaturation,
}

/// Increase or reduce the brightness of the image. The value is the maximum brightness.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterBrightnessMax(serde_json::Number);

impl Default for RasterBrightnessMax {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Increase or reduce the brightness of the image. The value is the minimum brightness.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterBrightnessMin(serde_json::Number);

impl Default for RasterBrightnessMin {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Increase or reduce the contrast of the image.
///
/// Range: -1..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterContrast(serde_json::Number);

impl Default for RasterContrast {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Fade duration when a new tile is added, or when a video is started or its coordinates are updated.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterFadeDuration(serde_json::Number);

impl Default for RasterFadeDuration {
    fn default() -> Self {
        Self(300.into())
    }
}

/// Rotates hues around the color wheel.
///
/// Range: ..every 360
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterHueRotate(serde_json::Number);

impl Default for RasterHueRotate {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The opacity at which the image will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterOpacity(serde_json::Number);

impl Default for RasterOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// The resampling/interpolation method to use for overscaling, also known as texture magnification filter
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum RasterResampling {
    /// (Bi)linear filtering interpolates pixel values using the weighted average of the four closest original source pixels creating a smooth but blurry look when overscaled
    #[serde(rename = "linear")]
    Linear,
    /// Nearest neighbor filtering interpolates pixel values using the nearest original source pixel creating a sharp but pixelated look when overscaled
    #[serde(rename = "nearest")]
    Nearest,
}

impl Default for RasterResampling {
    fn default() -> Self {
        Self::Linear
    }
}

/// Increase or reduce the saturation of the image.
///
/// Range: -1..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct RasterSaturation(serde_json::Number);

impl Default for RasterSaturation {
    fn default() -> Self {
        Self(0.into())
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PaintSymbol {
    /// The color of the icon. This can only be used with SDF icons.
    #[serde(rename = "icon-color")]
    pub icon_color: IconColor,
    /// Fade out the halo towards the outside.
    #[serde(rename = "icon-halo-blur")]
    pub icon_halo_blur: IconHaloBlur,
    /// The color of the icon's halo. Icon halos can only be used with SDF icons.
    #[serde(rename = "icon-halo-color")]
    pub icon_halo_color: IconHaloColor,
    /// Distance of halo to the icon outline.
    ///
    /// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
    #[serde(rename = "icon-halo-width")]
    pub icon_halo_width: IconHaloWidth,
    /// The opacity at which the icon will be drawn.
    #[serde(rename = "icon-opacity")]
    pub icon_opacity: IconOpacity,
    /// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "icon-translate")]
    pub icon_translate: IconTranslate,
    /// Controls the frame of reference for `icon-translate`.
    #[serde(rename = "icon-translate-anchor")]
    pub icon_translate_anchor: IconTranslateAnchor,
    /// The color with which the text will be drawn.
    #[serde(rename = "text-color")]
    pub text_color: TextColor,
    /// The halo's fadeout distance towards the outside.
    #[serde(rename = "text-halo-blur")]
    pub text_halo_blur: TextHaloBlur,
    /// The color of the text's halo, which helps it stand out from backgrounds.
    #[serde(rename = "text-halo-color")]
    pub text_halo_color: TextHaloColor,
    /// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
    #[serde(rename = "text-halo-width")]
    pub text_halo_width: TextHaloWidth,
    /// The opacity at which the text will be drawn.
    #[serde(rename = "text-opacity")]
    pub text_opacity: TextOpacity,
    /// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
    #[serde(rename = "text-translate")]
    pub text_translate: TextTranslate,
    /// Controls the frame of reference for `text-translate`.
    #[serde(rename = "text-translate-anchor")]
    pub text_translate_anchor: TextTranslateAnchor,
}

/// The color of the icon. This can only be used with SDF icons.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct IconColor(color::DynamicColor);

impl Default for IconColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// Fade out the halo towards the outside.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct IconHaloBlur(serde_json::Number);

impl Default for IconHaloBlur {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The color of the icon's halo. Icon halos can only be used with SDF icons.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct IconHaloColor(color::DynamicColor);

impl Default for IconHaloColor {
    fn default() -> Self {
        Self(
            color::parse_color("rgba(0, 0, 0, 0)")
                .expect("Invalid color specified as the default value"),
        )
    }
}

/// Distance of halo to the icon outline.
///
/// The unit is in pixels only for SDF sprites that were created with a blur radius of 8, multiplied by the display density. I.e., the radius needs to be 16 for `@2x` sprites, etc.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct IconHaloWidth(serde_json::Number);

impl Default for IconHaloWidth {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The opacity at which the icon will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct IconOpacity(serde_json::Number);

impl Default for IconOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Distance that the icon's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct IconTranslate(serde_json::Value);

impl Default for IconTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `icon-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum IconTranslateAnchor {
    /// Icons are translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// Icons are translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for IconTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

/// The color with which the text will be drawn.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct TextColor(color::DynamicColor);

impl Default for TextColor {
    fn default() -> Self {
        Self(color::parse_color("#000000").expect("Invalid color specified as the default value"))
    }
}

/// The halo's fadeout distance towards the outside.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextHaloBlur(serde_json::Number);

impl Default for TextHaloBlur {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The color of the text's halo, which helps it stand out from backgrounds.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct TextHaloColor(color::DynamicColor);

impl Default for TextHaloColor {
    fn default() -> Self {
        Self(
            color::parse_color("rgba(0, 0, 0, 0)")
                .expect("Invalid color specified as the default value"),
        )
    }
}

/// Distance of halo to the font outline. Max text halo width is 1/4 of the font-size.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextHaloWidth(serde_json::Number);

impl Default for TextHaloWidth {
    fn default() -> Self {
        Self(0.into())
    }
}

/// The opacity at which the text will be drawn.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct TextOpacity(serde_json::Number);

impl Default for TextOpacity {
    fn default() -> Self {
        Self(1.into())
    }
}

/// Distance that the text's anchor is moved from its original placement. Positive values indicate right and down, while negative values indicate left and up.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct TextTranslate(serde_json::Value);

impl Default for TextTranslate {
    fn default() -> Self {
        vec![0, 0]
    }
}

/// Controls the frame of reference for `text-translate`.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TextTranslateAnchor {
    /// The text is translated relative to the map.
    #[serde(rename = "map")]
    Map,
    /// The text is translated relative to the viewport.
    #[serde(rename = "viewport")]
    Viewport,
}

impl Default for TextTranslateAnchor {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Projection {
    /// The projection definition type. Can be specified as a string, a transition state, or an expression.
    #[serde(rename = "type")]
    pub r#type: Type,
}

/// The projection definition type. Can be specified as a string, a transition state, or an expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Type(serde_json::Value);

impl Default for Type {
    fn default() -> Self {
        mercator
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Promoteid {
    /// A name of a feature property to use as ID for feature state.
    #[serde(rename = "*")]
    pub star: Star,
}

/// A name of a feature property to use as ID for feature state.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Star(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct PropertyType {
    /// Property should be specified using a color ramp from which the output color can be sampled based on a property calculation.
    #[serde(rename = "color-ramp")]
    pub color_ramp: ColorRamp,
    /// Property is constant across all zoom levels and property values.
    #[serde(rename = "constant")]
    pub constant: Constant,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms.
    #[serde(rename = "cross-faded")]
    pub cross_faded: CrossFaded,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms. It can be represented using a property expression.
    #[serde(rename = "cross-faded-data-driven")]
    pub cross_faded_data_driven: CrossFadedDataDriven,
    /// Property is interpolable but cannot be represented using a property expression.
    #[serde(rename = "data-constant")]
    pub data_constant: DataConstant,
    /// Property is interpolable and can be represented using a property expression.
    #[serde(rename = "data-driven")]
    pub data_driven: DataDriven,
}

/// Property should be specified using a color ramp from which the output color can be sampled based on a property calculation.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct ColorRamp(serde_json::Value);

/// Property is constant across all zoom levels and property values.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Constant(serde_json::Value);

/// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct CrossFaded(serde_json::Value);

/// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms. It can be represented using a property expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct CrossFadedDataDriven(serde_json::Value);

/// Property is interpolable but cannot be represented using a property expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct DataConstant(serde_json::Value);

/// Property is interpolable and can be represented using a property expression.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct DataDriven(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Sky {
    /// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
    #[serde(rename = "atmosphere-blend")]
    pub atmosphere_blend: AtmosphereBlend,
    /// The base color for the fog. Requires 3D terrain.
    #[serde(rename = "fog-color")]
    pub fog_color: FogColor,
    /// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
    #[serde(rename = "fog-ground-blend")]
    pub fog_ground_blend: FogGroundBlend,
    /// The base color at the horizon.
    #[serde(rename = "horizon-color")]
    pub horizon_color: HorizonColor,
    /// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
    #[serde(rename = "horizon-fog-blend")]
    pub horizon_fog_blend: HorizonFogBlend,
    /// The base color for the sky.
    #[serde(rename = "sky-color")]
    pub sky_color: SkyColor,
    /// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
    #[serde(rename = "sky-horizon-blend")]
    pub sky_horizon_blend: SkyHorizonBlend,
}

/// How to blend the atmosphere. Where 1 is visible atmosphere and 0 is hidden. It is best to interpolate this expression when using globe projection.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct AtmosphereBlend(serde_json::Number);

impl Default for AtmosphereBlend {
    fn default() -> Self {
        Self(0.8.into())
    }
}

/// The base color for the fog. Requires 3D terrain.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct FogColor(color::DynamicColor);

impl Default for FogColor {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the fog over the 3D terrain. Where 0 is the map center and 1 is the horizon.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct FogGroundBlend(serde_json::Number);

impl Default for FogGroundBlend {
    fn default() -> Self {
        Self(0.5.into())
    }
}

/// The base color at the horizon.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct HorizonColor(color::DynamicColor);

impl Default for HorizonColor {
    fn default() -> Self {
        Self(color::parse_color("#ffffff").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the fog color and the horizon color. Where 0 is using the horizon color only and 1 is using the fog color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct HorizonFogBlend(serde_json::Number);

impl Default for HorizonFogBlend {
    fn default() -> Self {
        Self(0.8.into())
    }
}

/// The base color for the sky.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
struct SkyColor(color::DynamicColor);

impl Default for SkyColor {
    fn default() -> Self {
        Self(color::parse_color("#88C6FC").expect("Invalid color specified as the default value"))
    }
}

/// How to blend the sky color and the horizon color. Where 1 is blending the color at the middle of the sky and 0 is not blending at all and using the sky color only.
///
/// Range: 0..=1
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SkyHorizonBlend(serde_json::Number);

impl Default for SkyHorizonBlend {
    fn default() -> Self {
        Self(0.8.into())
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum Source {
    SourceVector(SourceVector),
    SourceRaster(SourceRaster),
    SourceRasterDem(SourceRasterDem),
    SourceGeojson(SourceGeojson),
    SourceVideo(SourceVideo),
    SourceImage(SourceImage),
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceGeojson {
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(rename = "attribution")]
    pub attribution: Attribution,
    /// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
    #[serde(rename = "buffer")]
    pub buffer: Buffer,
    /// If the data is a collection of point features, setting this to true clusters the points by radius into groups. Cluster groups become new `Point` features in the source with additional properties:
    ///
    ///  * `cluster` Is `true` if the point is a cluster
    ///
    ///  * `cluster_id` A unique id for the cluster to be used in conjunction with the [cluster inspection methods](https://maplibre.org/maplibre-gl-js/docs/API/classes/GeoJSONSource/#getclusterexpansionzoom)
    ///
    ///  * `point_count` Number of original points grouped into this cluster
    ///
    ///  * `point_count_abbreviated` An abbreviated point count
    #[serde(rename = "cluster")]
    pub cluster: Cluster,
    /// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
    #[serde(rename = "clusterMaxZoom")]
    pub cluster_max_zoom: Clustermaxzoom,
    /// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
    #[serde(rename = "clusterMinPoints")]
    pub cluster_min_points: Clusterminpoints,
    /// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
    ///
    /// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
    ///
    /// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
    ///
    /// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
    #[serde(rename = "clusterProperties")]
    pub cluster_properties: Clusterproperties,
    /// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
    #[serde(rename = "clusterRadius")]
    pub cluster_radius: Clusterradius,
    /// A URL to a GeoJSON file, or inline GeoJSON.
    #[serde(rename = "data")]
    pub data: Data,
    /// An expression for filtering features prior to processing them for rendering.
    #[serde(rename = "filter")]
    pub filter: Filter,
    /// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
    #[serde(rename = "generateId")]
    pub generate_id: Generateid,
    /// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
    #[serde(rename = "lineMetrics")]
    pub line_metrics: Linemetrics,
    /// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
    #[serde(rename = "maxzoom")]
    pub maxzoom: Maxzoom,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
    #[serde(rename = "promoteId")]
    pub promote_id: Promoteid,
    /// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
    #[serde(rename = "tolerance")]
    pub tolerance: Tolerance,
    /// The data type of the GeoJSON source.
    #[serde(rename = "type")]
    pub r#type: Type,
}

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Attribution(serde_json::Value);

/// Size of the tile buffer on each side. A value of 0 produces no buffer. A value of 512 produces a buffer as wide as the tile itself. Larger values produce fewer rendering artifacts near tile edges and slower performance.
///
/// Range: 0..=512
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Buffer(serde_json::Number);

impl Default for Buffer {
    fn default() -> Self {
        Self(128.into())
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
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Cluster(serde_json::Value);

impl Default for Cluster {
    fn default() -> Self {
        false
    }
}

/// Max zoom on which to cluster points if clustering is enabled. Defaults to one zoom less than maxzoom (so that last zoom features are not clustered). Clusters are re-evaluated at integer zoom levels so setting clusterMaxZoom to 14 means the clusters will be displayed until z15.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Clustermaxzoom(serde_json::Number);

/// Minimum number of points necessary to form a cluster if clustering is enabled. Defaults to `2`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Clusterminpoints(serde_json::Number);

/// An object defining custom properties on the generated clusters if clustering is enabled, aggregating values from clustered points. Has the form `{"property_name": [operator, map_expression]}`. `operator` is any expression function that accepts at least 2 operands (e.g. `"+"` or `"max"`) — it accumulates the property value from clusters/points the cluster contains; `map_expression` produces the value of a single point.
///
/// Example: `{"sum": ["+", ["get", "scalerank"]]}`.
///
/// For more advanced use cases, in place of `operator`, you can use a custom reduce expression that references a special `["accumulated"]` value, e.g.:
///
/// `{"sum": [["+", ["accumulated"], ["get", "sum"]], ["get", "scalerank"]]}`
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Clusterproperties(serde_json::Value);

/// Radius of each cluster if clustering is enabled. A value of 512 indicates a radius equal to the width of a tile.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Clusterradius(serde_json::Number);

impl Default for Clusterradius {
    fn default() -> Self {
        Self(50.into())
    }
}

/// A URL to a GeoJSON file, or inline GeoJSON.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Data(serde_json::Value);

/// An expression for filtering features prior to processing them for rendering.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Filter(serde_json::Value);

/// Whether to generate ids for the geojson features. When enabled, the `feature.id` property will be auto assigned based on its index in the `features` array, over-writing any previous values.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Generateid(serde_json::Value);

impl Default for Generateid {
    fn default() -> Self {
        false
    }
}

/// Whether to calculate line distance metrics. This is required for line layers that specify `line-gradient` values.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Linemetrics(serde_json::Value);

impl Default for Linemetrics {
    fn default() -> Self {
        false
    }
}

/// Maximum zoom level at which to create vector tiles (higher means greater detail at high zoom levels).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Maxzoom(serde_json::Number);

impl Default for Maxzoom {
    fn default() -> Self {
        Self(18.into())
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Promoteid(serde_json::Value);

/// Douglas-Peucker simplification tolerance (higher means simpler geometries and faster performance).
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Tolerance(serde_json::Number);

impl Default for Tolerance {
    fn default() -> Self {
        Self(0.375.into())
    }
}

/// The data type of the GeoJSON source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// A GeoJSON data source.
    #[serde(rename = "geojson")]
    Geojson,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceImage {
    /// Corners of image specified in longitude, latitude pairs.
    #[serde(rename = "coordinates")]
    pub coordinates: Coordinates,
    /// The data type of the image source.
    #[serde(rename = "type")]
    pub r#type: Type,
    /// URL that points to an image.
    #[serde(rename = "url")]
    pub url: Url,
}

/// Corners of image specified in longitude, latitude pairs.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Coordinates(serde_json::Value);

/// The data type of the image source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// An image data source.
    #[serde(rename = "image")]
    Image,
}

/// URL that points to an image.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Url(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRaster {
    /// Other keys to configure the data source.
    #[serde(rename = "*")]
    pub star: Star,
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(rename = "attribution")]
    pub attribution: Attribution,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(rename = "bounds")]
    pub bounds: Bounds,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(rename = "maxzoom")]
    pub maxzoom: Maxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(rename = "minzoom")]
    pub minzoom: Minzoom,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    #[serde(rename = "scheme")]
    pub scheme: Scheme,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: Tilesize,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(rename = "tiles")]
    pub tiles: Tiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: Type,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(rename = "url")]
    pub url: Url,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(rename = "volatile")]
    pub volatile: Volatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Star(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Attribution(serde_json::Value);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Bounds(serde_json::Value);

impl Default for Bounds {
    fn default() -> Self {
        vec![-180, -85.051129, 180, 85.051129]
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Maxzoom(serde_json::Number);

impl Default for Maxzoom {
    fn default() -> Self {
        Self(22.into())
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Minzoom(serde_json::Number);

impl Default for Minzoom {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Scheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    Xyz,
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Xyz
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Tilesize(serde_json::Number);

impl Default for Tilesize {
    fn default() -> Self {
        Self(512.into())
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Tiles(serde_json::Value);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// A raster tile source.
    #[serde(rename = "raster")]
    Raster,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Url(serde_json::Value);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Volatile(serde_json::Value);

impl Default for Volatile {
    fn default() -> Self {
        false
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceRasterDem {
    /// Other keys to configure the data source.
    #[serde(rename = "*")]
    pub star: Star,
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(rename = "attribution")]
    pub attribution: Attribution,
    /// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
    #[serde(rename = "baseShift")]
    pub base_shift: Baseshift,
    /// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
    #[serde(rename = "blueFactor")]
    pub blue_factor: Bluefactor,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(rename = "bounds")]
    pub bounds: Bounds,
    /// The encoding used by this source. Mapbox Terrain RGB is used by default.
    #[serde(rename = "encoding")]
    pub encoding: Encoding,
    /// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
    #[serde(rename = "greenFactor")]
    pub green_factor: Greenfactor,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(rename = "maxzoom")]
    pub maxzoom: Maxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(rename = "minzoom")]
    pub minzoom: Minzoom,
    /// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
    #[serde(rename = "redFactor")]
    pub red_factor: Redfactor,
    /// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
    #[serde(rename = "tileSize")]
    pub tile_size: Tilesize,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(rename = "tiles")]
    pub tiles: Tiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: Type,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(rename = "url")]
    pub url: Url,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(rename = "volatile")]
    pub volatile: Volatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Star(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Attribution(serde_json::Value);

/// Value that will be added to the encoding mix when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Baseshift(serde_json::Number);

impl Default for Baseshift {
    fn default() -> Self {
        Self(0.0.into())
    }
}

/// Value that will be multiplied by the blue channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Bluefactor(serde_json::Number);

impl Default for Bluefactor {
    fn default() -> Self {
        Self(1.0.into())
    }
}

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Bounds(serde_json::Value);

impl Default for Bounds {
    fn default() -> Self {
        vec![-180, -85.051129, 180, 85.051129]
    }
}

/// The encoding used by this source. Mapbox Terrain RGB is used by default.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Encoding {
    /// Decodes tiles using the redFactor, blueFactor, greenFactor, baseShift parameters.
    #[serde(rename = "custom")]
    Custom,
    /// Mapbox Terrain RGB tiles. See https://www.mapbox.com/help/access-elevation-data/#mapbox-terrain-rgb for more info.
    #[serde(rename = "mapbox")]
    Mapbox,
    /// Terrarium format PNG tiles. See https://aws.amazon.com/es/public-datasets/terrain/ for more info.
    #[serde(rename = "terrarium")]
    Terrarium,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Mapbox
    }
}

/// Value that will be multiplied by the green channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Greenfactor(serde_json::Number);

impl Default for Greenfactor {
    fn default() -> Self {
        Self(1.0.into())
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Maxzoom(serde_json::Number);

impl Default for Maxzoom {
    fn default() -> Self {
        Self(22.into())
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Minzoom(serde_json::Number);

impl Default for Minzoom {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Value that will be multiplied by the red channel value when decoding. Only used on custom encodings.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Redfactor(serde_json::Number);

impl Default for Redfactor {
    fn default() -> Self {
        Self(1.0.into())
    }
}

/// The minimum visual size to display tiles for this layer. Only configurable for raster layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Tilesize(serde_json::Number);

impl Default for Tilesize {
    fn default() -> Self {
        Self(512.into())
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Tiles(serde_json::Value);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// A RGB-encoded raster DEM source
    #[serde(rename = "raster-dem")]
    RasterDem,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Url(serde_json::Value);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Volatile(serde_json::Value);

impl Default for Volatile {
    fn default() -> Self {
        false
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVector {
    /// Other keys to configure the data source.
    #[serde(rename = "*")]
    pub star: Star,
    /// Contains an attribution to be displayed when the map is shown to a user.
    #[serde(rename = "attribution")]
    pub attribution: Attribution,
    /// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
    #[serde(rename = "bounds")]
    pub bounds: Bounds,
    /// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
    #[serde(rename = "encoding")]
    pub encoding: Encoding,
    /// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
    #[serde(rename = "maxzoom")]
    pub maxzoom: Maxzoom,
    /// Minimum zoom level for which tiles are available, as in the TileJSON spec.
    #[serde(rename = "minzoom")]
    pub minzoom: Minzoom,
    /// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
    #[serde(rename = "promoteId")]
    pub promote_id: Promoteid,
    /// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
    #[serde(rename = "scheme")]
    pub scheme: Scheme,
    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(rename = "tiles")]
    pub tiles: Tiles,
    /// The type of the source.
    #[serde(rename = "type")]
    pub r#type: Type,
    /// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
    #[serde(rename = "url")]
    pub url: Url,
    /// A setting to determine whether a source's tiles are cached locally.
    #[serde(rename = "volatile")]
    pub volatile: Volatile,
}

/// Other keys to configure the data source.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Star(serde_json::Value);

/// Contains an attribution to be displayed when the map is shown to a user.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Attribution(serde_json::Value);

/// An array containing the longitude and latitude of the southwest and northeast corners of the source's bounding box in the following order: `[sw.lng, sw.lat, ne.lng, ne.lat]`. When this property is included in a source, no tiles outside of the given bounds are requested by MapLibre.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Bounds(serde_json::Value);

impl Default for Bounds {
    fn default() -> Self {
        vec![-180, -85.051129, 180, 85.051129]
    }
}

/// The encoding used by this source. Mapbox Vector Tiles encoding is used by default.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Encoding {
    /// MapLibre Vector Tiles. See https://github.com/maplibre/maplibre-tile-spec for more info.
    #[serde(rename = "mlt")]
    Mlt,
    /// Mapbox Vector Tiles. See http://github.com/mapbox/vector-tile-spec for more info.
    #[serde(rename = "mvt")]
    Mvt,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Mvt
    }
}

/// Maximum zoom level for which tiles are available, as in the TileJSON spec. Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Maxzoom(serde_json::Number);

impl Default for Maxzoom {
    fn default() -> Self {
        Self(22.into())
    }
}

/// Minimum zoom level for which tiles are available, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Minzoom(serde_json::Number);

impl Default for Minzoom {
    fn default() -> Self {
        Self(0.into())
    }
}

/// A property to use as a feature id (for feature state). Either a property name, or an object of the form `{<sourceLayer>: <propertyName>}`. If specified as a string for a vector tile source, the same property is used across all its source layers.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Promoteid(serde_json::Value);

/// Influences the y direction of the tile coordinates. The global-mercator (aka Spherical Mercator) profile is assumed.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Scheme {
    /// OSGeo spec scheme.
    #[serde(rename = "tms")]
    Tms,
    /// Slippy map tilenames scheme.
    #[serde(rename = "xyz")]
    Xyz,
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Xyz
    }
}

/// An array of one or more tile source URLs, as in the TileJSON spec.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Tiles(serde_json::Value);

/// The type of the source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// A vector tile source.
    #[serde(rename = "vector")]
    Vector,
}

/// A URL to a TileJSON resource. Supported protocols are `http:` and `https:`.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Url(serde_json::Value);

/// A setting to determine whether a source's tiles are cached locally.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Volatile(serde_json::Value);

impl Default for Volatile {
    fn default() -> Self {
        false
    }
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct SourceVideo {
    /// Corners of video specified in longitude, latitude pairs.
    #[serde(rename = "coordinates")]
    pub coordinates: Coordinates,
    /// The data type of the video source.
    #[serde(rename = "type")]
    pub r#type: Type,
    /// URLs to video content in order of preferred format.
    #[serde(rename = "urls")]
    pub urls: Urls,
}

/// Corners of video specified in longitude, latitude pairs.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Coordinates(serde_json::Value);

/// The data type of the video source.
#[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    /// A video data source.
    #[serde(rename = "video")]
    Video,
}

/// URLs to video content in order of preferred format.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Urls(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Sources {
    /// Specification of a data source. For vector and raster sources, either TileJSON or a URL to a TileJSON must be provided. For image and video sources, a URL must be provided. For GeoJSON sources, a URL or inline GeoJSON must be provided.
    #[serde(rename = "*")]
    pub star: Star,
}

/// Specification of a data source. For vector and raster sources, either TileJSON or a URL to a TileJSON must be provided. For image and video sources, a URL must be provided. For GeoJSON sources, a URL or inline GeoJSON must be provided.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Star(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Terrain {
    /// The exaggeration of the terrain - how high it will look.
    #[serde(rename = "exaggeration")]
    pub exaggeration: Exaggeration,
    /// The source for the terrain data.
    #[serde(rename = "source")]
    pub source: Source,
}

/// The exaggeration of the terrain - how high it will look.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Exaggeration(serde_json::Number);

impl Default for Exaggeration {
    fn default() -> Self {
        Self(1.0.into())
    }
}

/// The source for the terrain data.
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[deprecated = "not_implemented"]
struct Source(serde_json::Value);

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Transition {
    /// Length of time before a transition begins.
    #[serde(rename = "delay")]
    pub delay: Delay,
    /// Time allotted for transitions to complete.
    #[serde(rename = "duration")]
    pub duration: Duration,
}

/// Length of time before a transition begins.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Delay(serde_json::Number);

impl Default for Delay {
    fn default() -> Self {
        Self(0.into())
    }
}

/// Time allotted for transitions to complete.
///
/// Range: 0..
#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub struct Duration(serde_json::Number);

impl Default for Duration {
    fn default() -> Self {
        Self(300.into())
    }
}
