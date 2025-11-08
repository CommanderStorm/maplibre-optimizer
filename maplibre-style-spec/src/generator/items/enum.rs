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
                let var_name = to_upper_camel_case(value);
                let var= enu.new_variant(&to_upper_camel_case(value));
                if value != &var_name {
                    var.annotation(format!("serde(rename=\"{value}\")"));
                }
            }
        }
        EnumValues::Numeric(values) => {
            assert!(values.len() <= u8::MAX as usize);

            let enu=
scope                .new_enum(&name)
                .doc(&common.doc)
                .vis("pub")
                .repr("u8")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
            for v in values{
                enu.new_variant(to_upper_camel_case(&v.to_string())).discriminant(v.to_string());
            }
            assert!(
                values
                    .iter()
                    .all(|v| v.as_u64().is_some_and(|v| v <= u8::MAX as u64))
            );
        }
        EnumValues::Complex(values) => {
            let enu = scope
                .new_enum(name)
                .doc(&common.doc)
                .vis("pub")
                .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
            for (key, value) in values {
                let var_name = to_upper_camel_case(key);
                let var = enu.new_variant(&var_name).doc(&value.doc);
                if key != &var_name {
                    var.annotation(format!("#[serde(rename=\"{key}\")]"));
                }
            }
        }
    }

    if let Some(default) = default {
        scope
            .new_impl(&name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!("Self::{}", to_upper_camel_case(&default.to_string())));
    }
}
