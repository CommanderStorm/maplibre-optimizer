use codegen::Scope;
use serde_json::Value;

use crate::decoder::{EnumValues, Fields};
use crate::generator::formatter::to_upper_camel_case;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Value>,
    values: &EnumValues,
) {
    match values {
        EnumValues::Simple(values) => {
            let enu = scope
                .new_enum(name)
                .doc(&common.doc)
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
            for value in values {
                enu.new_variant(to_upper_camel_case(value))
                    .annotation(format!("serde(rename=\"{value}\")"));
            }
        }
        EnumValues::Numeric(values) => {
            scope
                .new_struct(&name)
                .doc(&common.doc)
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy")
                .tuple_field("u8");
            assert!(values.len() <= u8::MAX as usize);
            assert!(
                values
                    .iter()
                    .all(|v| v.as_u64().is_some_and(|v| v <= u8::MAX as u64))
            );
            // todo: contribute proper repr(u8) variant support
        }
        EnumValues::Complex(values) => {
            let enu = scope
                .new_enum(name)
                .doc(&common.doc)
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
            for (key, value) in values {
                enu.new_variant(to_upper_camel_case(key))
                    .annotation(format!("#[serde(rename=\"{key}\")]"))
                    .annotation(format!("/// {}", value.doc));
                // todo: this is sort of a hack, but it works for now
                // upstream a proprer .doc() method
            }
        }
    }

    if let Some(default) = default {
        scope
            .new_impl(&name)
            .impl_trait("Default")
            .new_fn("default")
            .line(default);
    }
}
