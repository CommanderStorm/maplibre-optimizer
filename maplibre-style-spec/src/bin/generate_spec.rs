use std::fs;
use std::path::PathBuf;

use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::generator::generate_spec_modules;
use maplibre_style_spec::mir::MirSpec;

fn main() {
    let json_content = include_str!("../../../upstream/src/reference/v8.json");

    let reference: StyleReference =
        serde_json::from_str(json_content).expect("Failed to parse v8.json into StyleReference");

    let spec = MirSpec::from(reference);
    let generated_scope = generate_spec_modules(&spec);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let spec_rs = manifest_dir.join("src/spec.rs");
    let spec_dir = manifest_dir.join("src/spec");
    fs::create_dir_all(&spec_dir).expect("failed to create src/spec directory");

    // Avoid module resolution ambiguity (`src/spec.rs` vs `src/spec/mod.rs`).
    let _ = fs::remove_file(&spec_rs);

    let domains = [
        "literals",
        "root",
        "named_types",
        "expressions",
        "sources",
        "layers",
    ];

    let mod_rs_content = domains
        .iter()
        .map(|d| format!("pub mod {d};"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n\n"
        + &domains
            .iter()
            .map(|d| format!("pub use {d}::*;"))
            .collect::<Vec<_>>()
            .join("\n")
        + "\npub use crate::shared_expr::*;";

    let mod_rs_path = spec_dir.join("mod.rs");
    println!("Generating spec mod at {mod_rs_path:?}");
    fs::write(&mod_rs_path, mod_rs_content).expect("failed to write spec/mod.rs");

    let mut code_bytes_cnt = 0usize;
    let mut code_lines_cnt = 0usize;
    for (domain_name, module) in generated_scope.modules() {
        let body = module.body_to_string();
        let content = format!(
            "#![allow(clippy::large_enum_variant)]\n#[allow(unused_imports)]\nuse super::*;\n#[allow(unused_imports)]\nuse crate::{{numeric_prop, color_prop, boolean_prop, string_prop}};\n\n{body}\n"
        );
        let out_path = spec_dir.join(format!("{domain_name}.rs"));
        println!("Generating spec module {domain_name} at {out_path:?}");
        code_bytes_cnt += content.len();
        code_lines_cnt += content.lines().count();
        fs::write(&out_path, content)
            .unwrap_or_else(|e| panic!("failed to write {out_path:?} because {e:?}"));
    }

    println!("Successfully generated split spec into {spec_dir:?}");
    println!("Generated {code_bytes_cnt}B on {code_lines_cnt} lines code");
}
