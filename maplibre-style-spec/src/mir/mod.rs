mod expressions;
mod layers;
pub mod lower;
mod preprocessing;
pub mod resources;
mod root;
mod sources;
pub mod types;

use std::collections::BTreeMap;

pub use expressions::{
    ExprParamType, ExprType, ExpressionGroup, ExpressionOperator, ExpressionOverload,
    ExpressionParam, IntermediateExpressions as Expressions, LiteralKind, OverloadParams,
    ResolvedParam,
};
pub use layers::{IntermediateLayerField, IntermediateLayerType, IntermediateLayers};
pub use root::IntermediateRootPrimitives;
/// Alias kept for existing imports (`generator`, tests).
pub type Layers = IntermediateLayers;
pub use sources::{IntermediateSources as Sources, SourceTypeDef};
pub use types::{
    ArrayElement, ArrayField, BooleanField, ColorArrayField, ColorField, EnumField,
    ExpressionCapabilities, FieldMeta, FormattedTextField, MirEnum, NumberArrayField, NumberField,
    PaddingField, ProjectionDefinitionField, ReferenceField, RegularEnum, RegularVariant,
    ResolvedImageField, StateField, StringField, SyntaxEnumMap, SyntaxVariantDef, VersionEnum,
};

use crate::decoder;
use crate::decoder::{ParsedItem, PrimitiveType, TopLevelItem};
use crate::mir::expressions::IntermediateExpressions;
use crate::mir::lower::lower_parsed_item;
use crate::mir::resources::{IntermediateFontResources, IntermediateSpriteResources};
use crate::mir::sources::IntermediateSources;
use crate::mir::types::MirField;

// ── IntermediateNamedType ─────────────────────────────────────────────────────

/// A named top-level type referenced from the spec.
pub enum IntermediateNamedType {
    /// A group of fields (e.g. `light`, `terrain`, `fog`, `sky`).
    Struct(Vec<MirField>),
    /// A single type definition: enum, alias, or special type.
    TypeDef(MirField),
    /// A sum type (Rust enum wrapping multiple struct variants).
    OneOf(IntermediateOneOf),
}

/// A OneOf type — a Rust enum where each variant wraps a named struct.
pub struct IntermediateOneOf {
    /// Variant names (keys into `IntermediateSpec::named_types` of type `Struct`).
    pub variants: Vec<String>,
    /// Serde tag field, if discriminant detection found one (→ `#[serde(tag="...")]`).
    pub tag: Option<String>,
    /// Maps variant name → serde rename value (only populated when `tag.is_some()`).
    pub renames: BTreeMap<String, String>,
}

// ── IntermediateSpec ──────────────────────────────────────────────────────────

/// The fully-semantic intermediate representation of the style spec.
/// All preprocessing has been applied; every consumer works only with MIR types.
pub struct IntermediateSpec {
    pub version: u8,
    /// Simple root-level fields (e.g. `center`, `zoom`, `pitch`, `bearing`).
    pub root: IntermediateRootPrimitives,
    /// Rendering layers.
    pub layers: IntermediateLayers,
    /// Expression operators grouped by output type.
    pub expressions: IntermediateExpressions,
    /// Data source type definitions.
    pub sources: IntermediateSources,
    /// Remaining named types (groups, type aliases, sum types).
    pub named_types: BTreeMap<String, IntermediateNamedType>,
    /// Glyph/font resource metadata (from spec's `glyphs` root field).
    pub fonts: IntermediateFontResources,
    /// Sprite resource metadata (from spec's `sprite` root field).
    pub sprite: IntermediateSpriteResources,
}

impl From<decoder::StyleReference> for IntermediateSpec {
    fn from(mut value: decoder::StyleReference) -> Self {
        let expressions = preprocessing::preprocess_expression(&mut value.fields);
        let layers = preprocessing::preprocess_layers(&mut value);
        let sources = preprocessing::preprocess_sources(&mut value.fields);

        // Remove the meta-type entry — not useful for codegen.
        value.fields.remove("property_type");

        // Extract font/sprite resource metadata from $root before it is consumed.
        let fonts = {
            let url_template = value.root.remove("glyphs").and_then(|item| {
                if let ParsedItem::Primitive(PrimitiveType::String { common, default }) = item {
                    default.or_else(|| {
                        common
                            .example
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                    })
                } else {
                    None
                }
            });
            IntermediateFontResources { url_template }
        };

        let sprite = {
            let url = value.root.remove("sprite").and_then(|item| {
                if let ParsedItem::Primitive(PrimitiveType::Sprite(common)) = item {
                    common.example
                } else {
                    None
                }
            });
            IntermediateSpriteResources { url }
        };

        let named_types = lower_remaining_fields(value.fields);

        IntermediateSpec {
            version: value.version,
            root: IntermediateRootPrimitives::from(value.root),
            expressions,
            layers,
            sources,
            named_types,
            fonts,
            sprite,
        }
    }
}

