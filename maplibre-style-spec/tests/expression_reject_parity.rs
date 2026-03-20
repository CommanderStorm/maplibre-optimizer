//! Upstream expression integration parity (compile success vs failure only).
//!
//! Corpus: `upstream/test/integration/expression/tests/**/test.json` (same layout
//! as `upstream/test/integration/expression/expression.test.ts`).
//!
//! **Compared:** `expected.compiled.result` is `"success"` or `"error"`.
//!
//! **Actual:** `validate_expression_with_spec` (in `maplibre_style_spec::expression_validate`) —
//! recursive walk, operator whitelist from `IntermediateSpec`, and typed serde checks against
//! generated syntax enums per operator output group. Full evaluation/output parity is out of scope.
//!
//! **Policy:** we only fail the run when upstream marks an expression as **`success`** but the
//! validator rejects it. Cases where upstream expects **`error`** but we still accept the expression
//! are tracked as `permissive_count` (the checker is intentionally looser than GL JS type rules).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::expression_validate::{
    operator_groups_map, validate_expression_with_spec,
};
use maplibre_style_spec::mir::{ExprParamType, ExprType, IntermediateSpec};
use maplibre_style_spec::spec::ExprOrLiteral;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PropertySpec {
    #[serde(rename = "type")]
    property_type: String,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExpressionFixture {
    expression: ExprOrLiteral,
    #[serde(rename = "propertySpec")]
    property_spec: Option<PropertySpec>,
    #[serde(default)]
    expected: Option<FixtureExpected>,
}

#[derive(Debug, Deserialize)]
struct FixtureExpected {
    compiled: FixtureCompiled,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "result", rename_all = "lowercase")]
enum FixtureCompiled {
    Success {
        #[serde(rename = "isFeatureConstant")]
        _is_feature_constant: Option<bool>,
        #[serde(rename = "isZoomConstant")]
        _is_zoom_constant: Option<bool>,
        #[serde(rename = "type")]
        _type: Option<String>,
    },
    Error {
        #[allow(dead_code)]
        errors: Vec<FixtureError>,
    },
}

#[derive(Debug, Deserialize)]
struct FixtureError {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    error: String,
}

