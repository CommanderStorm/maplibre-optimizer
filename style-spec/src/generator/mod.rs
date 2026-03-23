use std::collections::BTreeMap;

use codegen2::Scope;

use crate::generator::formatter::{to_snake_case, to_upper_camel_case};
use crate::generator::literals::generate_literals;
use crate::mir::types::{
    MirArrayElement, MirArrayElementType, MirBooleanField, MirColorArrayField, MirColorField,
    MirEnum, MirEnumField, MirField, MirFieldMeta, MirFormattedTextField, MirNumberArrayField,
    MirNumberField, MirPaddingField, MirProjectionDefinitionField, MirRegularEnum,
    MirRegularVariant, MirResolvedImageField, MirStateField, MirStringField, MirType,
};
use crate::mir::{
    MirExpressions, MirLayerField, MirLayers, MirNamedType, MirOneOf, MirSources, MirSpec,
};

mod autotest;
pub mod formatter;
pub(crate) mod fuzz;
mod items;
mod literals;
pub(crate) mod untagged;

/// Generate Rust source from the semantic MIR.
/// This is the sole entry point; it never touches decoder types.
pub fn generate_spec_scope(spec: &MirSpec) -> String {
    let mut scope = Scope::new();

    // Expression syntax tuples reference literal newtypes (`StringLiteral`, …).
    generate_literals(&mut scope);
    generate_root_struct(&mut scope, spec);

    // Named types (groups, type aliases, OneOf enums)
    for (key, named_type) in &spec.named_types {
        let name = to_upper_camel_case(key);
        generate_named_type(&mut scope, &name, named_type);
    }

    // Expression syntax enums (per-output-type)
    generate_expression_types(&mut scope, &spec.expressions);

    // Source struct types
    generate_source_types(&mut scope, &spec.sources);

    // Layer struct types
    generate_layer_types(&mut scope, &spec.layers);

    scope
        .get_or_new_module("test")
        .attr("cfg(test)")
        .import("super", "*");

    scope.to_string()
}

