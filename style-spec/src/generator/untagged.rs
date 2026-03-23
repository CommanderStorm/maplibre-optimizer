use codegen2::Scope;

/// Describes one variant of an untagged enum for serde codegen.
pub struct Variant {
    /// Rust variant name (e.g. `"Expr"`, `"Literal"`)
    pub name: String,
    /// Inner type *without* `Box<>` wrapper (e.g. `"FillColorExpression"`).
    /// Empty string for unit variants.
    pub inner_type: String,
    /// Whether the field is `Box<T>` — affects both serialize (deref) and deserialize (wrap)
    pub is_boxed: bool,
    /// Whether this is a unit variant (no inner type)
    pub is_unit: bool,
    /// Optional condition to skip this variant during deserialization (e.g. `"value.is_array()"`).
    /// When set, the variant is only tried if the condition evaluates to false.
    pub skip_when: Option<String>,
}

/// Emit hand-written `Serialize` + `Deserialize` impls for an untagged enum.
///
/// The enum itself must already be defined in `scope` (with *no* `serde::Serialize` or
/// `serde::Deserialize` in its derive list and *no* `#[serde(untagged)]` attribute).
///
/// Deserialization strategy: deserialize the input into `serde_json::Value`, then try
/// each variant in order.  On total failure the error message lists every variant that
/// was attempted together with the per-variant error.
pub fn emit_untagged_serde(scope: &mut Scope, enum_name: &str, variants: &[Variant]) {
    let ser_arms: String = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                format!(
                    "            Self::{name} => serializer.serialize_unit(),\n",
                    name = v.name,
                )
            } else {
                let deref = if v.is_boxed { "v.as_ref()" } else { "v" };
                format!(
                    "            Self::{name}(v) => {ser}.serialize(serializer),\n",
                    name = v.name,
                    ser = deref,
                )
            }
        })
        .collect();

    // ── Deserialize ──────────────────────────────────────────────────────
    // Sort: try specific types before catch-all `serde_json::Value` variants.
    let mut deser_order: Vec<usize> = (0..variants.len()).collect();
    deser_order.sort_by_key(|&i| {
        if variants[i].inner_type == "serde_json::Value" {
            1
        } else {
            0
        }
    });
    let deser_tries: String = deser_order
        .iter()
        .map(|&i| &variants[i])
        .map(|v| {
            if v.is_unit {
                format!(
                    "\
        if value.is_null() {{
            return Ok(Self::{name});
        }}\n",
                    name = v.name,
                )
            } else {
                let wrap = if v.is_boxed {
                    format!("Self::{}(Box::new(v))", v.name)
                } else {
                    format!("Self::{}(v)", v.name)
                };
                let body = format!(
                    "\
        match <{ty} as serde::Deserialize>::deserialize(&value) {{
            Ok(v) => return Ok({wrap}),
            Err(e) => errors.push((\"{variant}\", e.to_string())),
        }}\n",
                    ty = v.inner_type,
                    variant = v.name,
                );
                if let Some(ref cond) = v.skip_when {
                    format!("        if !({cond}) {{\n    {body}        }}\n")
                } else {
                    body
                }
            }
        })
        .collect();

    let variant_labels: Vec<_> = variants
        .iter()
        .map(|v| {
            if v.is_unit {
                v.name.clone()
            } else {
                format!("{}({})", v.name, v.inner_type)
            }
        })
        .collect();
    let expecting = variant_labels.join(" | ");

    scope.raw(format!(
        "\
impl serde::Serialize for {enum_name} {{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {{
        match self {{
{ser_arms}        }}
    }}
}}

impl<'de> serde::Deserialize<'de> for {enum_name} {{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {{
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let mut errors: Vec<(&str, std::string::String)> = Vec::new();
        {deser_tries}
        let details: Vec<std::string::String> = errors.iter().map(|(v, e)| format!(\"{{v}}: {{e}}\")).collect();
        Err(serde::de::Error::custom(format!(
            \"{enum_name}: no variant matched. Expected {expecting}. Errors: [{{}}]\",
            details.join(\"; \")
        )))
    }}
}}"
    ));
}