// ── Remaining-fields lowering ─────────────────────────────────────────────────

/// Lower whatever remains in `fields` into `IntermediateNamedType` entries.
/// Applies discriminant detection and removal before lowering groups.
fn lower_remaining_fields(
    fields: BTreeMap<String, TopLevelItem>,
) -> BTreeMap<String, IntermediateNamedType> {
    // Separate fields into bins so we can process discriminants before lowering
    let mut groups: BTreeMap<String, BTreeMap<String, ParsedItem>> = BTreeMap::new();
    let mut one_ofs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut items: BTreeMap<String, ParsedItem> = BTreeMap::new();

    for (key, item) in fields {
        match item {
            TopLevelItem::Group(g) => {
                groups.insert(key, g);
            }
            TopLevelItem::OneOf(v) => {
                one_ofs.insert(key, v);
            }
            TopLevelItem::Item(i) => {
                items.insert(key, *i);
            }
        }
    }

    // Detect and remove discriminants from groups referenced by OneOfs
    let discriminants = extract_discriminants(&one_ofs, &mut groups);

    let mut result: BTreeMap<String, IntermediateNamedType> = BTreeMap::new();

    // Lower groups (filter out PropertyType meta-fields)
    for (key, group) in groups {
        let mir_fields = group
            .into_iter()
            .filter(|(k, _)| k != "property-type")
            .filter(|(_, v)| !matches!(v, ParsedItem::Primitive(PrimitiveType::PropertyType(_))))
            .map(|(k, v)| lower_parsed_item(&k, v))
            .collect();
        result.insert(key, IntermediateNamedType::Struct(mir_fields));
    }

    // Lower single items
    for (key, item) in items {
        let field = lower_parsed_item(&key, item);
        result.insert(key, IntermediateNamedType::TypeDef(field));
    }

    // Lower OneOfs (using the extracted discriminant info)
    for (key, variants) in one_ofs {
        let (tag, renames) = discriminants
            .get(&key)
            .cloned()
            .map(|(tag, renames)| (Some(tag), renames))
            .unwrap_or_default();

        result.insert(
            key,
            IntermediateNamedType::OneOf(IntermediateOneOf {
                variants,
                tag,
                renames,
            }),
        );
    }

    result
}

/// Detect single-variant enum fields that serve as serde discriminants for OneOf types.
/// Removes those fields from their groups so codegen doesn't emit them.
///
/// Returns a map from OneOf key → (tag_field_name, map{variant_key → discriminant_value}).
fn extract_discriminants(
    one_ofs: &BTreeMap<String, Vec<String>>,
    groups: &mut BTreeMap<String, BTreeMap<String, ParsedItem>>,
) -> BTreeMap<String, (String, BTreeMap<String, String>)> {
    use crate::decoder::r#enum::EnumValues;
    use crate::generator::formatter::to_upper_camel_case;

    let mut result: BTreeMap<String, (String, BTreeMap<String, String>)> = BTreeMap::new();

    for (top_name, join_keys) in one_ofs {
        let mut found_tag: Option<String> = None;
        let mut found_renames: BTreeMap<String, String> = BTreeMap::new();

        for join_key in join_keys {
            let Some(group) = groups.get(join_key) else {
                continue;
            };

            for (field_name, item) in group {
                if let ParsedItem::Primitive(PrimitiveType::Enum { values, .. }) = item
                    && values.len() == 1
                    && let EnumValues::Enum(enum_map) = values
                {
                    let value = enum_map.keys().next().expect("len is 1").clone();
                    // The tag field name must be consistent across all variants
                    if found_tag.is_none() {
                        found_tag = Some(field_name.clone());
                    }
                    let variant_name = to_upper_camel_case(join_key);
                    found_renames.insert(variant_name, value);
                }
            }
        }

        if let Some(tag) = found_tag {
            // Remove the discriminant field from all referenced groups
            for join_key in join_keys {
                if let Some(group) = groups.get_mut(join_key) {
                    group.remove(&tag);
                }
            }
            result.insert(top_name.clone(), (tag, found_renames));
        }
    }

    result
}
