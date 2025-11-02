use std::fs;

use maplibre_style_spec::spec::decoder::StyleReference;
use maplibre_style_spec::spec::generator::generate_spec_scope;

fn main() {
    let json_content = include_str!("../../tests/upstream/src/reference/v8.json");

    let reference: StyleReference =
        serde_json::from_str(json_content).expect("Failed to parse v8.json into StyleReference");

    let generated_code = generate_spec_scope(reference);

    let output_path = "src/generated_spec.rs";
    fs::write(output_path, &generated_code).expect("Failed to write generated_spec.rs");

    println!("Successfully generated spec code to {}", output_path);
    println!("Generated {} bytes of code", generated_code.len());
}