#[test]
fn upstream_expression_reject_parity() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let tests_root = workspace_root.join("upstream/test/integration/expression/tests");
    assert!(
        tests_root.exists(),
        "missing expression fixtures: {}",
        tests_root.display()
    );

    let v8 = include_str!("../../upstream/src/reference/v8.json");
    let reference: StyleReference =
        serde_json::from_str(v8).expect("v8.json should parse as StyleReference");
    let spec = IntermediateSpec::from(reference);
    let mut known_ops: HashSet<String> = spec.expressions.operators.keys().cloned().collect();
    // Used for short-circuit tests (`["error", "..."]`); not always present as a named spec entry.
    known_ops.insert("error".to_string());
    let op_to_groups = operator_groups_map(&spec.expressions);

    let mut fixture_paths = Vec::new();
    collect_test_json(&tests_root, &mut fixture_paths);
    fixture_paths.sort();

    assert!(
        !fixture_paths.is_empty(),
        "no test.json under {}",
        tests_root.display()
    );

    let mut examined = 0usize;
    let mut mismatches = Vec::new();
    let mut permissive_mismatches = Vec::new();
    let mut permissive_count = 0usize;

    for path in &fixture_paths {
        examined += 1;
        let text = fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("read {}: {e}", path.display());
        });
        let fixture: ExpressionFixture = serde_json::from_str(&text).unwrap_or_else(|e| {
            panic!("parse {}: {e}", path.display());
        });
        let Some(expected) = fixture.expected.as_ref() else {
            mismatches.push(format!(
                "{}: missing `expected` block",
                path.strip_prefix(&workspace_root)
                    .unwrap_or(path.as_path())
                    .display()
            ));
            continue;
        };

        let expected_ok = matches!(expected.compiled, FixtureCompiled::Success { .. });
        let expected_ty = fixture
            .property_spec
            .as_ref()
            .map(expr_type_from_property_spec)
            .unwrap_or(ExprType::Any);
        let actual_ok = validate_expression_with_spec(
            &fixture.expression,
            &expected_ty,
            &op_to_groups,
            &known_ops,
        )
        .is_ok();

        if expected_ok != actual_ok {
            let rel = path
                .strip_prefix(&workspace_root)
                .unwrap_or(path.as_path())
                .display()
                .to_string();
            let err = validate_expression_with_spec(
                &fixture.expression,
                &expected_ty,
                &op_to_groups,
                &known_ops,
            )
            .err()
            .unwrap_or_default();
            let reason = if err.is_empty() {
                String::new()
            } else {
                format!(", reason={err}")
            };
            // Rejecting a compile-success fixture is always a bug; accepting compile-error fixtures is
            // allowed until the validator matches GL JS type errors.
            if expected_ok && !actual_ok {
                mismatches.push(format!(
                    "{} -> expected=success, actual=error{}",
                    rel, reason,
                ));
            } else if !expected_ok && actual_ok {
                permissive_count += 1;
                // Print the upstream errors so we can see what we accepted incorrectly.
                // Upstream `errors[]` is the most informative part for permissive cases.
                if permissive_mismatches.len() < 30 {
                    let upstream_errors = match expected.compiled {
                        FixtureCompiled::Error { ref errors } => errors
                            .iter()
                            .map(|e| format!("{}: {}", e.key, e.error))
                            .collect::<Vec<_>>()
                            .join(" | "),
                        FixtureCompiled::Success { .. } => String::new(),
                    };
                    permissive_mismatches.push(format!(
                        "{} -> expected=error, actual=success{}",
                        rel,
                        if upstream_errors.is_empty() {
                            String::new()
                        } else {
                            format!(", upstream_errors=[{upstream_errors}]")
                        }
                    ));
                }
            }
        }
    }

    // Always print a short summary for local runs.
    eprintln!(
        "expression parity: {} fixtures, {} strict mismatches (upstream success but we error), {} permissive (upstream error but we accept)",
        examined,
        mismatches.len(),
        permissive_count
    );

    if !permissive_mismatches.is_empty() {
        eprintln!(
            "first permissive mismatches (showing {}):",
            permissive_mismatches.len()
        );
        for s in permissive_mismatches.iter() {
            eprintln!("  {s}");
        }
    }

    assert!(
        mismatches.is_empty(),
        "expressions marked compile-success upstream must validate here (first {}):\n{}",
        mismatches.len().min(80),
        mismatches
            .iter()
            .take(80)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn expr_type_from_property_spec(ps: &PropertySpec) -> ExprType {
    match ps.property_type.as_str() {
        "string" => ExprType::String,
        "number" => ExprType::Number,
        "boolean" => ExprType::Boolean,
        "color" => ExprType::Color,
        "formatted" => ExprType::Formatted,
        "image" => ExprType::Image,
        "object" => ExprType::Object,
        // Generic array: we don't know element type from the property alone.
        "array" => ExprType::Array {
            element: ps
                .value
                .as_deref()
                .and_then(|v| match v {
                    "string" => Some(ExprType::String),
                    "number" => Some(ExprType::Number),
                    "boolean" => Some(ExprType::Boolean),
                    "color" => Some(ExprType::Color),
                    _ => None,
                })
                .map(|inner_ty| Box::new(ExprParamType::Expression(inner_ty))),
            length: None,
        },
        other => {
            let ty = other.trim();
            if let Some(inner) = ty.strip_prefix("array<").and_then(|s| s.strip_suffix('>')) {
                let inner = inner.trim();
                let inner_ty = match inner {
                    "string" => ExprType::String,
                    "number" => ExprType::Number,
                    "boolean" => ExprType::Boolean,
                    "color" => ExprType::Color,
                    _ => {
                        return ExprType::Array {
                            element: None,
                            length: None,
                        };
                    }
                };
                return ExprType::Array {
                    element: Some(Box::new(ExprParamType::Expression(inner_ty))),
                    length: None,
                };
            }

            // Fallback: treat unknown property types as unconstrained.
            ExprType::Any
        }
    }
}

fn collect_test_json(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("entry: {e}"));
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            collect_test_json(&path, out);
        } else if name == "test.json" {
            out.push(path);
        }
    }
}
