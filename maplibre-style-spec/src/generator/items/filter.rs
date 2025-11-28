use codegen2::Scope;

use crate::decoder::Fields;
use crate::generator::autotest::generate_test_from_example_if_present;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .derive("serde::Deserialize, PartialEq, Debug, Clone")
        .tuple_field("BooleanExpression");
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
        insta::assert_snapshot!(scope.to_string(), @r"
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct Foo(BooleanExpression);
        ")
    }

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {
          "filter": {
            "type": "filter",
            "doc": "A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions."
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification {
            /// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
            pub filter: Option<RootFilter>,
        }

        /// A expression specifying conditions on source features. Only features that match the filter are displayed. Zoom expressions in filters are only evaluated at integer zoom levels. The `feature-state` expression is not supported in filter expressions.
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        struct RootFilter(BooleanExpression);

        #[cfg(test)] 
        mod test {
            use super::*;

        }
        ");
    }
}
