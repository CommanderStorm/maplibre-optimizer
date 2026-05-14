use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder;
use crate::decoder::DecodedTopLevelItem;
use crate::decoder::r#enum::{
    DecodedOverload, DecodedSyntaxEnum, Literal, Parameter, ParameterType,
};
use crate::mir::types::MirSyntaxVariantDef;

// ── Generator-facing view (unchanged) ─────────────────────────────────────────

/// All expression operators that produce a specific output type.
/// Used by the code generator to emit per-output-type enums.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressionGroup {
    /// Keyed by the expression operator name (e.g. `"literal"`, `"interpolate"`).
    pub variants: BTreeMap<String, MirSyntaxVariantDef>,
}

// ── Top-level container ────────────────────────────────────────────────────────

/// The complete expression system lifted from the spec.
///
/// Two complementary views of the same data are provided:
///
/// - `by_output_type` — the code-generator view: operators grouped by their output type
///   with T expanded into every concrete type (same structure the generator emits enums from).
/// - `operators` — the optimizer/analysis view: all operators flat, with fully resolved
///   parameter types and polymorphism preserved via [`MirExprType::TypeVar`] rather than
///   being expanded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressions {
    /// Code-generator view: operators grouped by output-type name (T expanded).
    pub by_output_type: BTreeMap<String, MirExpressionGroup>,
    /// Optimizer view: all operators keyed by spec name, T-polymorphism preserved.
    pub operators: BTreeMap<String, MirExpressionOperator>,
}

// ── Per-operator ──────────────────────────────────────────────────────────────

/// One expression operator (e.g. `"case"`, `"interpolate"`, `"literal"`).
///
/// Contains fully resolved, MIR-typed calling conventions. Polymorphic operators
/// (those whose output type depends on their input) use [`MirExprType::TypeVar`] rather
/// than being expanded into one copy per concrete type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressionOperator {
    pub doc: String,
    /// Grouping hint from the spec (e.g. `"Lookup"`, `"Math"`, `"Decision"`).
    pub group: Option<String>,
    pub example: Option<Value>,
    /// All calling conventions, in spec order.
    pub overloads: Vec<MirExpressionOverload>,
    /// Named parameter definitions shared across overloads.
    pub parameters: Vec<MirExpressionParam>,
}

// ── DecodedOverload (one calling convention) ────────────────────────────────────────

/// One fully-typed calling convention for an operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressionOverload {
    /// Resolved, typed parameter sequence for this calling convention.
    pub params: MirOverloadParams,
    /// The type this calling convention produces.
    pub output: MirExprType,
}

/// The structure of an overload's parameter sequence.
///
/// Three shapes occur in the spec:
///
/// 1. **Fixed** — every parameter is required; arity is constant.
/// 2. **WithOptional** — a required prefix followed by optional trailing parameters
///    (parameter names suffixed with `?` in the spec).
/// 3. **Variadic** — a fixed prefix, a repeating unit (pairs, triples …), and a fixed
///    suffix. Covers both explicit `"..."` separators and template-expansion variadics
///    (e.g. `val_1`, `val_2`, … where `val_i` is the template definition).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirOverloadParams {
    /// All parameters positional and required; arity is fixed.
    Fixed(Vec<MirResolvedParam>),
    /// A required prefix followed by optional trailing parameters.
    WithOptional {
        required: Vec<MirResolvedParam>,
        optional: Vec<MirResolvedParam>,
    },
    /// A fixed prefix, a unit that repeats arbitrarily, and a fixed suffix.
    Variadic {
        /// Parameters before the repeating section.
        prefix: Vec<MirResolvedParam>,
        /// Smallest repeating unit (one or two elements depending on the operator).
        repeating: Vec<MirResolvedParam>,
        /// Parameters after the repeating section (e.g. the fallback in `case`).
        suffix: Vec<MirResolvedParam>,
    },
}

// ── Parameter types ────────────────────────────────────────────────────────────

/// A resolved overload parameter: canonical name, resolved type, and optional docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirResolvedParam {
    /// The parameter reference name as it appears in the overload (e.g. `"cond_1"`,
    /// `"val_i"`, `"input?"`). Preserves the original spec form for use in code gen.
    pub name: String,
    pub r#type: MirExprParamType,
    pub doc: Option<String>,
}

/// A named parameter definition (the spec's `Parameter` lifted to MIR types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirExpressionParam {
    pub name: String,
    pub r#type: MirExprParamType,
    pub doc: Option<String>,
}

