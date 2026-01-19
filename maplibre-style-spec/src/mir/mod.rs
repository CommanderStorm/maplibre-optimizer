mod expressions;
mod layers;
mod preprocessing;
mod resources;
mod root;
mod sources;
mod types;

use crate::decoder;
use crate::mir::expressions::IntermediateExpression;
use crate::mir::layers::IntermediateLayers;
use crate::mir::root::IntermediateRootPrimitives;
use crate::mir::sources::IntermediateSources;

pub struct IntermediateSpec {
    /// simple items (numbers, strings, ...) which are at the root of the style
    root_primitives: IntermediateRootPrimitives,
    /// some items can be constructed based on data
    expressions: IntermediateExpression,
    /// rendering layers
    layers: IntermediateLayers,
    /// data sources
    sources: IntermediateSources,
    fonts: IntermediateFontResources,
    glyphs: IntermediateSpriteResources,
}
impl From<decoder::StyleReference> for IntermediateSpec {
    fn from(mut value: decoder::StyleReference) -> Self {
        let expressions = preprocessing::preprocess_expression(&mut value.fields);
        let layers = preprocessing::preprocess_layers(&mut value);
        let sources = preprocessing::preprocess_sources(&mut value.fields);

        Self {
            root_primitives: IntermediateRootPrimitives::from(value.root),
            expressions,
            layers,
        }
    }
}
