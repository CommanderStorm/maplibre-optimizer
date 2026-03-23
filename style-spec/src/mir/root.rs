use std::collections::BTreeMap;

use crate::decoder::DecodedParsedItem;
use crate::mir::lower::lower_parsed_item;
use crate::mir::types::MirField;

/// All simple root-level items that don't need special preprocessing.
/// Example: `"center": [50, 10]`
pub struct MirRootPrimitives(pub BTreeMap<String, MirField>);

impl From<BTreeMap<String, DecodedParsedItem>> for MirRootPrimitives {
    fn from(root_items: BTreeMap<String, DecodedParsedItem>) -> Self {
        let mut res = BTreeMap::new();
        for (key, item) in root_items {
            let field = match &item {
                DecodedParsedItem::Reference { references: _, .. } => {
                    // References in $root are fine — lower them as MirReferenceField
                    lower_parsed_item(&key, item)
                }
                DecodedParsedItem::Primitive(_) => lower_parsed_item(&key, item),
            };
            res.insert(key, field);
        }
        Self(res)
    }
}
