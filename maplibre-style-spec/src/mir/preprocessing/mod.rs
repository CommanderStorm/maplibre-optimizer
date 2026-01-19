mod expressions;
mod layers;
mod sources;

use crate::decoder::{ParsedItem, TopLevelItem};
pub use expressions::preprocess_expression;
pub use layers::preprocess_layers;
pub use sources::preprocess_sources;
use std::collections::{BTreeMap, HashMap};

/// poops `key` from `fiels` and returns the fields it points to
fn pop_one_of_as_group(
    fields: &mut BTreeMap<String, TopLevelItem>,
    key: &str,
) -> HashMap<String, BTreeMap<String, ParsedItem>> {
    let one_of = fields.remove(key).unwrap();
    let one_of_items = one_of.as_one_of();

    let prefix = format!("{key}_");
    let mut set = HashMap::new();
    for item in one_of_items {
        let clean_group_name = item.strip_prefix(&prefix).unwrap();
        let group = fields.remove(item).unwrap().as_group().clone();
        set.insert(clean_group_name.to_string(), group);
    }
    set
}
