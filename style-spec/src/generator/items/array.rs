use codegen2::Scope;
use serde_json::Value;

use super::escape_doc_for_macro;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::generator::fuzz;
use crate::generator::items::number::generate_number_default;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::{MirArrayElement, MirArrayField, MirRegularEnum};

pub fn generate(scope: &mut Scope, name: &str, field: &MirArrayField) {
    if field.meta.expression.is_some() {
        let doc = escape_doc_for_macro(&field.meta.doc);
        let mut args = format!("{name}, doc = \"{doc}\"");
        if let Some(default) = &field.default {
            let default_json = serde_json::to_string(default).expect("default should serialize");
            args.push_str(&format!(", default = serde_json::json!({default_json})"));
        }
        scope.raw(format!("array_prop!({args});"));
        generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
        return;
    }

    let element_type_name = to_upper_camel_case(format!("{name} Value"));
    let rust_element_type = generate_array_element(scope, &element_type_name, &field.element);

    let field_type = if is_direct_element(&field.element) {
        rust_element_type
    } else if let Some(length) = field.length {
        format!("Box<[{rust_element_type}; {length}]>")
    } else {
        format!("Vec<{rust_element_type}>")
    };

    let st = scope
        .new_struct(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    match (&field.element, field.length) {
        (MirArrayElement::Star, _) => {
            st.tuple_field_with_attrs([fuzz::ARB_VEC_JSON_VALUE], field_type);
        }
        (MirArrayElement::Number { .. }, None) => {
            st.tuple_field_with_attrs([fuzz::ARB_VEC_JSON_NUMBER], field_type);
        }
        (MirArrayElement::Number { .. }, Some(1)) => {
            st.tuple_field_with_attrs([fuzz::ARB_BOX_1_JSON_NUMBER], field_type);
        }
        (MirArrayElement::Number { .. }, Some(2)) => {
            st.tuple_field_with_attrs([fuzz::ARB_BOX_2_JSON_NUMBER], field_type);
        }
        (MirArrayElement::Number { .. }, Some(3)) => {
            st.tuple_field_with_attrs([fuzz::ARB_BOX_3_JSON_NUMBER], field_type);
        }
        (MirArrayElement::Number { .. }, Some(4)) => {
            st.tuple_field_with_attrs([fuzz::ARB_BOX_4_JSON_NUMBER], field_type);
        }
        (MirArrayElement::Number { .. }, Some(_)) => {
            st.tuple_field(field_type);
        }
        (MirArrayElement::Color, None) => {
            st.tuple_field_with_attrs([fuzz::ARB_VEC_DYNAMIC_COLOR], field_type);
        }
        _ => {
            st.tuple_field(field_type);
        }
    }

    if let Some(default) = &field.default {
        let mut default_expr = String::new();
        generate_value_array_default(&mut default_expr, default, field.length.as_ref());
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self({default_expr})"));
    }
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

/// Whether this element type should be used directly as the field type
/// rather than being wrapped in `Vec<...>`.
fn is_direct_element(element: &MirArrayElement) -> bool {
    matches!(
        element,
        MirArrayElement::FontFaces
            | MirArrayElement::ExpressionName
            | MirArrayElement::InterpolationName
    )
}

/// Returns the Rust type name for the array element, generating any necessary
/// helper types into `scope`.
fn generate_array_element(scope: &mut Scope, name: &str, element: &MirArrayElement) -> String {
    match element {
        MirArrayElement::String => "std::string::String".to_string(),
        MirArrayElement::Number { .. } => "serde_json::Number".to_string(),
        MirArrayElement::Boolean => "bool".to_string(),
        MirArrayElement::Color => "color::DynamicColor".to_string(),
        MirArrayElement::Star => "serde_json::Value".to_string(),
        MirArrayElement::Layer => "Layer".to_string(),
        MirArrayElement::FunctionStop => "FunctionStop".to_string(),
        MirArrayElement::ExpressionName => "ExpressionName".to_string(),
        MirArrayElement::InterpolationName => "InterpolationName".to_string(),

        MirArrayElement::Enum(r) => {
            generate_inline_enum(scope, name, r);
            name.to_string()
        }

        MirArrayElement::FontFaces => {
            generate_font_faces(scope);
            "std::collections::BTreeMap<std::string::String,FontFace>".to_string()
        }

        MirArrayElement::Either(options) => {
            let mut variant_types = Vec::with_capacity(options.len());
            for (i, option) in options.iter().enumerate() {
                let enum_variant_name = to_upper_camel_case(i.to_string());
                let variant_type_name = to_upper_camel_case(format!("{name} {enum_variant_name}"));
                variant_types.push((
                    enum_variant_name,
                    generate_array_element(scope, &variant_type_name, option),
                ));
            }

            let enu = scope
                .new_enum(name)
                .doc(format!("{name} Values"))
                .derive("PartialEq, Debug, Clone")
                .attr(fuzz::CFG_DERIVE_ARBITRARY)
                .vis("pub");
            for (variant_name, variant_type) in &variant_types {
                let v = enu.new_variant(variant_name);
                match variant_type.as_str() {
                    "serde_json::Number" => {
                        v.tuple_with_attrs([fuzz::ARB_JSON_NUMBER], variant_type);
                    }
                    "color::DynamicColor" => {
                        v.tuple_with_attrs([fuzz::ARB_DYNAMIC_COLOR], variant_type);
                    }
                    _ => {
                        v.tuple(variant_type);
                    }
                }
            }
            let variants: Vec<Variant> = variant_types
                .iter()
                .map(|(vn, vt)| Variant {
                    name: vn.clone(),
                    inner_type: vt.clone(),
                    is_boxed: false,
                    is_unit: false,
                    skip_when: None,
                })
                .collect();
            untagged::emit_untagged_serde(scope, name, &variants);
            name.to_string()
        }

        MirArrayElement::Complex(inner_field) => {
            // Delegate to the central MIR dispatch in the parent generator module.
            crate::generator::generate_mir_type(scope, name, inner_field);
            name.to_string()
        }
    }
}

fn generate_inline_enum(scope: &mut Scope, name: &str, r: &MirRegularEnum) {
    crate::generator::items::r#enum::generate_regular(scope, name, "", r, None);
}

fn generate_font_faces(scope: &mut Scope) {
    let font_with_range = scope
        .new_struct("FontWithRange")
        .vis("pub")
        .doc("Font file URL and the unicode-range at which it can be used")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    font_with_range
        .new_field("url", "url::Url")
        .vis("pub")
        .doc("URL the font can retrieved under")
        .annotation(fuzz::ARB_URL);
    font_with_range
        .new_field("unicode_range", "Vec<std::string::String>")
        .vis("pub")
        .doc("Unicode range(s) where this font applies (CSS `unicode-range` semantics)")
        .annotation("#[serde(rename=\"unicode-range\")]")
        .annotation("#[serde(default)]")
        .annotation("#[serde(skip_serializing_if = \"Vec::is_empty\")]");

    let enu = scope
        .new_enum("FontFace")
        .vis("pub")
        .derive("PartialEq, Eq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("Url")
        .doc("A single global font file URL")
        .tuple_with_attrs([fuzz::ARB_URL], "url::Url");
    enu.new_variant("FontRange")
        .doc("Load different fonts depending on the unicode range")
        .tuple("Vec<FontWithRange>");
    untagged::emit_untagged_serde(
        scope,
        "FontFace",
        &[
            Variant {
                name: "Url".into(),
                inner_type: "url::Url".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
            Variant {
                name: "FontRange".into(),
                inner_type: "Vec<FontWithRange>".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
        ],
    );
}

fn generate_value_array_default(buffer: &mut String, items: &[Value], length: Option<&usize>) {
    if length.is_some() {
        buffer.push_str("Box::new([");
    } else {
        buffer.push_str("Vec::from([");
    }
    let mut needs_separator = false;
    for item in items {
        if needs_separator {
            buffer.push_str(", ");
        }
        generate_value_default(buffer, item);
        needs_separator = true;
    }
    buffer.push_str("])");
}

fn generate_value_default(buffer: &mut String, item: &Value) {
    match item {
        Value::Null => buffer.push_str("None"),
        Value::Bool(b) => buffer.push_str(&b.to_string()),
        Value::Number(n) => buffer.push_str(&generate_number_default(n)),
        Value::String(s) => {
            buffer.push('"');
            buffer.push_str(s);
            buffer.push_str("\".to_string()");
        }
        Value::Array(a) => generate_value_array_default(buffer, a, None),
        Value::Object(o) => {
            let json = serde_json::to_string(o).expect("serializing json object must succeed");
            buffer.push_str(&format!(
                "serde_json::from_str::<serde_json::Value>({json:?}).expect(\"object default must be valid json\")"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::StyleReference;
    use crate::mir::types::MirFieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &MirArrayField {
                meta: MirFieldMeta::default(),
                default: None,
                element: MirArrayElement::Star,
                length: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Foo(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
            Vec<serde_json::Value>,
        );
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = serde_json::json!({
        "$version": 8,
        "$root": {},
        "position": {
            "type": "array",
            "default": [
                1.15,
                210,
                30
            ],
            "length": 3,
            "value": "number",
            "property-type": "data-constant",
            "transition": true,
            "expression": {
                "interpolated": true,
                "parameters": [
                  "zoom"
                ]
            },
            "doc": "Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).",
            "example": [
                1.5,
                90,
                80
            ],
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::MirSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r#"
        /// JSON number in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct NumberLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );

        /// JSON string in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct StringLiteral(std::string::String);

        impl From<serde_json::Number> for NumberLiteral {
            fn from(n: serde_json::Number) -> Self {
                Self(n)
            }
        }

        impl From<std::string::String> for StringLiteral {
            fn from(s: std::string::String) -> Self {
                Self(s)
            }
        }

        /// GeoJSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct GeoJSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_geojson))]
            geojson::GeoJson,
        );

        /// JSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
            pub  serde_json::Value,
        );

        /// JSON array literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONArrayLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
            pub Vec<serde_json::Value>,
        );

        /// Array whose elements are string literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfStringLiteral(Vec<StringLiteral>);

        /// Array whose elements are number literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfNumberLiteral(Vec<NumberLiteral>);

        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct MaplibreStyleSpecification;

        array_prop!(
            Position,
            doc = "Position of the light source relative to lit (extruded) geometries, in [r radial coordinate, a azimuthal angle, p polar angle] where r indicates the distance from the center of the base of an object to its light, a indicates the position of the light relative to 0° (0° when `light.anchor` is set to `viewport` corresponds to the top of the viewport, or 0° when `light.anchor` is set to `map` corresponds to due north, and degrees proceed clockwise), and p indicates the height of the light (from 0°, directly above, to 180°, directly below).",
            default = serde_json::json!([1.15, 210, 30])
        );

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_position_decodes() {
                let example = serde_json::json!([1.5, 90, 80]);
                let _ = serde_json::from_value::<Position>(example).expect("example should decode");
            }
        }

        /// An expression node or a literal JSON value in expression positions.
        #[derive(PartialEq, Debug, Clone)]
        pub enum ExprOrLiteral {
            Null,
            Bool(bool),
            NumberLiteral(NumberLiteral),
            StringLiteral(StringLiteral),
            GeoJSONObjectLiteral(GeoJSONObjectLiteral),
            JSONObjectLiteral(JSONObjectLiteral),
            JSONArrayLiteral(JSONArrayLiteral),
            AnyExpr(Box<Any>),
            ArrayExpr(Box<Array>),
            BooleanExpr(Box<Boolean>),
            CollatorExpr(Box<Collator>),
            ColorExpr(Box<Color>),
            FormattedExpr(Box<Formatted>),
            ImageExpr(Box<Image>),
            NumberExpr(Box<Number>),
            ObjectExpr(Box<Object>),
            StringExpr(Box<String>),
        }

        impl serde::Serialize for ExprOrLiteral {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::Null => serializer.serialize_unit(),
                    Self::Bool(v) => v.serialize(serializer),
                    Self::NumberLiteral(v) => v.serialize(serializer),
                    Self::StringLiteral(v) => v.serialize(serializer),
                    Self::GeoJSONObjectLiteral(v) => v.serialize(serializer),
                    Self::JSONObjectLiteral(v) => v.serialize(serializer),
                    Self::AnyExpr(v) => v.as_ref().serialize(serializer),
                    Self::ArrayExpr(v) => v.as_ref().serialize(serializer),
                    Self::BooleanExpr(v) => v.as_ref().serialize(serializer),
                    Self::CollatorExpr(v) => v.as_ref().serialize(serializer),
                    Self::ColorExpr(v) => v.as_ref().serialize(serializer),
                    Self::FormattedExpr(v) => v.as_ref().serialize(serializer),
                    Self::ImageExpr(v) => v.as_ref().serialize(serializer),
                    Self::NumberExpr(v) => v.as_ref().serialize(serializer),
                    Self::ObjectExpr(v) => v.as_ref().serialize(serializer),
                    Self::StringExpr(v) => v.as_ref().serialize(serializer),
                    Self::JSONArrayLiteral(v) => v.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for ExprOrLiteral {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                if value.is_null() {
                    return Ok(Self::Null);
                }
                match <bool as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Bool(v)),
                    Err(e) => errors.push(("Bool", e.to_string())),
                }
                match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberLiteral(v)),
                    Err(e) => errors.push(("NumberLiteral", e.to_string())),
                }
                match <StringLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringLiteral(v)),
                    Err(e) => errors.push(("StringLiteral", e.to_string())),
                }
                match <GeoJSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::GeoJSONObjectLiteral(v)),
                    Err(e) => errors.push(("GeoJSONObjectLiteral", e.to_string())),
                }
                if !(value.is_array()) {
                    match <JSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                        Ok(v) => return Ok(Self::JSONObjectLiteral(v)),
                        Err(e) => errors.push(("JSONObjectLiteral", e.to_string())),
                    }
                }
                match <Any as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::AnyExpr(Box::new(v))),
                    Err(e) => errors.push(("AnyExpr", e.to_string())),
                }
                match <Array as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ArrayExpr(Box::new(v))),
                    Err(e) => errors.push(("ArrayExpr", e.to_string())),
                }
                match <Boolean as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::BooleanExpr(Box::new(v))),
                    Err(e) => errors.push(("BooleanExpr", e.to_string())),
                }
                match <Collator as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::CollatorExpr(Box::new(v))),
                    Err(e) => errors.push(("CollatorExpr", e.to_string())),
                }
                match <Color as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ColorExpr(Box::new(v))),
                    Err(e) => errors.push(("ColorExpr", e.to_string())),
                }
                match <Formatted as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::FormattedExpr(Box::new(v))),
                    Err(e) => errors.push(("FormattedExpr", e.to_string())),
                }
                match <Image as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ImageExpr(Box::new(v))),
                    Err(e) => errors.push(("ImageExpr", e.to_string())),
                }
                match <Number as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberExpr(Box::new(v))),
                    Err(e) => errors.push(("NumberExpr", e.to_string())),
                }
                match <Object as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ObjectExpr(Box::new(v))),
                    Err(e) => errors.push(("ObjectExpr", e.to_string())),
                }
                match <String as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringExpr(Box::new(v))),
                    Err(e) => errors.push(("StringExpr", e.to_string())),
                }
                match <JSONArrayLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::JSONArrayLiteral(v)),
                    Err(e) => errors.push(("JSONArrayLiteral", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "ExprOrLiteral: no variant matched. Expected Null | Bool(bool) | NumberLiteral(NumberLiteral) | StringLiteral(StringLiteral) | GeoJSONObjectLiteral(GeoJSONObjectLiteral) | JSONObjectLiteral(JSONObjectLiteral) | AnyExpr(Any) | ArrayExpr(Array) | BooleanExpr(Boolean) | CollatorExpr(Collator) | ColorExpr(Color) | FormattedExpr(Formatted) | ImageExpr(Image) | NumberExpr(Number) | ObjectExpr(Object) | StringExpr(String) | JSONArrayLiteral(JSONArrayLiteral). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }

        impl ExprOrLiteral {
            /// Collapse expression-wrapping-literal into the canonical literal variant.
            ///
            /// Expression enums (`Boolean`, `Number`, `String`, `Color`) have `Literal` and
            /// `AnyExpr` variants for top-level use (e.g. filter position). When boxed inside
            /// `ExprOrLiteral`, those overlap with `Bool`, `NumberLiteral`, `StringLiteral`,
            /// and `AnyExpr`. This method normalises to the canonical form.
            #[must_use]
            pub fn normalize(self) -> Self {
                match self {
                    ExprOrLiteral::BooleanExpr(b) => match *b {
                        Boolean::Literal(v) => ExprOrLiteral::Bool(v),
                        Boolean::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::BooleanExpr(Box::new(other)),
                    },
                    ExprOrLiteral::NumberExpr(n) => match *n {
                        Number::Literal(v) => ExprOrLiteral::NumberLiteral(v),
                        Number::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::NumberExpr(Box::new(other)),
                    },
                    ExprOrLiteral::StringExpr(s) => match *s {
                        String::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        String::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::StringExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ColorExpr(c) => match *c {
                        Color::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        Color::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::ColorExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ArrayExpr(a) => match *a {
                        Array::Literal(v) => ExprOrLiteral::JSONArrayLiteral(v),
                        other => ExprOrLiteral::ArrayExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ObjectExpr(o) => match *o {
                        Object::Literal(v) => ExprOrLiteral::JSONObjectLiteral(v),
                        other => ExprOrLiteral::ObjectExpr(Box::new(other)),
                    },
                    // JSONObjectLiteral/JSONArrayLiteral can wrap any serde_json::Value;
                    // normalise primitive contents to the matching literal variant.
                    ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(v)) => match v {
                        serde_json::Value::Null => ExprOrLiteral::Null,
                        serde_json::Value::Bool(b) => ExprOrLiteral::Bool(b),
                        serde_json::Value::Number(n) => {
                            ExprOrLiteral::NumberLiteral(NumberLiteral::from(n))
                        }
                        serde_json::Value::String(s) => {
                            ExprOrLiteral::StringLiteral(StringLiteral::from(s))
                        }
                        other => ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(other)),
                    },
                    other => other,
                }
            }
        }

        #[cfg(feature = "fuzz")]
        impl<'a> arbitrary::Arbitrary<'a> for ExprOrLiteral {
            fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                let tag: u8 = u.arbitrary()?;
                Ok(match tag % 17 {
                    0 => ExprOrLiteral::Null,
                    1 => ExprOrLiteral::Bool(u.arbitrary()?),
                    2 => ExprOrLiteral::NumberLiteral(u.arbitrary()?),
                    3 => ExprOrLiteral::StringLiteral(u.arbitrary()?),
                    4 => ExprOrLiteral::GeoJSONObjectLiteral(u.arbitrary()?),
                    5 => ExprOrLiteral::JSONObjectLiteral(u.arbitrary()?),
                    6 => ExprOrLiteral::JSONArrayLiteral(u.arbitrary()?),
                    7 => ExprOrLiteral::AnyExpr(u.arbitrary()?),
                    8 => ExprOrLiteral::ArrayExpr(u.arbitrary()?),
                    9 => ExprOrLiteral::BooleanExpr(u.arbitrary()?),
                    10 => ExprOrLiteral::CollatorExpr(u.arbitrary()?),
                    11 => ExprOrLiteral::ColorExpr(u.arbitrary()?),
                    12 => ExprOrLiteral::FormattedExpr(u.arbitrary()?),
                    13 => ExprOrLiteral::ImageExpr(u.arbitrary()?),
                    14 => ExprOrLiteral::NumberExpr(u.arbitrary()?),
                    15 => ExprOrLiteral::ObjectExpr(u.arbitrary()?),
                    _ => ExprOrLiteral::StringExpr(u.arbitrary()?),
                }
                .normalize())
            }
        }
        "#);
    }

    #[test]
    fn test_generate_spec_layers() {
        let reference = serde_json::json!({
            "$version": 8,
            "$root": {},
            "layers": {
                "required": true,
                "type": "array",
                "value": "layer",
                "doc": "A style's `layers` property lists all the layers available in that style.",
                "example": [
                    {
                        "id": "coastline",
                        "source": "maplibre",
                        "source-layer": "countries",
                        "type": "line",
                        "paint": {
                            "line-color": "#198EC8"
                        }
                    }
                ]
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::MirSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r##"
        /// JSON number in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct NumberLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );

        /// JSON string in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct StringLiteral(std::string::String);

        impl From<serde_json::Number> for NumberLiteral {
            fn from(n: serde_json::Number) -> Self {
                Self(n)
            }
        }

        impl From<std::string::String> for StringLiteral {
            fn from(s: std::string::String) -> Self {
                Self(s)
            }
        }

        /// GeoJSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct GeoJSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_geojson))]
            geojson::GeoJson,
        );

        /// JSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
            pub  serde_json::Value,
        );

        /// JSON array literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONArrayLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
            pub Vec<serde_json::Value>,
        );

        /// Array whose elements are string literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfStringLiteral(Vec<StringLiteral>);

        /// Array whose elements are number literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfNumberLiteral(Vec<NumberLiteral>);

        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct MaplibreStyleSpecification;

        /// A style's `layers` property lists all the layers available in that style.
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Layers(Vec<Layer>);

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_layers_decodes() {
                let example = serde_json::json!([{"id":"coastline","paint":{"line-color":"#198EC8"},"source":"maplibre","source-layer":"countries","type":"line"}]);
                let _ = serde_json::from_value::<Layers>(example).expect("example should decode");
            }
        }

        /// An expression node or a literal JSON value in expression positions.
        #[derive(PartialEq, Debug, Clone)]
        pub enum ExprOrLiteral {
            Null,
            Bool(bool),
            NumberLiteral(NumberLiteral),
            StringLiteral(StringLiteral),
            GeoJSONObjectLiteral(GeoJSONObjectLiteral),
            JSONObjectLiteral(JSONObjectLiteral),
            JSONArrayLiteral(JSONArrayLiteral),
            AnyExpr(Box<Any>),
            ArrayExpr(Box<Array>),
            BooleanExpr(Box<Boolean>),
            CollatorExpr(Box<Collator>),
            ColorExpr(Box<Color>),
            FormattedExpr(Box<Formatted>),
            ImageExpr(Box<Image>),
            NumberExpr(Box<Number>),
            ObjectExpr(Box<Object>),
            StringExpr(Box<String>),
        }

        impl serde::Serialize for ExprOrLiteral {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::Null => serializer.serialize_unit(),
                    Self::Bool(v) => v.serialize(serializer),
                    Self::NumberLiteral(v) => v.serialize(serializer),
                    Self::StringLiteral(v) => v.serialize(serializer),
                    Self::GeoJSONObjectLiteral(v) => v.serialize(serializer),
                    Self::JSONObjectLiteral(v) => v.serialize(serializer),
                    Self::AnyExpr(v) => v.as_ref().serialize(serializer),
                    Self::ArrayExpr(v) => v.as_ref().serialize(serializer),
                    Self::BooleanExpr(v) => v.as_ref().serialize(serializer),
                    Self::CollatorExpr(v) => v.as_ref().serialize(serializer),
                    Self::ColorExpr(v) => v.as_ref().serialize(serializer),
                    Self::FormattedExpr(v) => v.as_ref().serialize(serializer),
                    Self::ImageExpr(v) => v.as_ref().serialize(serializer),
                    Self::NumberExpr(v) => v.as_ref().serialize(serializer),
                    Self::ObjectExpr(v) => v.as_ref().serialize(serializer),
                    Self::StringExpr(v) => v.as_ref().serialize(serializer),
                    Self::JSONArrayLiteral(v) => v.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for ExprOrLiteral {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                if value.is_null() {
                    return Ok(Self::Null);
                }
                match <bool as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Bool(v)),
                    Err(e) => errors.push(("Bool", e.to_string())),
                }
                match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberLiteral(v)),
                    Err(e) => errors.push(("NumberLiteral", e.to_string())),
                }
                match <StringLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringLiteral(v)),
                    Err(e) => errors.push(("StringLiteral", e.to_string())),
                }
                match <GeoJSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::GeoJSONObjectLiteral(v)),
                    Err(e) => errors.push(("GeoJSONObjectLiteral", e.to_string())),
                }
                if !(value.is_array()) {
                    match <JSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                        Ok(v) => return Ok(Self::JSONObjectLiteral(v)),
                        Err(e) => errors.push(("JSONObjectLiteral", e.to_string())),
                    }
                }
                match <Any as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::AnyExpr(Box::new(v))),
                    Err(e) => errors.push(("AnyExpr", e.to_string())),
                }
                match <Array as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ArrayExpr(Box::new(v))),
                    Err(e) => errors.push(("ArrayExpr", e.to_string())),
                }
                match <Boolean as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::BooleanExpr(Box::new(v))),
                    Err(e) => errors.push(("BooleanExpr", e.to_string())),
                }
                match <Collator as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::CollatorExpr(Box::new(v))),
                    Err(e) => errors.push(("CollatorExpr", e.to_string())),
                }
                match <Color as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ColorExpr(Box::new(v))),
                    Err(e) => errors.push(("ColorExpr", e.to_string())),
                }
                match <Formatted as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::FormattedExpr(Box::new(v))),
                    Err(e) => errors.push(("FormattedExpr", e.to_string())),
                }
                match <Image as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ImageExpr(Box::new(v))),
                    Err(e) => errors.push(("ImageExpr", e.to_string())),
                }
                match <Number as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberExpr(Box::new(v))),
                    Err(e) => errors.push(("NumberExpr", e.to_string())),
                }
                match <Object as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ObjectExpr(Box::new(v))),
                    Err(e) => errors.push(("ObjectExpr", e.to_string())),
                }
                match <String as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringExpr(Box::new(v))),
                    Err(e) => errors.push(("StringExpr", e.to_string())),
                }
                match <JSONArrayLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::JSONArrayLiteral(v)),
                    Err(e) => errors.push(("JSONArrayLiteral", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "ExprOrLiteral: no variant matched. Expected Null | Bool(bool) | NumberLiteral(NumberLiteral) | StringLiteral(StringLiteral) | GeoJSONObjectLiteral(GeoJSONObjectLiteral) | JSONObjectLiteral(JSONObjectLiteral) | AnyExpr(Any) | ArrayExpr(Array) | BooleanExpr(Boolean) | CollatorExpr(Collator) | ColorExpr(Color) | FormattedExpr(Formatted) | ImageExpr(Image) | NumberExpr(Number) | ObjectExpr(Object) | StringExpr(String) | JSONArrayLiteral(JSONArrayLiteral). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }

        impl ExprOrLiteral {
            /// Collapse expression-wrapping-literal into the canonical literal variant.
            ///
            /// Expression enums (`Boolean`, `Number`, `String`, `Color`) have `Literal` and
            /// `AnyExpr` variants for top-level use (e.g. filter position). When boxed inside
            /// `ExprOrLiteral`, those overlap with `Bool`, `NumberLiteral`, `StringLiteral`,
            /// and `AnyExpr`. This method normalises to the canonical form.
            #[must_use]
            pub fn normalize(self) -> Self {
                match self {
                    ExprOrLiteral::BooleanExpr(b) => match *b {
                        Boolean::Literal(v) => ExprOrLiteral::Bool(v),
                        Boolean::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::BooleanExpr(Box::new(other)),
                    },
                    ExprOrLiteral::NumberExpr(n) => match *n {
                        Number::Literal(v) => ExprOrLiteral::NumberLiteral(v),
                        Number::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::NumberExpr(Box::new(other)),
                    },
                    ExprOrLiteral::StringExpr(s) => match *s {
                        String::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        String::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::StringExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ColorExpr(c) => match *c {
                        Color::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        Color::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::ColorExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ArrayExpr(a) => match *a {
                        Array::Literal(v) => ExprOrLiteral::JSONArrayLiteral(v),
                        other => ExprOrLiteral::ArrayExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ObjectExpr(o) => match *o {
                        Object::Literal(v) => ExprOrLiteral::JSONObjectLiteral(v),
                        other => ExprOrLiteral::ObjectExpr(Box::new(other)),
                    },
                    // JSONObjectLiteral/JSONArrayLiteral can wrap any serde_json::Value;
                    // normalise primitive contents to the matching literal variant.
                    ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(v)) => match v {
                        serde_json::Value::Null => ExprOrLiteral::Null,
                        serde_json::Value::Bool(b) => ExprOrLiteral::Bool(b),
                        serde_json::Value::Number(n) => {
                            ExprOrLiteral::NumberLiteral(NumberLiteral::from(n))
                        }
                        serde_json::Value::String(s) => {
                            ExprOrLiteral::StringLiteral(StringLiteral::from(s))
                        }
                        other => ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(other)),
                    },
                    other => other,
                }
            }
        }

        #[cfg(feature = "fuzz")]
        impl<'a> arbitrary::Arbitrary<'a> for ExprOrLiteral {
            fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                let tag: u8 = u.arbitrary()?;
                Ok(match tag % 17 {
                    0 => ExprOrLiteral::Null,
                    1 => ExprOrLiteral::Bool(u.arbitrary()?),
                    2 => ExprOrLiteral::NumberLiteral(u.arbitrary()?),
                    3 => ExprOrLiteral::StringLiteral(u.arbitrary()?),
                    4 => ExprOrLiteral::GeoJSONObjectLiteral(u.arbitrary()?),
                    5 => ExprOrLiteral::JSONObjectLiteral(u.arbitrary()?),
                    6 => ExprOrLiteral::JSONArrayLiteral(u.arbitrary()?),
                    7 => ExprOrLiteral::AnyExpr(u.arbitrary()?),
                    8 => ExprOrLiteral::ArrayExpr(u.arbitrary()?),
                    9 => ExprOrLiteral::BooleanExpr(u.arbitrary()?),
                    10 => ExprOrLiteral::CollatorExpr(u.arbitrary()?),
                    11 => ExprOrLiteral::ColorExpr(u.arbitrary()?),
                    12 => ExprOrLiteral::FormattedExpr(u.arbitrary()?),
                    13 => ExprOrLiteral::ImageExpr(u.arbitrary()?),
                    14 => ExprOrLiteral::NumberExpr(u.arbitrary()?),
                    15 => ExprOrLiteral::ObjectExpr(u.arbitrary()?),
                    _ => ExprOrLiteral::StringExpr(u.arbitrary()?),
                }
                .normalize())
            }
        }
        "##);
    }

    #[test]
    fn test_generate_spec_interpolation() {
        let reference = serde_json::json!({
            "$version": 8,
            "$root": {},
            "interpolation": {
              "type": "array",
              "value": "interpolation_name",
              "minimum": 1,
              "doc": "An interpolation defines how to transition between items. The first element of an interpolation array is a string naming the interpolation operator, e.g. `\"linear\"` or `\"exponential\"`. Elements that follow (if any) are the _arguments_ to the interpolation."
            },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = crate::mir::MirSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec), @r#"
        /// JSON number in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct NumberLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
            serde_json::Number,
        );

        /// JSON string in an expression position
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct StringLiteral(std::string::String);

        impl From<serde_json::Number> for NumberLiteral {
            fn from(n: serde_json::Number) -> Self {
                Self(n)
            }
        }

        impl From<std::string::String> for StringLiteral {
            fn from(s: std::string::String) -> Self {
                Self(s)
            }
        }

        /// GeoJSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct GeoJSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_geojson))]
            geojson::GeoJson,
        );

        /// JSON object literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONObjectLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_value))]
            pub  serde_json::Value,
        );

        /// JSON array literal
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct JSONArrayLiteral(
            #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_value))]
            pub Vec<serde_json::Value>,
        );

        /// Array whose elements are string literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfStringLiteral(Vec<StringLiteral>);

        /// Array whose elements are number literals (e.g. match labels)
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct ArrayOfNumberLiteral(Vec<NumberLiteral>);

        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct MaplibreStyleSpecification;

        /// An interpolation defines how to transition between items. The first element of an interpolation array is a string naming the interpolation operator, e.g. `"linear"` or `"exponential"`. Elements that follow (if any) are the _arguments_ to the interpolation.
        ///
        /// Range: 1..
        #[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub struct Interpolation(InterpolationName);

        /// An expression node or a literal JSON value in expression positions.
        #[derive(PartialEq, Debug, Clone)]
        pub enum ExprOrLiteral {
            Null,
            Bool(bool),
            NumberLiteral(NumberLiteral),
            StringLiteral(StringLiteral),
            GeoJSONObjectLiteral(GeoJSONObjectLiteral),
            JSONObjectLiteral(JSONObjectLiteral),
            JSONArrayLiteral(JSONArrayLiteral),
            AnyExpr(Box<Any>),
            ArrayExpr(Box<Array>),
            BooleanExpr(Box<Boolean>),
            CollatorExpr(Box<Collator>),
            ColorExpr(Box<Color>),
            FormattedExpr(Box<Formatted>),
            ImageExpr(Box<Image>),
            NumberExpr(Box<Number>),
            ObjectExpr(Box<Object>),
            StringExpr(Box<String>),
        }

        impl serde::Serialize for ExprOrLiteral {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::Null => serializer.serialize_unit(),
                    Self::Bool(v) => v.serialize(serializer),
                    Self::NumberLiteral(v) => v.serialize(serializer),
                    Self::StringLiteral(v) => v.serialize(serializer),
                    Self::GeoJSONObjectLiteral(v) => v.serialize(serializer),
                    Self::JSONObjectLiteral(v) => v.serialize(serializer),
                    Self::AnyExpr(v) => v.as_ref().serialize(serializer),
                    Self::ArrayExpr(v) => v.as_ref().serialize(serializer),
                    Self::BooleanExpr(v) => v.as_ref().serialize(serializer),
                    Self::CollatorExpr(v) => v.as_ref().serialize(serializer),
                    Self::ColorExpr(v) => v.as_ref().serialize(serializer),
                    Self::FormattedExpr(v) => v.as_ref().serialize(serializer),
                    Self::ImageExpr(v) => v.as_ref().serialize(serializer),
                    Self::NumberExpr(v) => v.as_ref().serialize(serializer),
                    Self::ObjectExpr(v) => v.as_ref().serialize(serializer),
                    Self::StringExpr(v) => v.as_ref().serialize(serializer),
                    Self::JSONArrayLiteral(v) => v.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for ExprOrLiteral {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                if value.is_null() {
                    return Ok(Self::Null);
                }
                match <bool as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Bool(v)),
                    Err(e) => errors.push(("Bool", e.to_string())),
                }
                match <NumberLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberLiteral(v)),
                    Err(e) => errors.push(("NumberLiteral", e.to_string())),
                }
                match <StringLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringLiteral(v)),
                    Err(e) => errors.push(("StringLiteral", e.to_string())),
                }
                match <GeoJSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::GeoJSONObjectLiteral(v)),
                    Err(e) => errors.push(("GeoJSONObjectLiteral", e.to_string())),
                }
                if !(value.is_array()) {
                    match <JSONObjectLiteral as serde::Deserialize>::deserialize(&value) {
                        Ok(v) => return Ok(Self::JSONObjectLiteral(v)),
                        Err(e) => errors.push(("JSONObjectLiteral", e.to_string())),
                    }
                }
                match <Any as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::AnyExpr(Box::new(v))),
                    Err(e) => errors.push(("AnyExpr", e.to_string())),
                }
                match <Array as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ArrayExpr(Box::new(v))),
                    Err(e) => errors.push(("ArrayExpr", e.to_string())),
                }
                match <Boolean as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::BooleanExpr(Box::new(v))),
                    Err(e) => errors.push(("BooleanExpr", e.to_string())),
                }
                match <Collator as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::CollatorExpr(Box::new(v))),
                    Err(e) => errors.push(("CollatorExpr", e.to_string())),
                }
                match <Color as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ColorExpr(Box::new(v))),
                    Err(e) => errors.push(("ColorExpr", e.to_string())),
                }
                match <Formatted as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::FormattedExpr(Box::new(v))),
                    Err(e) => errors.push(("FormattedExpr", e.to_string())),
                }
                match <Image as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ImageExpr(Box::new(v))),
                    Err(e) => errors.push(("ImageExpr", e.to_string())),
                }
                match <Number as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::NumberExpr(Box::new(v))),
                    Err(e) => errors.push(("NumberExpr", e.to_string())),
                }
                match <Object as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::ObjectExpr(Box::new(v))),
                    Err(e) => errors.push(("ObjectExpr", e.to_string())),
                }
                match <String as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::StringExpr(Box::new(v))),
                    Err(e) => errors.push(("StringExpr", e.to_string())),
                }
                match <JSONArrayLiteral as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::JSONArrayLiteral(v)),
                    Err(e) => errors.push(("JSONArrayLiteral", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "ExprOrLiteral: no variant matched. Expected Null | Bool(bool) | NumberLiteral(NumberLiteral) | StringLiteral(StringLiteral) | GeoJSONObjectLiteral(GeoJSONObjectLiteral) | JSONObjectLiteral(JSONObjectLiteral) | AnyExpr(Any) | ArrayExpr(Array) | BooleanExpr(Boolean) | CollatorExpr(Collator) | ColorExpr(Color) | FormattedExpr(Formatted) | ImageExpr(Image) | NumberExpr(Number) | ObjectExpr(Object) | StringExpr(String) | JSONArrayLiteral(JSONArrayLiteral). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }

        impl ExprOrLiteral {
            /// Collapse expression-wrapping-literal into the canonical literal variant.
            ///
            /// Expression enums (`Boolean`, `Number`, `String`, `Color`) have `Literal` and
            /// `AnyExpr` variants for top-level use (e.g. filter position). When boxed inside
            /// `ExprOrLiteral`, those overlap with `Bool`, `NumberLiteral`, `StringLiteral`,
            /// and `AnyExpr`. This method normalises to the canonical form.
            #[must_use]
            pub fn normalize(self) -> Self {
                match self {
                    ExprOrLiteral::BooleanExpr(b) => match *b {
                        Boolean::Literal(v) => ExprOrLiteral::Bool(v),
                        Boolean::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::BooleanExpr(Box::new(other)),
                    },
                    ExprOrLiteral::NumberExpr(n) => match *n {
                        Number::Literal(v) => ExprOrLiteral::NumberLiteral(v),
                        Number::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::NumberExpr(Box::new(other)),
                    },
                    ExprOrLiteral::StringExpr(s) => match *s {
                        String::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        String::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::StringExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ColorExpr(c) => match *c {
                        Color::Literal(v) => ExprOrLiteral::StringLiteral(v),
                        Color::AnyExpr(a) => ExprOrLiteral::AnyExpr(a),
                        other => ExprOrLiteral::ColorExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ArrayExpr(a) => match *a {
                        Array::Literal(v) => ExprOrLiteral::JSONArrayLiteral(v),
                        other => ExprOrLiteral::ArrayExpr(Box::new(other)),
                    },
                    ExprOrLiteral::ObjectExpr(o) => match *o {
                        Object::Literal(v) => ExprOrLiteral::JSONObjectLiteral(v),
                        other => ExprOrLiteral::ObjectExpr(Box::new(other)),
                    },
                    // JSONObjectLiteral/JSONArrayLiteral can wrap any serde_json::Value;
                    // normalise primitive contents to the matching literal variant.
                    ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(v)) => match v {
                        serde_json::Value::Null => ExprOrLiteral::Null,
                        serde_json::Value::Bool(b) => ExprOrLiteral::Bool(b),
                        serde_json::Value::Number(n) => {
                            ExprOrLiteral::NumberLiteral(NumberLiteral::from(n))
                        }
                        serde_json::Value::String(s) => {
                            ExprOrLiteral::StringLiteral(StringLiteral::from(s))
                        }
                        other => ExprOrLiteral::JSONObjectLiteral(JSONObjectLiteral(other)),
                    },
                    other => other,
                }
            }
        }

        #[cfg(feature = "fuzz")]
        impl<'a> arbitrary::Arbitrary<'a> for ExprOrLiteral {
            fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                let tag: u8 = u.arbitrary()?;
                Ok(match tag % 17 {
                    0 => ExprOrLiteral::Null,
                    1 => ExprOrLiteral::Bool(u.arbitrary()?),
                    2 => ExprOrLiteral::NumberLiteral(u.arbitrary()?),
                    3 => ExprOrLiteral::StringLiteral(u.arbitrary()?),
                    4 => ExprOrLiteral::GeoJSONObjectLiteral(u.arbitrary()?),
                    5 => ExprOrLiteral::JSONObjectLiteral(u.arbitrary()?),
                    6 => ExprOrLiteral::JSONArrayLiteral(u.arbitrary()?),
                    7 => ExprOrLiteral::AnyExpr(u.arbitrary()?),
                    8 => ExprOrLiteral::ArrayExpr(u.arbitrary()?),
                    9 => ExprOrLiteral::BooleanExpr(u.arbitrary()?),
                    10 => ExprOrLiteral::CollatorExpr(u.arbitrary()?),
                    11 => ExprOrLiteral::ColorExpr(u.arbitrary()?),
                    12 => ExprOrLiteral::FormattedExpr(u.arbitrary()?),
                    13 => ExprOrLiteral::ImageExpr(u.arbitrary()?),
                    14 => ExprOrLiteral::NumberExpr(u.arbitrary()?),
                    15 => ExprOrLiteral::ObjectExpr(u.arbitrary()?),
                    _ => ExprOrLiteral::StringExpr(u.arbitrary()?),
                }
                .normalize())
            }
        }

        #[cfg(test)]
        mod test {
            use super::*;
        }
        "#);
    }
}
