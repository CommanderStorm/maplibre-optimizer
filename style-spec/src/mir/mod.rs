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
    MirExprParamType, MirExprType, MirExpressionGroup, MirExpressionOperator,
    MirExpressionOverload, MirExpressionParam, MirExpressions, MirLiteralKind, MirOverloadParams,
    MirResolvedParam,
};
pub use layers::{MirLayerField, MirLayerType, MirLayers, MirPropertySection};
pub use root::MirRootPrimitives;
pub use sources::{MirSourceTypeDef, MirSources};
pub use types::{
    MirArrayElement, MirArrayField, MirBooleanField, MirColorArrayField, MirColorField, MirEnum,
    MirEnumField, MirExpressionCapabilities, MirFieldMeta, MirFormattedTextField,
    MirNumberArrayField, MirNumberField, MirPaddingField, MirProjectionDefinitionField,
    MirReferenceField, MirRegularEnum, MirRegularVariant, MirResolvedImageField, MirStateField,
    MirStringField, MirSyntaxEnumMap, MirSyntaxVariantDef, MirVersionEnum,
};

use crate::decoder;
use crate::decoder::{DecodedParsedItem, DecodedPrimitiveType, DecodedTopLevelItem};
use crate::mir::lower::lower_parsed_item;
use crate::mir::resources::{MirFontResources, MirSpriteResources};
use crate::mir::types::MirField;

// ── MirNamedType ─────────────────────────────────────────────────────

/// A named top-level type referenced from the spec.
pub enum MirNamedType {
    /// A group of fields (e.g. `light`, `terrain`, `fog`, `sky`).
    Struct(Vec<MirField>),
    /// A single type definition: enum, alias, or special type.
    TypeDef(MirField),
    /// A sum type (Rust enum wrapping multiple struct variants).
    OneOf(MirOneOf),
}

/// A OneOf type — a Rust enum where each variant wraps a named struct.
pub struct MirOneOf {
    /// Variant names (keys into `MirSpec::named_types` of type `Struct`).
    pub variants: Vec<String>,
    /// Serde tag field, if discriminant detection found one (→ `#[serde(tag="...")]`).
    pub tag: Option<String>,
    /// Maps variant name → serde rename value (only populated when `tag.is_some()`).
    pub renames: BTreeMap<String, String>,
}

// ── MirSpec ──────────────────────────────────────────────────────────

/// The fully-semantic intermediate representation of the style spec.
/// All preprocessing has been applied; every consumer works only with MIR types.
pub struct MirSpec {
    pub version: u8,
    /// Simple root-level fields (e.g. `center`, `zoom`, `pitch`, `bearing`).
    pub root: MirRootPrimitives,
    /// Rendering layers.
    pub layers: MirLayers,
    /// Expression operators grouped by output type.
    pub expressions: MirExpressions,
    /// Data source type definitions.
    pub sources: MirSources,
    /// Remaining named types (groups, type aliases, sum types).
    pub named_types: BTreeMap<String, MirNamedType>,
    /// Glyph/font resource metadata (from spec's `glyphs` root field).
    pub fonts: MirFontResources,
    /// Sprite resource metadata (from spec's `sprite` root field).
    pub sprite: MirSpriteResources,
}

impl From<decoder::StyleReference> for MirSpec {
    fn from(mut value: decoder::StyleReference) -> Self {
        let expressions = preprocessing::preprocess_expression(&mut value.fields);
        let layers = preprocessing::preprocess_layers(&mut value);
        let sources = preprocessing::preprocess_sources(&mut value.fields);

        // Remove the meta-type entry — not useful for codegen.
        value.fields.remove("property_type");

        // Extract font/sprite resource metadata from $root before it is consumed.
        let fonts = {
            let url_template = value.root.remove("glyphs").and_then(|item| {
                if let DecodedParsedItem::Primitive(DecodedPrimitiveType::String {
                    common,
                    default,
                }) = item
                {
                    default.or_else(|| {
                        common
                            .example
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                    })
                } else {
                    None
                }
            });
            MirFontResources { url_template }
        };

        let sprite = {
            let url = value.root.remove("sprite").and_then(|item| {
                if let DecodedParsedItem::Primitive(DecodedPrimitiveType::Sprite(common)) = item {
                    common.example
                } else {
                    None
                }
            });
            MirSpriteResources { url }
        };

        let named_types = lower_remaining_fields(value.fields);

        MirSpec {
            version: value.version,
            root: MirRootPrimitives::from(value.root),
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

/// Lower whatever remains in `fields` into `MirNamedType` entries.
/// Applies discriminant detection and removal before lowering groups.
fn lower_remaining_fields(
    fields: BTreeMap<String, DecodedTopLevelItem>,
) -> BTreeMap<String, MirNamedType> {
    // Separate fields into bins so we can process discriminants before lowering
    let mut groups: BTreeMap<String, BTreeMap<String, DecodedParsedItem>> = BTreeMap::new();
    let mut one_ofs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut items: BTreeMap<String, DecodedParsedItem> = BTreeMap::new();

    for (key, item) in fields {
        match item {
            DecodedTopLevelItem::Group(g) => {
                groups.insert(key, g);
            }
            DecodedTopLevelItem::OneOf(v) => {
                one_ofs.insert(key, v);
            }
            DecodedTopLevelItem::Item(i) => {
                items.insert(key, *i);
            }
        }
    }

    // Detect and remove discriminants from groups referenced by OneOfs
    let discriminants = extract_discriminants(&one_ofs, &mut groups);

    let mut result: BTreeMap<String, MirNamedType> = BTreeMap::new();

    // Lower groups (filter out DecodedPropertyType meta-fields)
    for (key, group) in groups {
        let mir_fields = group
            .into_iter()
            .filter(|(k, _)| k != "property-type")
            .filter(|(_, v)| {
                !matches!(
                    v,
                    DecodedParsedItem::Primitive(DecodedPrimitiveType::PropertyType(_))
                )
            })
            .map(|(k, v)| lower_parsed_item(&k, v))
            .collect();
        result.insert(key, MirNamedType::Struct(mir_fields));
    }

    // Lower single items
    for (key, item) in items {
        let field = lower_parsed_item(&key, item);
        result.insert(key, MirNamedType::TypeDef(field));
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
            MirNamedType::OneOf(MirOneOf {
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
    groups: &mut BTreeMap<String, BTreeMap<String, DecodedParsedItem>>,
) -> BTreeMap<String, (String, BTreeMap<String, String>)> {
    use crate::decoder::r#enum::DecodedEnumValues;
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
                if let DecodedParsedItem::Primitive(DecodedPrimitiveType::Enum { values, .. }) =
                    item
                    && values.len() == 1
                    && let DecodedEnumValues::Enum(enum_map) = values
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
