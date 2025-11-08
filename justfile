#!/usr/bin/env just --justfile

set shell := ['bash', '-c']

main_crate := 'martin'

export CARGO_TERM_COLOR := 'always'

#export RUST_LOG := 'debug'
#export RUST_LOG := 'sqlx::query=info,trace'
#export RUST_BACKTRACE := '1'

# if running in CI, treat warnings as errors by setting RUSTFLAGS and RUSTDOCFLAGS to '-D warnings' unless they are already set
# Use `CI=true just ci-test` to run the same tests as in GitHub CI.
# Use `just env-info` to see the current values of RUSTFLAGS and RUSTDOCFLAGS
ci_mode := if env('CI', '') != '' {'1'} else {''}
# cargo-binstall needs a workaround due to caching
# ci_mode might be manually set by user, so re-check the env var
binstall_args := if env('CI', '') != '' {'--no-track'} else {''}
export RUSTFLAGS := env('RUSTFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUSTDOCFLAGS := env('RUSTDOCFLAGS', if ci_mode == '1' {'-D warnings'} else {''})
export RUST_BACKTRACE := env('RUST_BACKTRACE', if ci_mode == '1' {'1'} else {''})

@_default:
    {{just_executable()}} --list

# Run integration tests and save its output as the new expected output (ordering is important)
bless: clean-gen clean-test bless-insta gen

# Run integration tests and save its output as the new expected output
bless-insta *args:  (cargo-install 'cargo-insta')
    cargo insta test --accept --workspace {{args}}
    {{just_executable()}} gen

# Quick compile without building a binary
check: (cargo-install 'cargo-hack')
    cargo hack check --all-targets --each-feature --workspace

# Test documentation generation
check-doc:  (docs '')

# Run all tests as expected by CI
ci-test: env-info test-fmt clippy check-doc test check && assert-git-is-clean

# Perform  cargo clean  to delete all build files
clean: clean-test
    cargo clean

# Run cargo clippy to lint the code
clippy *args:
    cargo clippy --workspace --all-targets {{args}}

# Generate code coverage report. Will install `cargo llvm-cov` if missing.
coverage *args='--no-clean --open':  (cargo-install 'cargo-llvm-cov') clean
    #!/usr/bin/env bash
    set -euo pipefail
    if ! rustup component list | grep llvm-tools-preview > /dev/null; then \
        echo "llvm-tools-preview could not be found. Installing..." ;\
        rustup component add llvm-tools-preview ;\
    fi

    source <(cargo llvm-cov show-env --export-prefix)
    cargo llvm-cov clean --workspace

    echo "::group::Unit tests"
    {{just_executable()}} test-cargo --all-targets
    echo "::endgroup::"

    # echo "::group::Documentation tests"
    # {{just_executable()}} test-doc <- deliberately disabled until --doctest for cargo-llvm-cov does not hang indefinitely
    # echo "::endgroup::"

    {{just_executable()}} test-int

    cargo llvm-cov report {{args}}

# Build and open code documentation
docs *args='--open':
    DOCS_RS=1 cargo doc --no-deps {{args}} --workspace

# Print environment info
env-info:
    @echo "Running {{if ci_mode == '1' {'in CI mode'} else {'in dev mode'} }} on {{os()}} / {{arch()}}"
    @echo "PWD $(pwd)"
    {{just_executable()}} --version
    rustc --version
    cargo --version
    rustup --version
    @echo "RUSTFLAGS='$RUSTFLAGS'"
    @echo "RUSTDOCFLAGS='$RUSTDOCFLAGS'"
    @echo "RUST_BACKTRACE='$RUST_BACKTRACE'"
    npm --version
    node --version

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if (rustup toolchain list | grep nightly && rustup component list --toolchain nightly | grep rustfmt) &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Reformat all Cargo.toml files using cargo-sort
fmt-toml *args: (cargo-install 'cargo-sort')
    cargo sort --workspace --order package,lib,bin,bench,features,dependencies,build-dependencies,dev-dependencies {{args}}

# Generate a new spec file
gen: clean-gen
    cargo run --bin generate_spec
    {{ just_executable() }} fmt

# Run cargo fmt and cargo clippy
lint: fmt clippy

# Run all tests using a test database
test: (test-cargo '--all-targets') test-doc

# Run Rust unit tests (cargo test)
test-cargo *args:
    cargo test {{args}}

# Run Rust doc tests
test-doc *args:
    cargo test --doc {{args}}

# Test code formatting
test-fmt: (cargo-install 'cargo-sort') && (fmt-toml '--check' '--check-format')
    cargo fmt --all -- --check

# Update all dependencies, including breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:
    cargo +nightly -Z unstable-options update --breaking
    cargo update

# Make sure the git repo has no uncommitted changes
[private]
assert-git-is-clean:
    @if [ -n "$(git status --untracked-files --porcelain)" ]; then \
      >&2 echo "ERROR: git repo is no longer clean. Make sure compilation and tests artifacts are in the .gitignore, and no repo files are modified." ;\
      >&2 echo "######### git status ##########" ;\
      git status ;\
      git --no-pager diff ;\
      exit 1 ;\
    fi

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD='' *args='':
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v $COMMAND > /dev/null; then
        if ! command -v cargo-binstall > /dev/null; then
            echo "$COMMAND could not be found. Installing it with    cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}"
            cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}
        else
            echo "$COMMAND could not be found. Installing it with    cargo binstall ${INSTALL_CMD:-$COMMAND} {{binstall_args}} --locked"
            cargo binstall ${INSTALL_CMD:-$COMMAND} {{binstall_args}} --locked
        fi
    fi

# Delete test output files
[private]
clean-test:
    rm -rf tests/output

# Remove a previusly generated spec file
[private]
clean-gen:
    rm maplibre-style-spec/src/spec/mod.rs
    echo "pub struct MaplibreStyleSpecification;" > maplibre-style-spec/src/spec/mod.rs

# Install SQLX cli if not already installed.
[private]
install-sqlx:  (cargo-install 'cargo-sqlx' 'sqlx-cli' '--no-default-features' '--features' 'sqlite,native-tls')
