use codegen2::Scope;

use crate::generator::fuzz;

/// Newtypes and aliases referenced from generated expression-syntax tuples (`string literal`, etc.).
/// Emitted before syntax enums so parameter types resolve.
pub fn generate_literals(scope: &mut Scope) {
    scope
        .new_struct("NumberLiteral")
        .vis("pub")
        .doc("JSON number in an expression position")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field_with_attrs([fuzz::ARB_JSON_NUMBER], "serde_json::Number");

    scope
        .new_struct("StringLiteral")
        .vis("pub")
        .doc("JSON string in an expression position")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field("std::string::String");

    scope.raw(
        r"impl From<serde_json::Number> for NumberLiteral {
    fn from(n: serde_json::Number) -> Self {
        Self(n)
    }
}",
    );

    scope.raw(
        r"impl From<std::string::String> for StringLiteral {
    fn from(s: std::string::String) -> Self {
        Self(s)
    }
}",
    );

    scope
        .new_struct("GeoJSONObjectLiteral")
        .vis("pub")
        .doc("GeoJSON object literal")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field_with_attrs([fuzz::ARB_GEOJSON], "geojson::GeoJson");

    // JSONObjectLiteral and JSONArrayLiteral have pub fields so that
    // ExprOrLiteral::normalize() can destructure and reconstruct them.
    scope.raw(
        r#"/// JSON object literal
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct JSONObjectLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
    pub serde_json::Value,
);

/// JSON array literal
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub struct JSONArrayLiteral(
    #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
    pub Vec<serde_json::Value>,
);
"#,
    );

    scope
        .new_struct("ArrayOfStringLiteral")
        .vis("pub")
        .doc("Array whose elements are string literals (e.g. match labels)")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field("Vec<StringLiteral>");

    scope
        .new_struct("ArrayOfNumberLiteral")
        .vis("pub")
        .doc("Array whose elements are number literals (e.g. match labels)")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .tuple_field("Vec<NumberLiteral>");
}
