use std::collections::BTreeMap;

use crate::mir::types::SyntaxVariantDef;

/// All expression definitions, grouped by their output type.
/// Used to generate the per-output-type expression enums (e.g. `NumberExpression`).
pub struct IntermediateExpressions {
    /// Keyed by UpperCamelCase output type name (e.g. `"NumberExpression"`).
    pub by_output_type: BTreeMap<String, ExpressionGroup>,
}

/// All expression operators that produce a specific output type.
pub struct ExpressionGroup {
    /// Keyed by the expression operator name (e.g. `"literal"`, `"interpolate"`).
    pub variants: BTreeMap<String, SyntaxVariantDef>,
}
