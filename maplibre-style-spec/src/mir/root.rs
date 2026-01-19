use crate::decoder::{ParsedItem, PrimitiveType};
use crate::mir::types::IntermediateType;
use serde_json::Value;
use std::collections::BTreeMap;

/// This holds all primitive root items that don't need extra handling
/// Example: `"center": [50, 10]`
pub struct IntermediateRootPrimitives(BTreeMap<String, RootItem>);

impl From<BTreeMap<String, ParsedItem>> for IntermediateRootPrimitives {
    fn from(root_items: BTreeMap<String, ParsedItem>) -> Self {
        let mut res = BTreeMap::new();
        for (key, item) in root_items {
            let item = match item {
                ParsedItem::Primitive(p) => RootItem::from(p),
                ParsedItem::Reference { references, .. } => {
                    panic!(
                        "{references} needs to be handled one level up or preprocessed to be primitive"
                    )
                }
            };
            res.insert(key, item);
        }
        Self(res)
    }
}

struct RootItem {
    /// what type of item this is?
    r#type: IntermediateType,
    /// documentation for the item
    doc: String,
    /// an example to sniff-test if our impl is correct
    example: Option<Value>,
}

impl From<PrimitiveType> for RootItem {
    fn from(value: PrimitiveType) -> Self {
        todo!()
    }
}
