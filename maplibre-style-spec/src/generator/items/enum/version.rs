use codegen2::Scope;
use serde_json::Number;

use crate::decoder::Fields;
use crate::generator::formatter::to_upper_camel_case;

pub fn generate_version(scope: &mut Scope, name: &str, common: &&Fields, values: &Vec<Number>) {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;

    #[test]
    fn test_generate_spec_version() {
        let reference = json!({
        "$version": 8,
        "$root": {
          "version": {
            "required": true,
            "type": "enum",
            "values": [
              8
            ],
            "doc": "Style specification version number. Must be 8.",
            "example": 8
          },
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        insta::assert_snapshot!(crate::generator::generate_spec_scope(reference), @r#"
        /// This is a Maplibre Style Specification
        #[derive(serde::Deserialize, PartialEq, Debug, Clone)]
        pub struct MaplibreStyleSpecification {
            /// Style specification version number. Must be 8.
            pub version: RootVersion,
        }

        /// Style specification version number. Must be 8.
        #[derive(serde::Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
        #[repr(u8)]
        pub enum RootVersion {
            Eight = 8,
        }

        #[cfg(test)]
        mod test {
            use super::*;

            #[test]
            fn test_example_root_version_decodes() {
                let example = serde_json::json!(8);
                let _ = serde_json::from_value::<RootVersion>(example).expect("example should decode");
            }
        }
        "#);
    }
}
