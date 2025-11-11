use std::collections::BTreeMap;
use codegen::Scope;
use serde_json::{Number, Value};

use crate::decoder::{EnumDocs, EnumValues, Fields};
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Value>,
    values: &EnumValues,
) {
    match values {
        EnumValues::Version(values) => {
            generate_version(scope, name, &common, values);
        }
        EnumValues::Enum(values) => {
            generate_regular_enum(scope, name, &common, values);
        }
    }

    if let Some(default) = default {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!(
                "Self::{}",
                to_upper_camel_case(&default.to_string())
            ));
    }
    generate_test_from_example_if_present(scope, name, common);
}


fn generate_regular_enum(scope: &mut Scope, name: &str, common: &&Fields, values: &BTreeMap<String, EnumDocs>) {
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

fn generate_version(scope: &mut Scope, name: &str, common: &&Fields, values: &Vec<Number>) {
    assert!(values.len() <= u8::MAX as usize);

    let enu = scope
        .new_enum(name)
        .doc(&common.doc)
        .vis("pub")
        .repr("u8")
        .derive("serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy");
    for v in values {
        enu.new_variant(to_upper_camel_case(&v.to_string()))
            .discriminant(v.to_string());
    }
    assert!(
        values
            .iter()
            .all(|v| v.as_u64().is_some_and(|v| v <= u8::MAX as u64))
    );
}
