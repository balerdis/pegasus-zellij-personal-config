#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
plugin_dir=$repo_root/plugins/pegasus-tab-bar
target=wasm32-wasip1
artifact=$plugin_dir/target/$target/release/pegasus-tab-bar.wasm

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustup >/dev/null 2>&1; then
    printf 'Rust and rustup are required. Install Rust with https://rustup.rs/ before building.\n' >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx "$target"; then
    rustup target add "$target"
fi

cargo build --manifest-path "$plugin_dir/Cargo.toml" --release --target "$target"
printf '%s\n' "$artifact"
