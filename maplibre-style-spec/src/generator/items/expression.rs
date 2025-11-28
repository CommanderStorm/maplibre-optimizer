use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"expression not implemented\"")
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("serde_json::Value");
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default());
        insta::assert_snapshot!(scope.to_string(), @r#"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "expression not implemented"]
        struct Foo(serde_json::Value);
        "#)
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "function": {
          "expression": {
            "type": "expression",
            "doc": "An expression."
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification;

        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct Function {
            /// An expression.
            pub expression: Option<FunctionExpression>,
        }

        /// An expression.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        #[deprecated = "expression not implemented"]
        struct FunctionExpression(serde_json::Value);

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        "#);
    }
}
