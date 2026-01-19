use crate::decoder;
use crate::decoder::TopLevelItem;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct IntermediateExpression {}

impl From<decoder::TopLevelItem> for IntermediateExpression {
    fn from(value: TopLevelItem) -> Self {
        todo!()
    }
}
