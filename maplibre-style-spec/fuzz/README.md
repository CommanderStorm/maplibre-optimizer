# maplibre-style-spec fuzz targets

Requires **nightly** Rust and [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz):

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

From this directory:

```sh
cargo +nightly fuzz run spec_roundtrip
```

On Linux, `cargo-fuzz` may default to a `musl` triple that is not installed. Pass your **host** GNU triple (e.g. `x86_64-unknown-linux-gnu` or `aarch64-unknown-linux-gnu`) and, if linking fails, disable the sanitizer:

```sh
cargo +nightly fuzz run spec_roundtrip --target "$(rustc -vV | sed -n 's/^host: //p')" --sanitizer none -- -runs=1000 -max_len=1048576
```

The harness uses `pretty_assertions::assert_eq!` when a round-trip mismatch is detected.