// ── Type system ───────────────────────────────────────────────────────────────

/// The type of an expression parameter.
///
/// This is recursive: parameters can themselves be sub-expressions, typed arrays,
/// or inline object schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirExprParamType {
    /// A literal JSON value (not a sub-expression — a bare constant in the array).
    Literal(MirLiteralKind),
    /// One of several allowed literal kinds.
    LiteralAnyOf(Vec<MirLiteralKind>),
    /// A sub-expression whose output must be of the given type.
    Expression(MirExprType),
    /// A sub-expression whose output may be any of the given types.
    ExpressionAnyOf(Vec<MirExprParamType>),
    /// An inline object with a fixed field schema (used by `format` / `image` args).
    InlineObject(BTreeMap<String, MirExprParamType>),
    /// A polymorphic type variable — `"T"` means "same type as the evaluation context".
    TypeVar(String),
}

/// The output type of an expression, or the type constraint on an expression parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MirExprType {
    Any,
    Boolean,
    Number,
    String,
    Collator,
    Formatted,
    Image,
    Object,
    Color,
    /// A typed or untyped array, with an optional element type and fixed length.
    Array {
        element: Option<Box<MirExprParamType>>,
        length: Option<usize>,
    },
    /// The interpolation method type returned by `linear`, `exponential`, `cubic-bezier`.
    Interpolation,
    /// A polymorphic type variable — the output type depends on the input context.
    TypeVar(String),
}

/// A literal JSON value kind (a bare constant — not a sub-expression).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirLiteralKind {
    Number,
    String,
    GeoJSONObject,
    JSONObject,
    JSONArray,
}

// ── From: Literal → MirLiteralKind ──────────────────────────────────────────────

impl From<&Literal> for MirLiteralKind {
    fn from(l: &Literal) -> Self {
        match l {
            Literal::Number => MirLiteralKind::Number,
            Literal::String => MirLiteralKind::String,
            Literal::GeoJSONObject => MirLiteralKind::GeoJSONObject,
            Literal::JSONObject => MirLiteralKind::JSONObject,
            Literal::JSONArray => MirLiteralKind::JSONArray,
        }
    }
}

// ── From: decoder::Expression → MirExprType ─────────────────────────────────────

impl From<&decoder::r#enum::Expression> for MirExprType {
    fn from(e: &decoder::r#enum::Expression) -> Self {
        use decoder::r#enum::Expression as DE;
        match e {
            DE::Any => MirExprType::Any,
            DE::Boolean => MirExprType::Boolean,
            DE::Number => MirExprType::Number,
            DE::String => MirExprType::String,
            DE::Collator => MirExprType::Collator,
            DE::Formatted => MirExprType::Formatted,
            DE::Image => MirExprType::Image,
            DE::Object => MirExprType::Object,
            DE::Color => MirExprType::Color,
            DE::Array { r#type, length } => MirExprType::Array {
                element: r#type
                    .as_ref()
                    .map(|pt| Box::new(MirExprParamType::from(pt))),
                length: *length,
            },
        }
    }
}

// ── From: ParameterType → MirExprType (for overload output_type) ─────────────────

impl From<&ParameterType> for MirExprType {
    fn from(pt: &ParameterType) -> Self {
        match pt {
            ParameterType::Expression(e) => MirExprType::from(e.as_ref()),
            ParameterType::Reference(r) => expr_type_from_name(r.as_str()),
            // ExpressionAnyOf as an output type collapses to Any — unusual in the spec.
            ParameterType::ExpressionAnyOf(_) => MirExprType::Any,
            // Literals and objects as output types are edge cases; treat as Any.
            ParameterType::Literal(_) | ParameterType::LiteralAnyOf(_) => MirExprType::Any,
            ParameterType::Object(_) => MirExprType::Object,
        }
    }
}

// ── From: ParameterType → MirExprParamType ──────────────────────────────────────

