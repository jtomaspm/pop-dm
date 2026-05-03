cargo fmt --manifest-path src/Cargo.toml --all
cargo check --workspace --manifest-path src/Cargo.toml
cargo test --workspace --manifest-path src/Cargo.toml
cargo clippy --workspace --manifest-path src/Cargo.toml --all-targets --all-features
cargo fmt --check --manifest-path src/Cargo.toml --all