/// Generate the Maplibre Style Specification types split into per-domain modules.
///
/// This returns an *outer* [`codegen2::Scope`] containing one child module per domain
/// (`literals`, `root`, `named_types`, `expressions`, `sources`, `layers`).
/// Each domain module contains its own `#[cfg(test)] mod test { ... }` for the
/// generated `test_example_*_decodes` checks.
pub fn generate_spec_modules(spec: &MirSpec) -> Scope {
    let mut outer = Scope::new();

    let domains = [
        "literals",
        "root",
        "named_types",
        "expressions",
        "sources",
        "layers",
    ];

    // Pre-create modules so we can unconditionally finalize the `test` submodule later.
    for d in domains {
        outer.get_or_new_module(d);
    }

    {
        let literals_scope = outer.get_or_new_module("literals").scope();
        generate_literals(literals_scope);
        literals_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    {
        let root_scope = outer.get_or_new_module("root").scope();
        generate_root_struct(root_scope, spec);
        root_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    {
        let named_types_scope = outer.get_or_new_module("named_types").scope();
        for (key, named_type) in &spec.named_types {
            let name = to_upper_camel_case(key);
            generate_named_type(named_types_scope, &name, named_type);
        }
        named_types_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    {
        let expressions_scope = outer.get_or_new_module("expressions").scope();
        generate_expression_types(expressions_scope, &spec.expressions);
        expressions_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    {
        let sources_scope = outer.get_or_new_module("sources").scope();
        generate_source_types(sources_scope, &spec.sources);
        sources_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    {
        let layers_scope = outer.get_or_new_module("layers").scope();
        generate_layer_types(layers_scope, &spec.layers);
        layers_scope
            .get_or_new_module("test")
            .attr("cfg(test)")
            .attr("allow(unused_imports)")
            .import("super", "*");
    }

    outer
}

// ── Root struct ───────────────────────────────────────────────────────────────

fn generate_root_struct(scope: &mut Scope, spec: &MirSpec) {
    let s = scope
        .new_struct("MaplibreStyleSpecification")
        .doc("This is a Maplibre Style Specification")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    for (key, field) in &spec.root.0 {
        let meta = field.meta();
        let type_name = to_upper_camel_case(format!("root {key}"));
        let mut field_type = type_name.clone();
        if meta.optional {
            field_type = format!("Option<{field_type}>");
        }
        let sf = s
            .new_field(&meta.rust_name, field_type)
            .vis("pub")
            .doc(&meta.doc);
        if &meta.rust_name != key {
            sf.annotation(format!("#[serde(rename=\"{key}\")]"));
        }
        if meta.optional {
            sf.annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
        }
    }

    // Add `layers` field — removed from root during MIR preprocessing but needed for typed access.
    if !spec.layers.layer_types.is_empty() {
        s.new_field("layers", "Vec<AnyLayer>")
            .vis("pub")
            .doc("Layers will be drawn in the order of this array.")
            .annotation("#[serde(default)]");
    }

    // Generate subtypes for each root field
    for (key, field) in &spec.root.0 {
        let type_name = to_upper_camel_case(format!("root {key}"));
        generate_mir_type(scope, &type_name, field);
    }
}

// ── Named types ───────────────────────────────────────────────────────────────

fn generate_named_type(scope: &mut Scope, name: &str, named_type: &MirNamedType) {
    match named_type {
        MirNamedType::Struct(fields) => generate_struct_from_fields(scope, name, fields),
        MirNamedType::TypeDef(field) => generate_mir_type(scope, name, field),
        MirNamedType::OneOf(one_of) => generate_oneof(scope, name, one_of),
    }
}

/// Generate a named struct from a slice of MIR fields.
/// Handles the single-star (`*`) wildcard field as a BTreeMap wrapper.
fn generate_struct_from_fields(scope: &mut Scope, name: &str, fields: &[MirField]) {
    // Special case: single-star field → BTreeMap wrapper
    if fields.len() == 1
        && let MirField::Star(meta) = &fields[0]
    {
        let inner_name = to_upper_camel_case(format!("Inner {name}"));
        scope
            .new_struct(name)
            .vis("pub")
            .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
            .attr(fuzz::CFG_DERIVE_ARBITRARY)
            .tuple_field(format!(
                "std::collections::BTreeMap<std::string::String,{inner_name}>"
            ));
        items::star::generate(scope, &inner_name, meta);
        return;
    }

    let s = scope
        .new_struct(name)
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    for field in fields {
        let meta = field.meta();
        let field_type_name = to_upper_camel_case(format!("{name} {}", meta.spec_name));
        let mut field_type = if meta.spec_name == "*" {
            format!("std::collections::BTreeMap<std::string::String,{field_type_name}>")
        } else {
            field_type_name.clone()
        };
        if meta.optional {
            field_type = format!("Option<{field_type}>");
        }
        let sf = s
            .new_field(&meta.rust_name, field_type)
            .vis("pub")
            .doc(&meta.doc);
        if meta.spec_name == "*" {
            sf.annotation("#[serde(flatten)]");
        } else if meta.rust_name != meta.spec_name.as_str() {
            sf.annotation(format!("#[serde(rename=\"{}\")]", meta.spec_name));
        }
        if meta.optional {
            sf.annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
        }
    }

    // Generate subtypes for each field (including `*` wildcard keys — e.g. promoteId groups).
    for field in fields {
        let meta = field.meta();
        let field_type_name = to_upper_camel_case(format!("{name} {}", meta.spec_name));
        generate_mir_type(scope, &field_type_name, field);
    }
}

/// Generate a `#[serde(tag)]` or untagged (custom visitor) sum-type enum.
fn generate_oneof(scope: &mut Scope, name: &str, one_of: &MirOneOf) {
    let is_tagged = one_of.tag.is_some();

    let derive = if is_tagged {
        "serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone"
    } else {
        "PartialEq, Debug, Clone"
    };

    let enu = scope
        .new_enum(name)
        .vis("pub")
        .derive(derive)
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    if let Some(tag) = &one_of.tag {
        enu.attr(format!("serde(tag=\"{tag}\")"));
    }

    // `clippy::enum_variant_names`: if all variants share a postfix equal to the enum name
    // (e.g. `Source::{GeojsonSource,ImageSource,...}`), rename the Rust variants by stripping
    // that postfix. Keep serde renames keyed by the original variant type names.
    let base_var_names: Vec<String> = one_of.variants.iter().map(to_upper_camel_case).collect();
    let strip_suffix = if base_var_names.len() > 1
        && base_var_names.iter().all(|v| v.ends_with(name))
        && name.len() > 1
    {
        Some(name)
    } else {
        None
    };

    let mut variant_info: Vec<(String, String)> = Vec::new();
    for variant_key in &one_of.variants {
        let base_var_name = to_upper_camel_case(variant_key);
        let var_ident = strip_suffix
            .and_then(|s| base_var_name.strip_suffix(s))
            .unwrap_or(&base_var_name)
            .to_string();
        let var = enu.new_variant(&var_ident).tuple(&base_var_name);
        if let Some(rename) = one_of.renames.get(&base_var_name) {
            var.annotation(format!("#[serde(rename=\"{rename}\")]"));
        }
        variant_info.push((var_ident, base_var_name));
    }

    if !is_tagged {
        let variants: Vec<untagged::Variant> = variant_info
            .iter()
            .map(|(vn, vt)| untagged::Variant {
                name: vn.clone(),
                inner_type: vt.clone(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            })
            .collect();
        untagged::emit_untagged_serde(scope, name, &variants);
    }
}

// ── Expression types ──────────────────────────────────────────────────────────

fn generate_expression_types(scope: &mut Scope, expressions: &MirExpressions) {
    // ExprOrLiteral must be defined exactly once per codegen scope, before any
    // expression enum is emitted (both in the monolith and in the split-module case).
    items::r#enum::syntax::ensure_expr_or_literal_type(scope);
    for (output_type_name, group) in &expressions.by_output_type {
        items::r#enum::syntax::generate_syntax_enum(
            scope,
            output_type_name,
            &format!("{output_type_name:?}"),
            &group.variants,
        );
    }
}

// ── Source types ──────────────────────────────────────────────────────────────

fn generate_source_types(scope: &mut Scope, sources: &MirSources) {
    if sources.source_types.is_empty() {
        return;
    }

    // Generate a struct per source type
    for (type_name, def) in &sources.source_types {
        let struct_name = to_upper_camel_case(format!("{type_name} source"));
        generate_struct_from_fields(scope, &struct_name, &def.fields);
    }

    // Generate the Source sum type
    let variant_keys: Vec<String> = sources
        .source_types
        .keys()
        .map(|k| format!("{k}_source"))
        .collect();

    // Common tag field: each source group had `type` stripped in MIR preprocessing; JSON
    // discriminant matches the `source_types` map key.
    let tag = Some("type".to_string());

    // JSON discriminant uses kebab-case (e.g. `raster-dem`), but MIR keys are
    // snake_case (stripped from `source_raster_dem`).  Restore the hyphens for serde.
    let renames: BTreeMap<String, String> = sources
        .source_types
        .keys()
        .map(|k| {
            (
                to_upper_camel_case(format!("{k}_source")),
                k.replace('_', "-"),
            )
        })
        .collect();

    generate_oneof(
        scope,
        "Source",
        &MirOneOf {
            variants: variant_keys,
            tag,
            renames,
        },
    );
}

// ── Layer types ───────────────────────────────────────────────────────────────

fn generate_layer_types(scope: &mut Scope, layers: &MirLayers) {
    if layers.common_fields.is_empty() && layers.layer_types.is_empty() {
        return;
    }

    // Hand-written `LayerFilter` — a boolean expression or literal bool.
    // The `filter` field was removed from common_fields in MIR preprocessing
    // so that we can emit this typed enum instead of the spec's string type.
    generate_layer_filter(scope);

    // Common `Layer` struct — the `filter` field was removed from MIR
    // common_fields, so we generate the struct and add it manually.
    let common_mir: Vec<MirField> = layer_fields_to_mir(&layers.common_fields);
    generate_layer_struct(scope, &common_mir);

    // Per-type layout and paint structs
    for (type_key, layer_type) in &layers.layer_types {
        let layout_name = to_upper_camel_case(format!("{type_key} layout layer"));
        let paint_name = to_upper_camel_case(format!("{type_key} paint layer"));
        let layout_mir: Vec<MirField> = layer_fields_to_mir(&layer_type.layout);
        let paint_mir: Vec<MirField> = layer_fields_to_mir(&layer_type.paint);
        generate_struct_from_fields(scope, &layout_name, &layout_mir);
        generate_struct_from_fields(scope, &paint_name, &paint_mir);
    }

    // TypedLayer discriminated enum
    generate_typed_layer_enum(scope, layers);

    // RefLayer struct for layers that use "ref" instead of "type"
    generate_ref_layer(scope);

    // AnyLayer wrapper enum (typed or ref)
    generate_any_layer(scope);

    // Helper impls on TypedLayer, AnyLayer, and newtype wrappers
    generate_layer_helper_impls(scope, layers);
}

/// Generate the `LayerFilter` type.
///
/// A typed enum with custom serde: `Literal(bool)` for bare booleans and
/// `["literal", bool]` (normalised on deserialize), `Expr(Box<Boolean>)` for
/// expression arrays.  Both `Boolean::serialize` and `Boolean::deserialize`
/// round-trip via the `["op", ...]` array format, so there is no information
/// loss.
fn generate_layer_filter(scope: &mut Scope) {
    scope.raw(
        r#"
/// A filter expression: a typed boolean expression, a polymorphic Any expression
/// (`match`, `step`, `case`, …), or a literal bool.
///
/// On deserialize, bare `true`/`false` and `["literal", true/false]` are both
/// normalised to `Literal(bool)`.  On serialize, `Literal(b)` emits the bare
/// JSON boolean.
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "fuzz", derive(arbitrary::Arbitrary))]
pub enum LayerFilter {
    Expr(Box<Boolean>),
    AnyExpr(Box<Any>),
    Literal(bool),
}

impl<'de> serde::Deserialize<'de> for LayerFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(LayerFilterVisitor)
    }
}

struct LayerFilterVisitor;

impl<'de> serde::de::Visitor<'de> for LayerFilterVisitor {
    type Value = LayerFilter;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a boolean literal, a Boolean expression array, or a polymorphic expression array")
    }

    fn visit_bool<E: serde::de::Error>(self, b: bool) -> Result<Self::Value, E> {
        Ok(LayerFilter::Literal(b))
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        // Collect all elements; we need to inspect the first to detect ["literal", bool].
        let mut elements: Vec<serde_json::Value> = Vec::new();
        while let Some(elem) = seq.next_element::<serde_json::Value>()? {
            elements.push(elem);
        }
        // Normalise ["literal", true/false] → Literal(bool).
        if elements.len() == 2
            && elements[0].as_str() == Some("literal")
            && elements[1].is_boolean()
        {
            return Ok(LayerFilter::Literal(elements[1].as_bool().unwrap()));
        }
        let arr = serde_json::Value::Array(elements);
        // Try Boolean first (fixed-output-type operators like `all`, `any`, `==`, …).
        if let Ok(expr) = serde_json::from_value::<Boolean>(arr.clone()) {
            return Ok(LayerFilter::Expr(Box::new(expr)));
        }
        // Fall back to Any (polymorphic operators like `match`, `step`, `case`, …).
        let expr = serde_json::from_value::<Any>(arr).map_err(serde::de::Error::custom)?;
        Ok(LayerFilter::AnyExpr(Box::new(expr)))
    }
}

impl serde::Serialize for LayerFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            LayerFilter::Expr(expr) => expr.serialize(serializer),
            LayerFilter::AnyExpr(expr) => expr.serialize(serializer),
            LayerFilter::Literal(b) => serializer.serialize_bool(*b),
        }
    }
}
"#,
    );
}

