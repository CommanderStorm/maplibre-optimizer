//! Upstream expression integration parity (compile success vs failure only).
//!
//! Corpus: `upstream/test/integration/expression/tests/**/test.json` (same layout
//! as `upstream/test/integration/expression/expression.test.ts`).
//!
//! **Compared:** `expected.compiled.result` is `"success"` or `"error"`.
//!
//! **Actual:** A structural validator plus an operator whitelist from the style
//! reference (`v8.json` → [`IntermediateSpec`]). This matches JS parse-level
//! checks (non-empty array, string operator, no bare objects as operands,
//! known operators) but **does not** type-check, so many upstream `"error"`
//! fixtures that fail only at typechecking will still look **valid** here.
//!
//! Goal: same scaffolding as [`style_spec_reject_parity`]; tighten validation
//! as the expression pipeline approaches full spec compliance. Full
//! evaluation/output parity is out of scope for this test.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::mir::IntermediateSpec;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ExpressionFixture {
    expression: Value,
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
        let actual_ok = mb_expression_accepted(&fixture.expression, &known_ops).is_ok();

        if expected_ok != actual_ok {
            let rel = path
                .strip_prefix(&workspace_root)
                .unwrap_or(path.as_path())
                .display()
                .to_string();
            let err = mb_expression_accepted(&fixture.expression, &known_ops)
                .err()
                .unwrap_or_default();
            let reason = if err.is_empty() {
                String::new()
            } else {
                format!(", reason={err}")
            };
            mismatches.push(format!(
                "{} -> expected={}, actual={}{}",
                rel,
                if expected_ok { "success" } else { "error" },
                if actual_ok { "success" } else { "error" },
                reason,
            ));
        }
    }

    // Always print a short summary for local runs.
    eprintln!(
        "expression parity: {} fixtures, {} mismatches",
        examined,
        mismatches.len()
    );

    assert!(
        mismatches.is_empty(),
        "expression reject-parity mismatches (first {}):\n{}",
        mismatches.len().min(80),
        mismatches
            .iter()
            .take(80)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
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

/// Whether our current validation accepts the fixture root `expression` value.
fn mb_expression_accepted(expr: &Value, known_ops: &HashSet<String>) -> Result<(), String> {
    match expr {
        // Legacy style functions and plain JSON literals (converted by upstream).
        Value::String(_) | Value::Number(_) | Value::Bool(_) => Ok(()),
        Value::Object(_) => Ok(()),
        Value::Null => Err("expression must not be null".to_string()),
        Value::Array(a) => walk_mb_expr_array(a, known_ops),
    }
}

fn walk_mb_expr_array(args: &[Value], known_ops: &HashSet<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(
            "expression array must be non-empty (use [\"literal\", []] for empty array)".into(),
        );
    }
    let op = args
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| "expression operator must be a string".to_string())?;

    if op == "literal" {
        for v in args.iter().skip(1) {
            walk_literal_json(v)?;
        }
        return Ok(());
    }

    if !known_ops.contains(op) {
        return Err(format!("unknown expression operator {op:?}"));
    }

    let tail = &args[1..];
    match op {
        // `["collator", { ... }]` — options object is not wrapped in `literal`.
        "collator" => {
            if tail.len() == 1 && tail[0].is_object() {
                return Ok(());
            }
            for v in tail {
                walk_value_in_expr_context(v, known_ops)?;
            }
            Ok(())
        }
        // GeoJSON-valued operands (plus optional nested expressions in some tests).
        "distance" | "within" => {
            for v in tail {
                if v.is_object() {
                    continue;
                }
                walk_value_in_expr_context(v, known_ops)?;
            }
            Ok(())
        }
        // Alternating string sections and `{ ... }` style objects.
        "format" => {
            for v in tail {
                if v.is_object() {
                    continue;
                }
                walk_value_in_expr_context(v, known_ops)?;
            }
            Ok(())
        }
        _ => {
            for v in tail {
                walk_value_in_expr_context(v, known_ops)?;
            }
            Ok(())
        }
    }
}

/// Walk a value that appears inside an expression: either a nested expression call or an opaque
/// array like `["linear"]` / `["exponential", 2]` that is **not** an operator application.
fn walk_value_in_expr_context(v: &Value, known_ops: &HashSet<String>) -> Result<(), String> {
    match v {
        Value::Array(items) => {
            if items.is_empty() {
                return Err("empty sub-array in expression context".into());
            }
            if let Some(head) = items[0].as_str()
                && known_ops.contains(head)
            {
                return walk_mb_expr_array(items, known_ops);
            }
            for item in items {
                walk_value_in_expr_context(item, known_ops)?;
            }
            Ok(())
        }
        Value::Object(_) => Err("bare JSON object in expression (use [\"literal\", {...}])".into()),
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => Ok(()),
    }
}

/// JSON values inside `["literal", ...]` — nested arrays/objects are data, not operators.
fn walk_literal_json(v: &Value) -> Result<(), String> {
    match v {
        Value::Array(children) => {
            for c in children {
                walk_literal_json(c)?;
            }
            Ok(())
        }
        Value::Object(map) => {
            for (_k, c) in map {
                walk_literal_json(c)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
