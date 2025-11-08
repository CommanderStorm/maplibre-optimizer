use codegen::Scope;
use serde_json::{Number, Value};

use crate::decoder::{ArrayValue, EnumValues, Fields};

#[allow(clippy::too_many_arguments)]
pub fn generate(
    scope: &mut Scope,
    name: &str,
    common: &Fields,
    default: Option<&Vec<Value>>,
    value: &ArrayValue,
    values: Option<&EnumValues>,
    min: Option<&Number>,
    max: Option<&Number>,
    length: Option<&usize>,
) {
    scope
        .new_struct(name)
        .doc(&common.doc_with_range(max, min, None))
        .attr("deprecated = \"not_implemented\"")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");

    if let Some(default) = default {
        let mut line = String::from("vec![");
        for item in default {
            if line.len() > "vec![".len() {
                line.push_str(", ");
            }
            line.push_str(&item.to_string());
        }
        line.push(']');

        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(&line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::SimpleArrayValue;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(
            &mut scope,
            "Foo",
            &Fields::default(),
            None,
            &ArrayValue::Simple(SimpleArrayValue::Star),
            None,
            None,
            None,
            None,
        );
        insta::assert_snapshot!(scope.to_string(), @r##"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "not_implemented"]
        struct Foo(serde_json::Value);
        "##)
    }
}