/// Generate the common `Layer` struct with the hand-written `filter` field
/// injected alongside the MIR-generated fields.
fn generate_layer_struct(scope: &mut Scope, common_mir: &[MirField]) {
    let s = scope
        .new_struct("Layer")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    // Hand-written filter field (not in MIR because we override its type).
    s.new_field("filter", "Option<LayerFilter>")
        .vis("pub")
        .doc("A expression specifying conditions on source features. Only features that match the filter are displayed.")
        .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");

    // MIR-generated fields.
    for field in common_mir {
        let meta = field.meta();
        let field_type_name = to_upper_camel_case(format!("Layer {}", meta.spec_name));
        let mut field_type = field_type_name.clone();
        if meta.optional {
            field_type = format!("Option<{field_type}>");
        }
        let sf = s
            .new_field(&meta.rust_name, field_type)
            .vis("pub")
            .doc(&meta.doc);
        if meta.rust_name != meta.spec_name.as_str() {
            sf.annotation(format!("#[serde(rename=\"{}\")]", meta.spec_name));
        }
        if meta.optional {
            sf.annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
        }
    }

    // Generate subtypes for each MIR field.
    for field in common_mir {
        let meta = field.meta();
        let field_type_name = to_upper_camel_case(format!("Layer {}", meta.spec_name));
        generate_mir_type(scope, &field_type_name, field);
    }
}

