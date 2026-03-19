use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::decoder;
use crate::decoder::TopLevelItem;
use crate::decoder::r#enum::{Literal, Overload, Parameter, ParameterType, SyntaxEnum};
use crate::mir::types::SyntaxVariantDef;

// ── Generator-facing view (unchanged) ─────────────────────────────────────────

/// All expression operators that produce a specific output type.
/// Used by the code generator to emit per-output-type enums.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionGroup {
    /// Keyed by the expression operator name (e.g. `"literal"`, `"interpolate"`).
    pub variants: BTreeMap<String, SyntaxVariantDef>,
}

// ── Top-level container ────────────────────────────────────────────────────────

/// The complete expression system lifted from the spec.
///
/// Two complementary views of the same data are provided:
///
/// - `by_output_type` — the code-generator view: operators grouped by their output type
///   with T expanded into every concrete type (same structure the generator emits enums from).
/// - `operators` — the optimizer/analysis view: all operators flat, with fully resolved
///   parameter types and polymorphism preserved via [`ExprType::TypeVar`] rather than
///   being expanded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateExpressions {
    /// Code-generator view: operators grouped by output-type name (T expanded).
    pub by_output_type: BTreeMap<String, ExpressionGroup>,
    /// Optimizer view: all operators keyed by spec name, T-polymorphism preserved.
    pub operators: BTreeMap<String, ExpressionOperator>,
}

// ── Per-operator ──────────────────────────────────────────────────────────────

/// One expression operator (e.g. `"case"`, `"interpolate"`, `"literal"`).
///
/// Contains fully resolved, MIR-typed calling conventions. Polymorphic operators
/// (those whose output type depends on their input) use [`ExprType::TypeVar`] rather
/// than being expanded into one copy per concrete type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionOperator {
    pub doc: String,
    /// Grouping hint from the spec (e.g. `"Lookup"`, `"Math"`, `"Decision"`).
    pub group: Option<String>,
    pub example: Option<Value>,
    /// All calling conventions, in spec order.
    pub overloads: Vec<ExpressionOverload>,
    /// Named parameter definitions shared across overloads.
    pub parameters: Vec<ExpressionParam>,
}

// ── Overload (one calling convention) ────────────────────────────────────────

/// One fully-typed calling convention for an operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionOverload {
    /// Resolved, typed parameter sequence for this calling convention.
    pub params: OverloadParams,
    /// The type this calling convention produces.
    pub output: ExprType,
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
pub enum OverloadParams {
    /// All parameters positional and required; arity is fixed.
    Fixed(Vec<ResolvedParam>),
    /// A required prefix followed by optional trailing parameters.
    WithOptional {
        required: Vec<ResolvedParam>,
        optional: Vec<ResolvedParam>,
    },
    /// A fixed prefix, a unit that repeats arbitrarily, and a fixed suffix.
    Variadic {
        /// Parameters before the repeating section.
        prefix: Vec<ResolvedParam>,
        /// Smallest repeating unit (one or two elements depending on the operator).
        repeating: Vec<ResolvedParam>,
        /// Parameters after the repeating section (e.g. the fallback in `case`).
        suffix: Vec<ResolvedParam>,
    },
}

// ── Parameter types ────────────────────────────────────────────────────────────

/// A resolved overload parameter: canonical name, resolved type, and optional docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedParam {
    /// The parameter reference name as it appears in the overload (e.g. `"cond_1"`,
    /// `"val_i"`, `"input?"`). Preserves the original spec form for use in code gen.
    pub name: String,
    pub r#type: ExprParamType,
    pub doc: Option<String>,
}

/// A named parameter definition (the spec's `Parameter` lifted to MIR types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionParam {
    pub name: String,
    pub r#type: ExprParamType,
    pub doc: Option<String>,
}

// ── Type system ───────────────────────────────────────────────────────────────

