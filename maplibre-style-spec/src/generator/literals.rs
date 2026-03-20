use codegen2::Scope;

/// Newtypes and aliases referenced from generated expression-syntax tuples (`string literal`, etc.).
/// Emitted before syntax enums so parameter types resolve.
pub fn generate_literals(scope: &mut Scope) {
    scope
        .new_struct("NumberLiteral")
        .vis("pub")
        .doc("JSON number in an expression position")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Number");

    scope
        .new_struct("StringLiteral")
        .vis("pub")
        .doc("JSON string in an expression position")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("std::string::String");

    scope
        .new_struct("GeoJSONObjectLiteral")
        .vis("pub")
        .doc("GeoJSON object literal")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("geojson::GeoJson");

    scope
        .new_struct("JSONObjectLiteral")
        .vis("pub")
        .doc("JSON object literal")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");

    scope
        .new_struct("JSONArrayLiteral")
        .vis("pub")
        .doc("JSON array literal")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("Vec<serde_json::Value>");

    scope
        .new_struct("ArrayOfStringLiteral")
        .vis("pub")
        .doc("Array whose elements are string literals (e.g. match labels)")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("Vec<StringLiteral>");

    scope
        .new_struct("ArrayOfNumberLiteral")
        .vis("pub")
        .doc("Array whose elements are number literals (e.g. match labels)")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .tuple_field("Vec<NumberLiteral>");
}