/// Generate a `#[serde(tag = "type")]` enum dispatching on the layer type.
///
/// Each variant contains `#[serde(flatten)] common: Layer` plus optional paint/layout
/// structs for that layer type.
fn generate_typed_layer_enum(scope: &mut Scope, layers: &MirLayers) {
    let enu = scope
        .new_enum("TypedLayer")
        .doc("A style layer with its type-specific paint and layout properties.")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY)
        .attr("serde(tag = \"type\")");

    for type_key in layers.layer_types.keys() {
        let variant_name = to_upper_camel_case(type_key);
        let layout_type = to_upper_camel_case(format!("{type_key} layout layer"));
        let paint_type = to_upper_camel_case(format!("{type_key} paint layer"));

        let var = enu.new_variant(&variant_name);
        var.annotation(format!("#[serde(rename = \"{type_key}\")]"));

        var.new_named("common", "Layer")
            .annotation("#[serde(flatten)]");
        var.new_named("paint", format!("Option<{paint_type}>"))
            .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
        var.new_named("layout", format!("Option<{layout_type}>"))
            .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
    }
}

/// Generate `RefLayer` — a layer that references another layer via `"ref"` instead of `"type"`.
fn generate_ref_layer(scope: &mut Scope) {
    let s = scope
        .new_struct("RefLayer")
        .doc("A layer that inherits its type and properties from a referenced layer via `ref`.")
        .vis("pub")
        .derive("serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);

    s.new_field("id", "LayerId").vis("pub");
    s.new_field("r#ref", "std::string::String")
        .vis("pub")
        .annotation("#[serde(rename = \"ref\")]");
    s.new_field("filter", "Option<LayerFilter>")
        .vis("pub")
        .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
    s.new_field("minzoom", "Option<LayerMinzoom>")
        .vis("pub")
        .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
    s.new_field("maxzoom", "Option<LayerMaxzoom>")
        .vis("pub")
        .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
    s.new_field("metadata", "Option<LayerMetadata>")
        .vis("pub")
        .annotation("#[serde(default, skip_serializing_if = \"Option::is_none\")]");
}

/// Generate `AnyLayer` — an untagged enum wrapping `TypedLayer` or `RefLayer`.
fn generate_any_layer(scope: &mut Scope) {
    let enu = scope
        .new_enum("AnyLayer")
        .doc("A layer in the style: either a fully typed layer or a `ref` layer.")
        .vis("pub")
        .derive("PartialEq, Debug, Clone")
        .attr(fuzz::CFG_DERIVE_ARBITRARY);
    enu.new_variant("Typed").tuple("TypedLayer");
    enu.new_variant("Ref").tuple("RefLayer");
    untagged::emit_untagged_serde(
        scope,
        "AnyLayer",
        &[
            untagged::Variant {
                name: "Typed".into(),
                inner_type: "TypedLayer".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
            untagged::Variant {
                name: "Ref".into(),
                inner_type: "RefLayer".into(),
                is_boxed: false,
                is_unit: false,
                skip_when: None,
            },
        ],
    );
}

/// Emit hand-written helper impls on `TypedLayer`, `AnyLayer`, and newtype wrappers.
fn generate_layer_helper_impls(scope: &mut Scope, layers: &MirLayers) {
    // Build match arms for common() / common_mut() / layer_type()
    let mut common_arms = String::new();
    let mut type_arms = String::new();
    for type_key in layers.layer_types.keys() {
        let variant = to_upper_camel_case(type_key);
        common_arms.push_str(&format!(
            "            TypedLayer::{variant} {{ common, .. }} |\n"
        ));
        type_arms.push_str(&format!(
            "            TypedLayer::{variant} {{ .. }} => \"{type_key}\",\n"
        ));
    }
    // Remove trailing " |\n" from common_arms and replace with " => common,\n"
    if common_arms.ends_with(" |\n") {
        common_arms.truncate(common_arms.len() - 3);
        common_arms.push_str(" => common,\n");
    }

    scope.raw(format!(
        r#"
impl TypedLayer {{
    /// Access the common `Layer` fields shared by all typed layers.
    pub fn common(&self) -> &Layer {{
        match self {{
{common_arms}        }}
    }}

    /// Mutably access the common `Layer` fields.
    pub fn common_mut(&mut self) -> &mut Layer {{
        match self {{
{common_arms}        }}
    }}

    /// The layer type string as it appears in JSON (e.g. `"fill"`, `"line"`).
    pub fn layer_type(&self) -> &'static str {{
        match self {{
{type_arms}        }}
    }}
}}

impl AnyLayer {{
    /// Get the layer ID regardless of layer kind.
    pub fn id(&self) -> &LayerId {{
        match self {{
            AnyLayer::Typed(t) => &t.common().id,
            AnyLayer::Ref(r) => &r.id,
        }}
    }}

    /// Access the common `Layer` if this is a typed layer.
    pub fn common(&self) -> Option<&Layer> {{
        match self {{
            AnyLayer::Typed(t) => Some(t.common()),
            AnyLayer::Ref(_) => None,
        }}
    }}

    /// Access the common `Layer` mutably if this is a typed layer.
    pub fn common_mut(&mut self) -> Option<&mut Layer> {{
        match self {{
            AnyLayer::Typed(t) => Some(t.common_mut()),
            AnyLayer::Ref(_) => None,
        }}
    }}

    /// The effective layer type string, or `None` for ref layers.
    pub fn layer_type(&self) -> Option<&'static str> {{
        match self {{
            AnyLayer::Typed(t) => Some(t.layer_type()),
            AnyLayer::Ref(_) => None,
        }}
    }}

    /// Get the source name if this is a typed layer with a source.
    pub fn source(&self) -> Option<&str> {{
        self.common()?.source.as_ref().map(|s| s.as_str())
    }}

    /// Get the source-layer name if this is a typed layer with one.
    pub fn source_layer(&self) -> Option<&str> {{
        self.common()?.source_layer.as_ref().map(|s| s.as_str())
    }}
}}

impl LayerId {{
    pub fn as_str(&self) -> &str {{
        &self.0
    }}
}}

impl LayerSource {{
    pub fn as_str(&self) -> &str {{
        &self.0
    }}
}}

impl LayerSourceLayer {{
    pub fn as_str(&self) -> &str {{
        &self.0
    }}
}}

impl LayerFilter {{
    /// Returns `true` if this filter is the literal `false` (layer never renders).
    pub fn is_always_false(&self) -> bool {{
        matches!(self, LayerFilter::Literal(false))
    }}

    /// Returns `true` if this filter is the literal `true` (layer always renders).
    pub fn is_always_true(&self) -> bool {{
        matches!(self, LayerFilter::Literal(true))
    }}

    /// Returns the inner expression if this is an `Expr` variant.
    pub fn as_boolean(&self) -> Option<&Boolean> {{
        match self {{
            LayerFilter::Expr(b) => Some(b),
            LayerFilter::AnyExpr(_) | LayerFilter::Literal(_) => None,
        }}
    }}

    /// Returns a mutable reference to the inner expression if this is an `Expr` variant.
    pub fn as_boolean_mut(&mut self) -> Option<&mut Boolean> {{
        match self {{
            LayerFilter::Expr(b) => Some(b),
            LayerFilter::AnyExpr(_) | LayerFilter::Literal(_) => None,
        }}
    }}

    /// Serialize to `serde_json::Value` for passes that still operate on JSON.
    pub fn to_json_value(&self) -> serde_json::Value {{
        serde_json::to_value(self).expect("LayerFilter serialization is infallible")
    }}

    /// Deserialize from `serde_json::Value`.  Returns `None` if the value is not
    /// a valid filter (e.g. a string or object).
    pub fn from_value(v: serde_json::Value) -> Option<Self> {{
        serde_json::from_value(v).ok()
    }}
}}

impl LayerMinzoom {{
    pub fn as_f64(&self) -> Option<f64> {{
        self.0.as_f64()
    }}

    pub fn from_f64(n: f64) -> Option<Self> {{
        serde_json::Number::from_f64(n).map(Self)
    }}
}}

impl LayerMaxzoom {{
    pub fn as_f64(&self) -> Option<f64> {{
        self.0.as_f64()
    }}

    pub fn from_f64(n: f64) -> Option<Self> {{
        serde_json::Number::from_f64(n).map(Self)
    }}
}}
"#
    ));
}