/// The type of an expression parameter.
///
/// This is recursive: parameters can themselves be sub-expressions, typed arrays,
/// or inline object schemas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprParamType {
    /// A literal JSON value (not a sub-expression — a bare constant in the array).
    Literal(LiteralKind),
    /// One of several allowed literal kinds.
    LiteralAnyOf(Vec<LiteralKind>),
    /// A sub-expression whose output must be of the given type.
    Expression(ExprType),
    /// A sub-expression whose output may be any of the given types.
    ExpressionAnyOf(Vec<ExprParamType>),
    /// An inline object with a fixed field schema (used by `format` / `image` args).
    InlineObject(BTreeMap<String, ExprParamType>),
    /// A polymorphic type variable — `"T"` means "same type as the evaluation context".
    TypeVar(String),
}

/// The output type of an expression, or the type constraint on an expression parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprType {
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
        element: Option<Box<ExprParamType>>,
        length: Option<usize>,
    },
    /// The interpolation method type returned by `linear`, `exponential`, `cubic-bezier`.
    Interpolation,
    /// A polymorphic type variable — the output type depends on the input context.
    TypeVar(String),
}

/// A literal JSON value kind (a bare constant — not a sub-expression).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralKind {
    Number,
    String,
    GeoJSONObject,
    JSONObject,
    JSONArray,
}

// ── From: Literal → LiteralKind ──────────────────────────────────────────────

impl From<&Literal> for LiteralKind {
    fn from(l: &Literal) -> Self {
        match l {
            Literal::Number => LiteralKind::Number,
            Literal::String => LiteralKind::String,
            Literal::GeoJSONObject => LiteralKind::GeoJSONObject,
            Literal::JSONObject => LiteralKind::JSONObject,
            Literal::JSONArray => LiteralKind::JSONArray,
        }
    }
}

// ── From: decoder::Expression → ExprType ─────────────────────────────────────

impl From<&decoder::r#enum::Expression> for ExprType {
    fn from(e: &decoder::r#enum::Expression) -> Self {
        use decoder::r#enum::Expression as DE;
        match e {
            DE::Any => ExprType::Any,
            DE::Boolean => ExprType::Boolean,
            DE::Number => ExprType::Number,
            DE::String => ExprType::String,
            DE::Collator => ExprType::Collator,
            DE::Formatted => ExprType::Formatted,
            DE::Image => ExprType::Image,
            DE::Object => ExprType::Object,
            DE::Color => ExprType::Color,
            DE::Array { r#type, length } => ExprType::Array {
                element: r#type.as_ref().map(|pt| Box::new(ExprParamType::from(pt))),
                length: *length,
            },
        }
    }
}

// ── From: ParameterType → ExprType (for overload output_type) ─────────────────

impl From<&ParameterType> for ExprType {
    fn from(pt: &ParameterType) -> Self {
        match pt {
            ParameterType::Expression(e) => ExprType::from(e.as_ref()),
            ParameterType::Reference(r) => expr_type_from_name(r.as_str()),
            // ExpressionAnyOf as an output type collapses to Any — unusual in the spec.
            ParameterType::ExpressionAnyOf(_) => ExprType::Any,
            // Literals and objects as output types are edge cases; treat as Any.
            ParameterType::Literal(_) | ParameterType::LiteralAnyOf(_) => ExprType::Any,
            ParameterType::Object(_) => ExprType::Object,
        }
    }
}

// ── From: ParameterType → ExprParamType ──────────────────────────────────────

impl From<&ParameterType> for ExprParamType {
    fn from(pt: &ParameterType) -> Self {
        match pt {
            ParameterType::Literal(l) => ExprParamType::Literal(LiteralKind::from(l)),
            ParameterType::LiteralAnyOf(ls) => {
                ExprParamType::LiteralAnyOf(ls.iter().map(LiteralKind::from).collect())
            }
            ParameterType::Expression(e) => ExprParamType::Expression(ExprType::from(e.as_ref())),
            ParameterType::ExpressionAnyOf(pts) => {
                ExprParamType::ExpressionAnyOf(pts.iter().map(ExprParamType::from).collect())
            }
            ParameterType::Object(obj) => ExprParamType::InlineObject(
                obj.iter()
                    .map(|(k, v)| (k.clone(), parsed_item_to_param_type(v)))
                    .collect(),
            ),
            ParameterType::Reference(r) => ExprParamType::TypeVar(r.clone()),
        }
    }
}

