#!/usr/bin/bash

cargo build --workspace --manifest-path src/Cargo.toml
cargo run --manifest-path src/pop_dm/Cargo.toml