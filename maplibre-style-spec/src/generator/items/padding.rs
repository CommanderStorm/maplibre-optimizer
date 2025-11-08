use codegen::Scope;
use serde_json::Number;

use crate::decoder::Fields;

pub fn generate(scope: &mut Scope, name: &str, common: &Fields, default: &[Number]) {
    scope
        .new_struct(name)
        .doc(&common.doc)
        .attr("deprecated = \"not_implemented\"")
        .tuple_field("serde_json::Value");

    let mut line = String::from("vec![");
    for item in default {
        if line.len() > "vec![".len() {
            line.push_str(", ");
        }
        line.push_str(&item.to_string());
    }
    line.push_str("]");

    scope
        .new_impl(&name)
        .impl_trait("Default")
        .new_fn("default")
        .line(line);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_empty() {
        let mut scope = Scope::new();
        generate(&mut scope, "Foo", &Fields::default(), &[]);
        insta::assert_snapshot!(scope.to_string(), @"")
    }
}