// ── From: Parameter → ExpressionParam ────────────────────────────────────────

impl From<&Parameter> for ExpressionParam {
    fn from(p: &Parameter) -> Self {
        ExpressionParam {
            name: p.name.clone(),
            r#type: ExprParamType::from(&p.r#type),
            doc: p.doc.clone(),
        }
    }
}

// ── Constructor ───────────────────────────────────────────────────────────────

impl IntermediateExpressions {
    /// Construct from the raw `expression_name` top-level item.
    ///
    /// All operators are taken from the `expression_name` SyntaxEnum.
    /// Operators whose overloads have `output-type: "T"` will be stored with
    /// [`ExprType::TypeVar`]`("T")` — polymorphism is **not** expanded here.
    pub fn build_operators(expression_name: &TopLevelItem) -> BTreeMap<String, ExpressionOperator> {
        let syntax_map = extract_syntax_enum(expression_name);
        syntax_map
            .iter()
            .map(|(name, syntax_enum)| {
                (
                    name.clone(),
                    ExpressionOperator::from_syntax_enum(syntax_enum),
                )
            })
            .collect()
    }
}

impl ExpressionOperator {
    /// Convert a single `SyntaxEnum` entry from the decoder into a fully resolved MIR operator.
    pub fn from_syntax_enum(s: &SyntaxEnum) -> Self {
        let parameters: Vec<ExpressionParam> = s
            .syntax
            .parameters
            .iter()
            .map(ExpressionParam::from)
            .collect();
        let overloads = s
            .syntax
            .overloads
            .iter()
            .map(|o| resolve_overload(o, &s.syntax.parameters))
            .collect();
        ExpressionOperator {
            doc: s.doc.clone(),
            group: s.group.clone(),
            example: s.example.clone(),
            overloads,
            parameters,
        }
    }
}

// ── Overload resolution ───────────────────────────────────────────────────────

/// Fully resolve one overload into a typed [`ExpressionOverload`].
///
/// Determines the calling convention shape (Fixed / WithOptional / Variadic) and
/// resolves every parameter-name reference to the matching [`Parameter`] definition.
fn resolve_overload(overload: &Overload, params: &[Parameter]) -> ExpressionOverload {
    let output = ExprType::from(&overload.output_type);
    let call_params = resolve_overload_params(overload, params);
    ExpressionOverload {
        params: call_params,
        output,
    }
}

