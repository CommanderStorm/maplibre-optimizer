mod regular;
mod syntax;
mod version;

use codegen2::Scope;
use serde_json::Value;

use crate::decoder::{EnumValues, Fields};
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
        EnumValues::Version(values) => version::generate_version(scope, name, &common, values),
        EnumValues::Enum(values) => regular::generate_regular_enum(scope, name, &common, values),
        EnumValues::SyntaxEnum(values) => {
            syntax::generate_syntax_enum(scope, name, &common, values)
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
    generate_test_from_example_if_present(scope, name, common.example.as_ref());
}
