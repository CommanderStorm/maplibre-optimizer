//! `#[cfg_attr(feature = "fuzz", …)]` fragments for generated `spec` types.

pub const CFG_DERIVE_ARBITRARY: &str = "cfg_attr(feature = \"fuzz\", derive(arbitrary::Arbitrary))";

pub const ARB_JSON_NUMBER: &str =
    "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]";
pub const ARB_JSON_VALUE: &str =
    "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]";
pub const ARB_OPTION_JSON_VALUE: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_value))]";
pub const ARB_VEC_JSON_VALUE: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]";
pub const ARB_VEC_JSON_NUMBER: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_number))]";
pub const ARB_BOX_1_JSON_NUMBER: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_box_1_json_number))]";
pub const ARB_BOX_2_JSON_NUMBER: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_box_2_json_number))]";
pub const ARB_BOX_3_JSON_NUMBER: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_box_3_json_number))]";
pub const ARB_BOX_4_JSON_NUMBER: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_box_4_json_number))]";
pub const ARB_JSON_MAP: &str =
    "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_json_map))]";
pub const ARB_OPTION_JSON_MAP: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_option_json_map))]";
pub const ARB_URL: &str =
    "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_url))]";
pub const ARB_DYNAMIC_COLOR: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_dynamic_color))]";
pub const ARB_VEC_DYNAMIC_COLOR: &str = "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_dynamic_color))]";
pub const ARB_GEOJSON: &str =
    "#[cfg_attr(feature = \"fuzz\", arbitrary(with = crate::fuzz_helpers::arbitrary_geojson))]";

pub const JSON_MAP_TY: &str = "serde_json::Map<std::string::String, serde_json::Value>";
pub const OPTION_JSON_MAP_TY: &str =
    "Option<serde_json::Map<std::string::String, serde_json::Value>>";
