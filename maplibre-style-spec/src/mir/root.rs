use std::collections::BTreeMap;

use crate::decoder::ParsedItem;
use crate::mir::lower::lower_parsed_item;
use crate::mir::types::MirField;

/// All simple root-level items that don't need special preprocessing.
/// Example: `"center": [50, 10]`
pub struct IntermediateRootPrimitives(pub BTreeMap<String, MirField>);

impl From<BTreeMap<String, ParsedItem>> for IntermediateRootPrimitives {
    fn from(root_items: BTreeMap<String, ParsedItem>) -> Self {
        let mut res = BTreeMap::new();
        for (key, item) in root_items {
            let field = match &item {
                ParsedItem::Reference { references, .. } => {
                    // References in $root are fine — lower them as ReferenceField
                    lower_parsed_item(&key, item)
                }
                ParsedItem::Primitive(_) => lower_parsed_item(&key, item),
            };
            res.insert(key, field);
        }
        Self(res)
    }
}
