use std::fs;
use std::path::PathBuf;
use maplibre_style_spec::decoder::StyleReference;
use maplibre_style_spec::generator::generate_spec_scope;

fn main() {
    let json_content = include_str!("../../tests/upstream/src/reference/v8.json");

    let reference: StyleReference =
        serde_json::from_str(json_content).expect("Failed to parse v8.json into StyleReference");

    let generated_code = generate_spec_scope(reference);
    let code_bytes_cnt = generated_code.len();
    let code_lines_cnt = generated_code.lines().count();

    let output_path = PathBuf::from("maplibre-style-spec/src/spec/mod.rs");
    println!("Generating spec code to {output_path:?}");
    if let Err(e) = fs::write(&output_path, &generated_code) {
        panic!("Failed to write {output_path:?} with {code_bytes_cnt}B on {code_lines_cnt} lines code because {e:?}");
    }

    println!("Successfully generated spec code to {output_path:?}");
    println!("Generated {code_bytes_cnt}B on {code_lines_cnt} lines code");
}
