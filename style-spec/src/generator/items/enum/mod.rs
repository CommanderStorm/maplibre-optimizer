pub mod regular;
pub mod syntax;
pub mod version;

use codegen2::Scope;
use serde_json::Value;

use super::escape_doc_for_macro;
use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::mir::types::{MirEnum, MirEnumField, MirRegularEnum};

/// Dispatch an `MirEnumField` from the MIR to the appropriate enum generator.
pub fn generate_mir(scope: &mut Scope, name: &str, field: &MirEnumField) {
    // Expression-backed enums use string_prop!, but skip the visibility pattern
    // (always `none`/`visible`) which uses a shared type alias instead.
    if field.meta.expression.is_some() && !is_visibility_pattern(field) {
        let doc = escape_doc_for_macro(&field.meta.doc);
        let mut args = format!("{name}, doc = \"{doc}\"");
        if let Some(s) = field.default.as_ref().and_then(|d| d.as_str()) {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            args.push_str(&format!(", default = \"{escaped}\".to_string()"));
        }
        scope.raw(format!("string_prop!({args});"));
        generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
        return;
    }

    match &field.variants {
        MirEnum::Version(v) => version::generate_version(scope, name, &field.meta.doc, v),
        MirEnum::Regular(r) => {
            regular::generate_regular_enum(scope, name, &field.meta.doc, r, field.default.as_ref())
        }
        MirEnum::Syntax(s) => {
            syntax::generate_syntax_enum(scope, name, &field.meta.doc, &s.variants)
        }
    }

    // `regular::generate_regular_enum` already emits `Default` when a default is set.
    // Version enums still need it here (see `items/enum/version.rs`).
    if matches!(&field.variants, MirEnum::Version(_))
        && let Some(default) = &field.default
    {
        scope
            .new_impl(name)
            .impl_trait("Default")
            .new_fn("default")
            .ret("Self")
            .line(format!(
                "Self::{}",
                to_upper_camel_case(default.to_string())
            ));
    }
    if matches!(&field.variants, MirEnum::Regular(_) | MirEnum::Version(_)) {
        generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
    }
}

fn is_visibility_pattern(field: &MirEnumField) -> bool {
    if let MirEnum::Regular(r) = &field.variants {
        let keys: Vec<&str> = r.variants.keys().map(String::as_str).collect();
        let has_none_visible =
            keys.len() == 2 && keys.contains(&"none") && keys.contains(&"visible");
        let default_is_visible = field
            .default
            .as_ref()
            .is_some_and(|d| d.as_str() == Some("visible"));
        has_none_visible && default_is_visible
    } else {
        false
    }
}

/// Convenience wrapper for generating a regular enum from array.rs (inline enum element).
pub fn generate_regular(
    scope: &mut Scope,
    name: &str,
    doc: &str,
    variants: &MirRegularEnum,
    default: Option<&Value>,
) {
    regular::generate_regular_enum(scope, name, doc, variants, default);
}
