mod expressions;
mod layers;
mod sources;

use std::collections::{BTreeMap, HashMap};

pub use expressions::preprocess_expression;
pub use layers::preprocess_layers;
pub use sources::preprocess_sources;

use crate::decoder::{DecodedParsedItem, DecodedTopLevelItem};

/// Pops a `OneOf` entry keyed by `key` from `fields`, then removes each referenced
/// group from `fields` and returns them as a map from (stripped) group name → fields.
pub fn pop_one_of_as_group(
    fields: &mut BTreeMap<String, DecodedTopLevelItem>,
    key: &str,
) -> HashMap<String, BTreeMap<String, DecodedParsedItem>> {
    let one_of = fields
        .remove(key)
        .unwrap_or_else(|| panic!("expected '{key}' in fields"));
    let one_of_items = one_of.as_one_of();

    let prefix = format!("{key}_");
    let mut set = HashMap::new();
    for item in one_of_items {
        let clean_group_name = item
            .strip_prefix(&prefix)
            .unwrap_or_else(|| panic!("'{item}' should start with prefix '{prefix}'"));
        let group = fields
            .remove(item)
            .unwrap_or_else(|| {
                panic!("'{item}' referenced in '{key}' OneOf but not found in fields")
            })
            .as_group()
            .clone();
        set.insert(clean_group_name.to_string(), group);
    }
    set
}
