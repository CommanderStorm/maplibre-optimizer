use codegen2::Scope;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::fuzz;
use crate::generator::items::number::generate_number_default;
use crate::generator::untagged::{self, Variant};
use crate::mir::types::NumberArrayField;

pub fn generate(scope: &mut Scope, name: &str, field: &NumberArrayField) {
    let enu = scope
        .new_enum(name)
        .vis("pub")
        .doc(&field.meta.doc)
        .derive("PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("One")
        .tuple_with_attrs([fuzz::ARB_JSON_NUMBER], "serde_json::Number");
    enu.new_variant("Many")
        .tuple_with_attrs([fuzz::ARB_VEC_JSON_NUMBER], "Vec<serde_json::Number>");
    untagged::emit_untagged_serde(
        scope,
        name,
        &[
            Variant {
                name: "One".into(),
                inner_type: "serde_json::Number".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
            Variant {
                name: "Many".into(),
                inner_type: "Vec<serde_json::Number>".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
        ],
    );

    if let Some(default) = &field.default {
        let default_expr = generate_number_default(default);
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::One({default_expr})"));
    }
    generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::types::FieldMeta;

    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &NumberArrayField {
                meta: FieldMeta::default(),
                default: None,
                min: None,
                max: None,
            },
        );
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(PartialEq, Debug, Clone)]
        #[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
        pub enum Foo {
            One(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_json_number))]
                serde_json::Number,
            ),
            Many(
                #[cfg_attr(feature = "fuzz", arbitrary(with = crate::fuzz_helpers::arbitrary_vec_json_number))]
                 Vec<serde_json::Number>,
            ),
        }

        impl serde::Serialize for Foo {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    Self::One(v) => v.serialize(serializer),
                    Self::Many(v) => v.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for Foo {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let mut errors: Vec<(&str, std::string::String)> = Vec::new();
                match <serde_json::Number as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::One(v)),
                    Err(e) => errors.push(("One", e.to_string())),
                }
                match <Vec<serde_json::Number> as serde::Deserialize>::deserialize(&value) {
                    Ok(v) => return Ok(Self::Many(v)),
                    Err(e) => errors.push(("Many", e.to_string())),
                }

                let details: Vec<std::string::String> =
                    errors.iter().map(|(v, e)| format!("{v}: {e}")).collect();
                Err(serde::de::Error::custom(format!(
                    "Foo: no variant matched. Expected One(serde_json::Number) | Many(Vec<serde_json::Number>). Errors: [{}]",
                    details.join("; ")
                )))
            }
        }
        "#)
    }
}
