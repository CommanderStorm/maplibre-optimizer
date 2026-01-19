use geojson::Value;

pub enum BasicType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Color,
}

pub enum Structure {
    Basic(BasicType),
    Vec(BasicType),
    Array(BasicType),
}
struct ExpressionStructure {
    outputs: Structure,
    capability: ExpressionCapabilities,
}

struct ExpressionCapabilities {
    interpolated: bool,
    zoom: bool,
    feature: bool,
    global_state: bool,
}
pub struct SimpleField {
    r#type: Structure,
    default: Option<Value>,
    codegen: CodegenMetadata,
    analysis: AnalysisMetadata,
}
pub struct CodegenMetadata {
    docs: String,
    example: Option<Value>,
}
pub struct AnalysisMetadata {
    min: Option<Value>,
    max: Option<Value>,
    enum_values: Option<Vec<Value>>,
}