fn layer_fields_to_mir(
    fields: &std::collections::BTreeMap<String, MirLayerField>,
) -> Vec<MirField> {
    fields
        .iter()
        .map(|(name, f)| layer_field_to_mir(name, f))
        .collect()
}

fn layer_field_to_mir(spec_name: &str, f: &MirLayerField) -> MirField {
    let meta = MirFieldMeta {
        spec_name: spec_name.to_string(),
        rust_name: to_snake_case(spec_name),
        optional: !f.required,
        transition: false,
        expression: f.expression.clone(),
        doc: f.doc.clone(),
        example: None,
        units: None,
    };

    match &f.r#type {
        MirType::Number { min, max } => MirField::Number(MirNumberField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            min: *min,
            max: *max,
            period: None,
        }),
        MirType::String => MirField::String(MirStringField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }),
        MirType::Boolean => MirField::Boolean(MirBooleanField {
            meta,
            default: f.default.as_ref().and_then(|v| v.as_bool()),
        }),
        MirType::Color => MirField::Color(MirColorField {
            meta,
            default: f.default.clone(),
        }),
        MirType::Enum { values } => MirField::Enum(MirEnumField {
            meta,
            default: f.default.clone(),
            variants: MirEnum::Regular(MirRegularEnum {
                variants: values
                    .iter()
                    .map(|v| (v.clone(), MirRegularVariant { doc: String::new() }))
                    .collect(),
            }),
        }),
        MirType::Array { element, length } => {
            let mir_element = array_element_type_to_mir(element);
            MirField::Array(crate::mir::types::MirArrayField {
                meta,
                default: f
                    .default
                    .as_ref()
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                element: mir_element,
                length: *length,
            })
        }
        MirType::Padding => MirField::Padding(MirPaddingField {
            meta,
            default: match &f.default {
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect(),
                _ => vec![],
            },
        }),
        MirType::Formatted { tokens } => MirField::FormattedText(MirFormattedTextField {
            meta,
            tokens: *tokens,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default(),
        }),
        MirType::ResolvedImage { tokens } => MirField::ResolvedImage(MirResolvedImageField {
            meta,
            tokens: Some(*tokens),
        }),
        MirType::NumberArray { min, max } => MirField::NumberArray(MirNumberArrayField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            min: *min,
            max: *max,
        }),
        MirType::ColorArray => MirField::ColorArray(MirColorArrayField {
            meta,
            default: f
                .default
                .as_ref()
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }),
        MirType::State => MirField::State(MirStateField {
            meta,
            default: f.default.clone().unwrap_or(serde_json::Value::Null),
        }),
        MirType::AnyObject => MirField::Star(meta),
        MirType::Sprite => MirField::Sprite(meta),
        MirType::PromoteId => MirField::PromoteId(meta),
        MirType::ProjectionDefinition => {
            MirField::ProjectionDefinition(MirProjectionDefinitionField {
                meta,
                default: f
                    .default
                    .as_ref()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
            })
        }
        MirType::VariableAnchorOffsetCollection => MirField::VariableAnchorOffsetCollection(meta),
    }
}

