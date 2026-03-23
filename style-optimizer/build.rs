fn main() {
    prost_build::Config::new()
        .compile_protos(&["proto/vector_tile.proto"], &["proto/"])
        .expect("Failed to compile vector_tile.proto");
}