impl From<&ParameterType> for MirExprParamType {
    fn from(pt: &ParameterType) -> Self {
        match pt {
            ParameterType::Literal(l) => MirExprParamType::Literal(MirLiteralKind::from(l)),
            ParameterType::LiteralAnyOf(ls) => {
                MirExprParamType::LiteralAnyOf(ls.iter().map(MirLiteralKind::from).collect())
            }
            ParameterType::Expression(e) => {
                MirExprParamType::Expression(MirExprType::from(e.as_ref()))
            }
            ParameterType::ExpressionAnyOf(pts) => {
                MirExprParamType::ExpressionAnyOf(pts.iter().map(MirExprParamType::from).collect())
            }
            ParameterType::Object(obj) => MirExprParamType::InlineObject(
                obj.iter()
                    .map(|(k, v)| (k.clone(), parsed_item_to_param_type(v)))
                    .collect(),
            ),
            ParameterType::Reference(r) => MirExprParamType::TypeVar(r.clone()),
        }
    }
}

// ── From: Parameter → MirExpressionParam ────────────────────────────────────────

impl From<&Parameter> for MirExpressionParam {
    fn from(p: &Parameter) -> Self {
        MirExpressionParam {
            name: p.name.clone(),
            r#type: MirExprParamType::from(&p.r#type),
            doc: p.doc.clone(),
        }
    }
}

// ── Constructor ───────────────────────────────────────────────────────────────

impl MirExpressions {
    /// Returns the logical negation of a comparison operator if the negated operator exists in MIR.
    ///
    /// e.g. `"=="` → `Some("!=")`, `"<"` → `Some(">=")`
    pub fn negation_of(&self, op: &str) -> Option<&'static str> {
        let (neg, check) = match op {
            "==" => ("!=", "!="),
            "!=" => ("==", "=="),
            "<" => (">=", ">="),
            "<=" => (">", ">"),
            ">" => ("<=", "<="),
            ">=" => ("<", "<"),
            _ => return None,
        };
        if self.operators.contains_key(check) {
            Some(neg)
        } else {
            None
        }
    }

    /// Construct from the raw `expression_name` top-level item.
    ///
    /// All operators are taken from the `expression_name` DecodedSyntaxEnum.
    /// Operators whose overloads have `output-type: "T"` will be stored with
    /// [`MirExprType::TypeVar`]`("T")` — polymorphism is **not** expanded here.
    pub fn build_operators(
        expression_name: &DecodedTopLevelItem,
    ) -> BTreeMap<String, MirExpressionOperator> {
        let syntax_map = extract_syntax_enum(expression_name);
        syntax_map
            .iter()
            .map(|(name, syntax_enum)| {
                (
                    name.clone(),
                    MirExpressionOperator::from_syntax_enum(syntax_enum),
                )
            })
            .collect()
    }
}

impl MirExpressionOperator {
    /// Whether this operator's output depends only on its arguments (no camera, feature, or
    /// state dependency), making it safe to constant-fold when all inputs are literals.
    pub fn is_pure(&self) -> bool {
        matches!(
            self.group.as_deref(),
            Some("Math" | "String" | "Type" | "Color")
        )
    }

    /// Convert a single `DecodedSyntaxEnum` entry from the decoder into a fully resolved MIR operator.
    pub fn from_syntax_enum(s: &DecodedSyntaxEnum) -> Self {
        let parameters: Vec<MirExpressionParam> = s
            .syntax
            .parameters
            .iter()
            .map(MirExpressionParam::from)
            .collect();
        let overloads = s
            .syntax
            .overloads
            .iter()
            .map(|o| resolve_overload(o, &s.syntax.parameters))
            .collect();
        MirExpressionOperator {
            doc: s.doc.clone(),
            group: s.group.clone(),
            example: s.example.clone(),
            overloads,
            parameters,
        }
    }
}

// ── DecodedOverload resolution ───────────────────────────────────────────────────────

/// Fully resolve one overload into a typed [`MirExpressionOverload`].
///
/// Determines the calling convention shape (Fixed / WithOptional / Variadic) and
/// resolves every parameter-name reference to the matching [`Parameter`] definition.
fn resolve_overload(overload: &DecodedOverload, params: &[Parameter]) -> MirExpressionOverload {
    let output = MirExprType::from(&overload.output_type);
    let call_params = resolve_overload_params(overload, params);
    MirExpressionOverload {
        params: call_params,
        output,
    }
}

