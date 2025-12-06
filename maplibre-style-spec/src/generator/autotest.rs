use codegen2::Scope;
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

pub fn generate_test_from_examples_if_present(
    scope: &mut Scope,
    name: &str,
    examples: Vec<&Value>,
) {
    let fun = scope
        .get_or_new_module("test")
        .new_fn(to_snake_case(&format!("test_example_{name}_decodes")))
        .arg("#[case] example", "serde_json::Value")
        .attr("rstest::rstest");
    for example in examples {
        if let Some(arr) = example.as_array() {
            if let Some(fst) = arr.first() {
                if let Some(op) = fst.as_str() {
                    let ident = to_snake_case(op);
                    fun.attr(format!("case::t_{ident}(serde_json::json!({example}))"));
                    continue;
                }
            }
        }
        fun.attr(format!("case(serde_json::json!({example}))"));
    }
    fun.line(format!(
        "let _ = serde_json::from_value::<{name}>(example).expect(\"example should decode\");"
    ));
}
