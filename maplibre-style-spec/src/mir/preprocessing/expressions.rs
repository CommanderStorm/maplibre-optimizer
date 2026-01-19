use crate::decoder::{StyleReference, TopLevelItem};
use crate::mir::expressions::IntermediateExpression;
use std::collections::BTreeMap;

pub fn preprocess_expression(
    fields: &mut BTreeMap<String, TopLevelItem>,
) -> IntermediateExpression {
    let expression_name = fields
        .remove("expression_name")
        .expect("expression_name to be a top level item");
    let _ = fields.remove("expression");
}

#[test]
fn test_expression_name_renaming() {
    let mut reference: StyleReference =
        serde_json::from_str(include_str!("../fixture/expression_name_renaming.json")).unwrap();
    let exprs = preprocess_expression(&mut reference.fields);
    insta::assert_json_snapshot!(reference, @"");
}