fn resolve_overload_params(overload: &Overload, params: &[Parameter]) -> OverloadParams {
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
        OverloadParams::WithOptional { required, optional }
    } else {
        OverloadParams::Fixed(
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
fn resolve_explicit_variadic(overload: &Overload, params: &[Parameter]) -> OverloadParams {
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

    OverloadParams::Variadic {
        prefix,
        repeating,
        suffix,
    }
}

/// Resolve an overload that is variadic through template expansion without `"..."`.
///
/// This happens when the overload references names like `val_3`, `val_4` that
/// can only match a `val_i` template parameter (since only `_1`/`_2` are defined).
fn resolve_template_variadic(overload: &Overload, params: &[Parameter]) -> OverloadParams {
    let (prefix_refs, template_refs) = partition_prefix_and_template(&overload.parameters, params);
    let prefix = prefix_refs
        .iter()
        .map(|p| resolve_param_ref(p, params))
        .collect();
    let repeating = deduplicate_template_params(&template_refs, params);
    OverloadParams::Variadic {
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
fn deduplicate_template_params(template_refs: &[&str], params: &[Parameter]) -> Vec<ResolvedParam> {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut result = Vec::new();

    for ref_name in template_refs {
        let base = ref_name.strip_suffix('?').unwrap_or(ref_name);
        if let Some(param) = params
            .iter()
            .find(|p| p.matches_overload_parameter_name(base))
            && seen.insert(param.name.clone())
        {
            result.push(ResolvedParam {
                name: param.name.clone(),
                r#type: ExprParamType::from(&param.r#type),
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
fn resolve_param_ref(param_ref: &str, params: &[Parameter]) -> ResolvedParam {
    let base = param_ref.strip_suffix('?').unwrap_or(param_ref);

    // Primary lookup.
    if let Some(param) = params
        .iter()
        .find(|p| p.matches_overload_parameter_name(base))
    {
        return ResolvedParam {
            name: param_ref.to_string(),
            r#type: ExprParamType::from(&param.r#type),
            doc: param.doc.clone(),
        };
    }

    // Fallback: treat names like `var_name_n` as instances of the `var_name_i` template.
    if let Some(param) = find_template_param_by_suffix(base, params) {
        return ResolvedParam {
            name: param_ref.to_string(),
            r#type: ExprParamType::from(&param.r#type),
            doc: param.doc.clone(),
        };
    }

    panic!("overload parameter '{param_ref}' not found in parameter definitions")
}

/// Try to match `name` (e.g. `var_name_n`) against a `_i`-suffixed template parameter
/// (e.g. `var_name_i`) by stripping the last `_<suffix>` and appending `_i`.
fn find_template_param_by_suffix<'a>(name: &str, params: &'a [Parameter]) -> Option<&'a Parameter> {
    let last_underscore = name.rfind('_')?;
    let base = &name[..last_underscore];
    let template_name = format!("{base}_i");
    params.iter().find(|p| p.name == template_name)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map a string type name (as found in `ParameterType::Reference`) to [`ExprType`].
fn expr_type_from_name(name: &str) -> ExprType {
    match name {
        "any" => ExprType::Any,
        "boolean" => ExprType::Boolean,
        "number" => ExprType::Number,
        "string" => ExprType::String,
        "collator" => ExprType::Collator,
        "formatted" => ExprType::Formatted,
        "image" => ExprType::Image,
        "object" => ExprType::Object,
        "color" => ExprType::Color,
        "interpolation" => ExprType::Interpolation,
        other => ExprType::TypeVar(other.to_string()),
    }
}

/// Convert a [`decoder::ParsedItem`] to an [`ExprParamType`].
///
/// Used for inline-object parameter schemas (e.g. `format` style-override arguments).
/// Reference items are converted via their type name string; primitive items are
/// mapped to their closest expression-type equivalent.
fn parsed_item_to_param_type(item: &decoder::ParsedItem) -> ExprParamType {
    use decoder::PrimitiveType;
    match item {
        decoder::ParsedItem::Reference { references, .. } => {
            ExprParamType::Expression(expr_type_from_name(references.as_str()))
        }
        decoder::ParsedItem::Primitive(p) => match p {
            PrimitiveType::Boolean { .. } => ExprParamType::Expression(ExprType::Boolean),
            PrimitiveType::Number { .. } => ExprParamType::Expression(ExprType::Number),
            PrimitiveType::String { .. } => ExprParamType::Expression(ExprType::String),
            PrimitiveType::Color { .. } => ExprParamType::Expression(ExprType::Color),
            PrimitiveType::Formatted { .. } => ExprParamType::Expression(ExprType::Formatted),
            PrimitiveType::ResolvedImage { .. } => ExprParamType::Expression(ExprType::Image),
            PrimitiveType::Array { .. } => ExprParamType::Expression(ExprType::Array {
                element: None,
                length: None,
            }),
            _ => ExprParamType::TypeVar(String::from("_")),
        },
    }
}

/// Extract the `SyntaxEnum` map from an `expression_name` [`TopLevelItem`].
fn extract_syntax_enum(item: &TopLevelItem) -> &BTreeMap<String, SyntaxEnum> {
    item.as_item().as_primitive().as_enum().0.as_syntax_enum()
}
