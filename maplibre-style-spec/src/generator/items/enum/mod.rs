pub mod regular;
pub mod syntax;
pub mod version;

use codegen2::Scope;
use serde_json::Value;

use crate::generator::autotest::generate_test_from_example_if_present;
use crate::generator::formatter::to_upper_camel_case;
use crate::mir::types::{EnumField, MirEnum, RegularEnum};

/// Dispatch an `EnumField` from the MIR to the appropriate enum generator.
pub fn generate_mir(scope: &mut Scope, name: &str, field: &EnumField) {
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
    if matches!(&field.variants, MirEnum::Version(_)) {
        if let Some(default) = &field.default {
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
    }
    if matches!(&field.variants, MirEnum::Regular(_) | MirEnum::Version(_)) {
        generate_test_from_example_if_present(scope, name, field.meta.example.as_ref());
    }
}

/// Convenience wrapper for generating a regular enum from array.rs (inline enum element).
pub fn generate_regular(
    scope: &mut Scope,
    name: &str,
    doc: &str,
    variants: &RegularEnum,
    default: Option<&Value>,
) {
    regular::generate_regular_enum(scope, name, doc, variants, default);
}