fn resolve_overload_params(overload: &DecodedOverload, params: &[Parameter]) -> MirOverloadParams {
    let has_explicit_variadic = overload.parameters.iter().any(|p| p == "...");
    let is_variadic = overload.is_variadic(params);
    let has_optional = !has_explicit_variadic
        && !is_variadic
        && overload.parameters.iter().any(|p| p.ends_with('?'));

    if has_explicit_variadic {
        resolve_explicit_variadic(overload, params)
    } else if is_variadic {
        resolve_template_variadic(overload, params)
    } else if has_optional {
        let mut required = Vec::new();
        let mut optional = Vec::new();
        for p in &overload.parameters {
            let resolved = resolve_param_ref(p, params);
            if p.ends_with('?') {
                optional.push(resolved);
            } else {
                required.push(resolved);
            }
        }
        MirOverloadParams::WithOptional { required, optional }
    } else {
        MirOverloadParams::Fixed(
            overload
                .parameters
                .iter()
                .map(|p| resolve_param_ref(p, params))
                .collect(),
        )
    }
}

/// Resolve an overload that contains an explicit `"..."` variadic separator.
///
/// The parameters before `"..."` are split into:
/// - **prefix** — non-template positional parameters (matched by exact name).
/// - **repeating** — the template unit (parameters matched via `_i` naming).
///
/// Parameters after `"..."` form the **suffix** (e.g. the fallback in `case`).
fn resolve_explicit_variadic(
    overload: &DecodedOverload,
    params: &[Parameter],
) -> MirOverloadParams {
    let dot_pos = overload.position_of_variadic_separator();
    let before_dot = &overload.parameters[..dot_pos];
    let after_dot = &overload.parameters[dot_pos + 1..];

    let (prefix_refs, template_refs) = partition_prefix_and_template(before_dot, params);

    let prefix = prefix_refs
        .iter()
        .map(|p| resolve_param_ref(p, params))
        .collect();
    let repeating = deduplicate_template_params(&template_refs, params);
    let suffix = after_dot
        .iter()
        .map(|p| resolve_param_ref(p, params))
        .collect();

    MirOverloadParams::Variadic {
        prefix,
        repeating,
        suffix,
    }
}

/// Resolve an overload that is variadic through template expansion without `"..."`.
///
/// This happens when the overload references names like `val_3`, `val_4` that
/// can only match a `val_i` template parameter (since only `_1`/`_2` are defined).
fn resolve_template_variadic(
    overload: &DecodedOverload,
    params: &[Parameter],
) -> MirOverloadParams {
    let (prefix_refs, template_refs) = partition_prefix_and_template(&overload.parameters, params);
    let prefix = prefix_refs
        .iter()
        .map(|p| resolve_param_ref(p, params))
        .collect();
    let repeating = deduplicate_template_params(&template_refs, params);
    MirOverloadParams::Variadic {
        prefix,
        repeating,
        suffix: Vec::new(),
    }
}

/// Split a parameter reference list into a non-template prefix and a template section.
///
/// A parameter reference is considered a "template instance" when it can only be
/// matched by a `_i`-suffixed parameter definition (i.e. the spec's template params).
/// The prefix contains all non-template refs that appear before the first template ref.
fn partition_prefix_and_template<'a>(
    param_refs: &'a [String],
    params: &[Parameter],
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut prefix = Vec::new();
    let mut template = Vec::new();
    let mut in_template = false;

    for p in param_refs {
        let base = p.strip_suffix('?').unwrap_or(p.as_str());
        let is_template_instance = params
            .iter()
            .any(|param| param.name.ends_with("_i") && param.matches_overload_parameter_name(base));
        if is_template_instance {
            in_template = true;
            template.push(p.as_str());
        } else if !in_template {
            prefix.push(p.as_str());
        }
        // Non-template entries after a template section are handled as suffix (caller's job).
    }

    (prefix, template)
}

/// Deduplicate a list of template-instance parameter references into the canonical
/// `_i` definitions, preserving order and without repeating the same template param.
fn deduplicate_template_params(
    template_refs: &[&str],
    params: &[Parameter],
) -> Vec<MirResolvedParam> {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut result = Vec::new();

    for ref_name in template_refs {
        let base = ref_name.strip_suffix('?').unwrap_or(ref_name);
        if let Some(param) = params
            .iter()
            .find(|p| p.matches_overload_parameter_name(base))
            && seen.insert(param.name.clone())
        {
            result.push(MirResolvedParam {
                name: param.name.clone(),
                r#type: MirExprParamType::from(&param.r#type),
                doc: param.doc.clone(),
            });
        }
    }

    result
}

