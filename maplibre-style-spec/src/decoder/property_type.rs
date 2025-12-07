#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyType {
    /// Property should be specified using a color ramp from which the output color can be sampled based on a property calculation.
    ColorRamp,
    /// Property is constant across all zoom levels and property values.
    Constant,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms.
    CrossFaded,
    /// Property is non-interpolable; rather, its values will be cross-faded to smoothly transition between integer zooms. It can be represented using a property expression.
    CrossFadedDataDriven,
    /// Property is interpolable but cannot be represented using a property expression.
    DataConstant,
    /// Property is interpolable and can be represented using a property expression.
    DataDriven,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::Value;

    use super::*;

    // test that all property types that can be parsed from the spec are parseable
    #[test]
    fn decode_property_types() {
        let content = include_str!("../../../upstream/src/reference/v8.json");
        let top: HashMap<String, Value> = serde_json::from_str(content).unwrap();
        let property_type = top.get("property-type").unwrap().as_object().unwrap();
        for k in property_type.keys() {
            let _: PropertyType = serde_json::from_str(&format!("\"{k}\"")).unwrap();
        }
    }
}