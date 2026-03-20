use codegen2::Scope;

use crate::generator::formatter::to_upper_camel_case;
use crate::mir::types::VersionEnum;

pub fn generate_version(scope: &mut Scope, name: &str, doc: &str, versions: &VersionEnum) {
    assert!(versions.versions.len() <= u8::MAX as usize);

    let enu = scope
        .new_enum(name)
        .doc(doc)
        .vis("pub")
        .repr("u8")
        .derive(
            "serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Eq, Debug, Clone, Copy",
        );
    for v in &versions.versions {
        enu.new_variant(to_upper_camel_case(v.to_string()))
            .discriminant(v.to_string());
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;
    use crate::mir::IntermediateSpec;

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
        let spec = IntermediateSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }
}