/// Look up a single parameter reference name (possibly `?`-suffixed, a template
/// instance like `val_1` / `val_2`, or a template tail name like `val_n`) in the
/// named parameter list and return a resolved entry.
///
/// Lookup order:
/// 1. Exact match / `_1` / `_2` / optional suffix via `matches_overload_parameter_name`.
/// 2. Fallback: strip the last `_<suffix>` and look for the corresponding `_i` template
///    (handles `var_name_n`, `var_value_n`, and other non-standard suffixes used in
///    the suffix position after `"..."`).
///
/// Panics if no matching parameter is found — indicates a malformed spec entry.
fn resolve_param_ref(param_ref: &str, params: &[Parameter]) -> MirResolvedParam {
    let base = param_ref.strip_suffix('?').unwrap_or(param_ref);

    // Primary lookup.
    if let Some(param) = params
        .iter()
        .find(|p| p.matches_overload_parameter_name(base))
    {
        return MirResolvedParam {
            name: param_ref.to_string(),
            r#type: MirExprParamType::from(&param.r#type),
            doc: param.doc.clone(),
        };
    }

    // Fallback: treat names like `var_name_n` as instances of the `var_name_i` template.
    if let Some(param) = find_template_param_by_suffix(base, params) {
        return MirResolvedParam {
            name: param_ref.to_string(),
            r#type: MirExprParamType::from(&param.r#type),
            doc: param.doc.clone(),
        };
    }

    panic!("overload parameter '{param_ref}' not found in parameter definitions")
}

/// Find a template parameter (containing `_i` as suffix or mid-name) that
/// matches the concrete name `name` (e.g. `var_name_n` → `var_name_i`,
/// `stop_n_input` → `stop_i_input`).
fn find_template_param_by_suffix<'a>(name: &str, params: &'a [Parameter]) -> Option<&'a Parameter> {
    params
        .iter()
        .find(|p| p.matches_overload_parameter_name(name))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map a string type name (as found in `ParameterType::Reference`) to [`MirExprType`].
fn expr_type_from_name(name: &str) -> MirExprType {
    match name {
        "any" => MirExprType::Any,
        "boolean" => MirExprType::Boolean,
        "number" => MirExprType::Number,
        "string" => MirExprType::String,
        "collator" => MirExprType::Collator,
        "formatted" => MirExprType::Formatted,
        "image" => MirExprType::Image,
        "object" => MirExprType::Object,
        "color" => MirExprType::Color,
        "interpolation" => MirExprType::Interpolation,
        other => MirExprType::TypeVar(other.to_string()),
    }
}

/// Convert a [`decoder::DecodedParsedItem`] to an [`MirExprParamType`].
///
/// Used for inline-object parameter schemas (e.g. `format` style-override arguments).
/// Reference items are converted via their type name string; primitive items are
/// mapped to their closest expression-type equivalent.
fn parsed_item_to_param_type(item: &decoder::DecodedParsedItem) -> MirExprParamType {
    use decoder::DecodedPrimitiveType;
    match item {
        decoder::DecodedParsedItem::Reference { references, .. } => {
            MirExprParamType::Expression(expr_type_from_name(references.as_str()))
        }
        decoder::DecodedParsedItem::Primitive(p) => match p {
            DecodedPrimitiveType::Boolean { .. } => {
                MirExprParamType::Expression(MirExprType::Boolean)
            }
            DecodedPrimitiveType::Number { .. } => {
                MirExprParamType::Expression(MirExprType::Number)
            }
            DecodedPrimitiveType::String { .. } => {
                MirExprParamType::Expression(MirExprType::String)
            }
            DecodedPrimitiveType::Color { .. } => MirExprParamType::Expression(MirExprType::Color),
            DecodedPrimitiveType::Formatted { .. } => {
                MirExprParamType::Expression(MirExprType::Formatted)
            }
            DecodedPrimitiveType::ResolvedImage { .. } => {
                MirExprParamType::Expression(MirExprType::Image)
            }
            DecodedPrimitiveType::Array { .. } => {
                MirExprParamType::Expression(MirExprType::Array {
                    element: None,
                    length: None,
                })
            }
            _ => MirExprParamType::TypeVar(String::from("_")),
        },
    }
}

/// Extract the `DecodedSyntaxEnum` map from an `expression_name` [`DecodedTopLevelItem`].
fn extract_syntax_enum(item: &DecodedTopLevelItem) -> &BTreeMap<String, DecodedSyntaxEnum> {
    item.as_item().as_primitive().as_enum().0.as_syntax_enum()
}
