//! Round-trip tests against Maputnik's style catalog.
//!
//! Fetches real-world styles from the Maputnik catalog and verifies that they
//! deserialize into our typed spec and re-serialize without error.
//! Legacy property functions (stops) are converted to expressions via `gl-style-migrate`.
#![cfg(feature = "full")]

use std::process::Command;

use maplibre_style_spec::spec::MaplibreStyleSpecification;

const CATALOG_URL: &str =
    "https://raw.githubusercontent.com/maplibre/maputnik/main/src/config/styles.json";

/// Run `gl-style-migrate` on a style JSON string, converting legacy stops to expressions.
fn migrate_style(raw: &str) -> Result<String, String> {
    let tmp = std::env::temp_dir().join(format!("style-migrate-{}.json", std::process::id()));
    std::fs::write(&tmp, raw).map_err(|e| format!("write tmp: {e}"))?;

    let output = Command::new("gl-style-migrate")
        .arg(&tmp)
        .output()
        .map_err(|e| format!("gl-style-migrate exec: {e}"))?;

    let _ = std::fs::remove_file(&tmp);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gl-style-migrate failed: {stderr}"));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("invalid utf8: {e}"))
}

#[tokio::test]
async fn roundtrip_maputnik_catalog() {
    let client = reqwest::Client::new();

    let catalog: Vec<serde_json::Value> = client
        .get(CATALOG_URL)
        .send()
        .await
        .expect("failed to fetch catalog")
        .json()
        .await
        .expect("failed to parse catalog JSON");

    let mut succeeded = 0u32;
    let mut skipped = 0u32;
    let mut failures: Vec<(String, String)> = Vec::new();

    for entry in &catalog {
        let id = entry["id"].as_str().unwrap();
        let url = entry["url"].as_str().unwrap();

        eprintln!("Testing style: {id} ({url})");

        // Fetch the style; skip on any HTTP error.
        let resp = match client.get(url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                eprintln!("  SKIP {id}: HTTP {}", r.status());
                skipped += 1;
                continue;
            }
            Err(e) => {
                eprintln!("  SKIP {id}: fetch failed: {e}");
                skipped += 1;
                continue;
            }
        };

        let raw_text = resp.text().await.expect("failed to read response body");

        // Migrate legacy property functions to expressions.
        let migrated = match migrate_style(&raw_text) {
            Ok(m) => m,
            Err(e) => {
                failures.push((id.to_owned(), format!("migrate: {e}")));
                continue;
            }
        };

        // Deserialize into our typed spec.
        let spec: MaplibreStyleSpecification = match serde_json::from_str(&migrated) {
            Ok(s) => s,
            Err(e) => {
                failures.push((id.to_owned(), format!("deserialize: {e}")));
                continue;
            }
        };

        // Re-serialize back to Value.
        if let Err(e) = serde_json::to_value(&spec) {
            failures.push((id.to_owned(), format!("re-serialize: {e}")));
            continue;
        }

        eprintln!("  OK {id}");
        succeeded += 1;
    }

    eprintln!("\n=== Catalog round-trip results ===");
    eprintln!("  succeeded: {succeeded}");
    eprintln!("  skipped:   {skipped}");
    eprintln!("  failed:    {}", failures.len());

    if !failures.is_empty() {
        for (id, msg) in &failures {
            eprintln!("  FAIL {id}: {msg}");
        }
        panic!(
            "{} style(s) failed round-trip: {}",
            failures.len(),
            failures
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    assert!(succeeded > 0, "no styles were successfully tested");
}
