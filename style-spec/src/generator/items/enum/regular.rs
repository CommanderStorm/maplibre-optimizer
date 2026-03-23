use codegen2::Scope;

use crate::generator::formatter::to_upper_camel_case;
use crate::generator::fuzz;
use crate::mir::types::MirRegularEnum;

pub fn generate_regular_enum(
    scope: &mut Scope,
    name: &str,
    doc: &str,
    variants: &MirRegularEnum,
    default: Option<&serde_json::Value>,
) {
    // Detect the Visibility pattern: exactly 2 variants ("none" + "visible"), default "visible".
    if is_visibility_pattern(variants, default) {
        scope.raw(format!("pub type {name} = Visibility;"));
        return;
    }

    let default_key = default.map(|d| d.to_string());
    let default_variant = default_key.as_ref().map(to_upper_camel_case);
    let mut enu = scope
        .new_enum(name)
        .doc(doc)
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Eq, Debug, Clone, Copy")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    if default_key.is_some() {
        enu = enu.derive("Default");
    }
    for (key, value) in &variants.variants {
        let var_name = to_upper_camel_case(key);
        let var = enu.new_variant(&var_name).doc(&value.doc);
        if key != &var_name {
            var.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
        if default_variant.as_ref() == Some(&var_name) {
            var.annotation("#[default]");
        }
    }
}

fn is_visibility_pattern(variants: &MirRegularEnum, default: Option<&serde_json::Value>) -> bool {
    let keys: Vec<&str> = variants.variants.keys().map(String::as_str).collect();
    let has_none_visible = keys.len() == 2 && keys.contains(&"none") && keys.contains(&"visible");
    let default_is_visible = default.is_some_and(|d| d.as_str() == Some("visible"));
    has_none_visible && default_is_visible
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::decoder::StyleReference;
    use crate::mir::MirSpec;

    #[test]
    fn test_generate_spec() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "colorSpace": {
          "type": "enum",
          "values": {
            "rgb": {
              "doc": "Use the RGB color space to interpolate color values"
            },
            "lab": {
              "doc": "Use the LAB color space to interpolate color values."
            },
            "hcl": {
              "doc": "Use the HCL color space to interpolate color values, interpolating the Hue, Chroma, and Luminance channels individually."
            }
          },
          "doc": "The color space in which colors interpolated. Interpolating colors in perceptual color spaces like LAB and HCL tend to produce color ramps that look more consistent and produce colors that can be differentiated more easily than those interpolated in RGB space.",
          "default": "rgb"
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = MirSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_weird_keys() {
        let reference = json!({
        "$version": 8,
        "$root": {},
        "filter_operator": {
          "type": "enum",
          "values": {
            "==": {
              "doc": "`[\"==\", key, value]` equality: `feature[key] = value`"
            },
            "!=": {
              "doc": "`[\"!=\", key, value]` inequality: `feature[key] ≠ value`"
            },
            ">": {
              "doc": "`[\">\", key, value]` greater than: `feature[key] > value`"
            },
            ">=": {
              "doc": "`[\">=\", key, value]` greater than or equal: `feature[key] ≥ value`"
            },
          },
          "doc": "The filter operator."
        },
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = MirSpec::from(reference);
        insta::assert_snapshot!(crate::generator::generate_spec_scope(&spec));
    }
}