fn array_element_type_to_mir(element: &MirArrayElementType) -> MirArrayElement {
    match element {
        MirArrayElementType::String => MirArrayElement::String,
        MirArrayElementType::Number => MirArrayElement::Number {
            min: None,
            max: None,
        },
        MirArrayElementType::Color => MirArrayElement::Color,
        MirArrayElementType::Enum(values) => MirArrayElement::Enum(MirRegularEnum {
            variants: values
                .iter()
                .map(|v| (v.clone(), MirRegularVariant { doc: String::new() }))
                .collect(),
        }),
        MirArrayElementType::Layer => MirArrayElement::Layer,
    }
}

// ── MirField dispatch ─────────────────────────────────────────────────────────

/// Dispatch a `MirField` to the appropriate item generator.
/// Called both from this module and from `items/array.rs` (for `Complex` elements).
pub fn generate_mir_type(scope: &mut Scope, name: &str, field: &MirField) {
    match field {
        MirField::Number(f) => items::number::generate(scope, name, f),
        MirField::Boolean(f) => items::boolean::generate(scope, name, f),
        MirField::String(f) => items::string::generate(scope, name, f),
        MirField::Color(f) => items::color::generate(scope, name, f),
        MirField::Enum(f) => items::r#enum::generate_mir(scope, name, f),
        MirField::Array(f) => items::array::generate(scope, name, f),
        MirField::NumberArray(f) => items::number_array::generate(scope, name, f),
        MirField::ColorArray(f) => items::color_array::generate(scope, name, f),
        MirField::FormattedText(f) => items::formatted::generate(scope, name, f),
        MirField::ResolvedImage(f) => items::resolved_image::generate(scope, name, f),
        MirField::Padding(f) => items::padding::generate(scope, name, f),
        MirField::State(f) => items::state::generate(scope, name, f),
        MirField::ProjectionDefinition(f) => items::projection_definition::generate(scope, name, f),
        MirField::Sprite(m) => items::sprite::generate(scope, name, m),
        MirField::PromoteId(m) => items::promote_id::generate(scope, name, m),
        MirField::VariableAnchorOffsetCollection(m) => {
            items::variable_anchor_offset_collection::generate(scope, name, m)
        }
        MirField::Star(m) => items::star::generate(scope, name, m),
        MirField::Reference(f) => items::reference::generate(scope, name, f),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::decoder::StyleReference;
    use crate::mir::MirSpec;

    #[test]
    fn test_generate_spec_items() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "doc": "A number between 0 and 10.",
              "type": "number",
              "default": 0
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = MirSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_groups() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "names": {
              "name_one": {
                "type": "number",
                "doc": "A number between 0 and 10.",
                "default": 1.0
              }
            }
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = MirSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }

    #[test]
    fn test_generate_spec_oneof() {
        let reference = json!({
            "$version": 8,
            "$root": {},
            "number_one": {
              "type": "number",
              "doc": "A number between 0 and 20.",
              "default": 1.0,
              "minimum": 0.0,
              "maximum": 10.0
            },
            "number_two": {
              "type": "number",
              "doc": "Another number"
            },
            "numbers": ["number_one", "number_two"]
        });
        let reference: StyleReference = serde_json::from_value(reference).unwrap();
        let spec = MirSpec::from(reference);
        insta::assert_snapshot!(generate_spec_scope(&spec));
    }
}
