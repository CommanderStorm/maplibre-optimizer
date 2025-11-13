use codegen::Scope;
use serde_json::Value;

use crate::generator::formatter::to_snake_case;

pub fn generate_test_from_example_if_present(
    scope: &mut Scope,
    name: &str,
    example: Option<&Value>,
) {
    let Some(example) = &example else {
        return;
    };
    let fun = scope
        .get_or_new_module("test")
        .new_fn(to_snake_case(&format!("test_example_{name}_decodes")))
        .attr("test");
    fun.line(format!("let example = serde_json::json!({example});"));
    fun.line(format!(
        "let _ = serde_json::from_value::<{name}>(example).expect(\"example should decode\");"
    ));
}
