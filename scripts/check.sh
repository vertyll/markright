#!/usr/bin/env bash
# Runs every check CI runs, from the repository root, and stops at the first that fails: the formatting, Clippy on
# the workspace, on the library without its features and on the WebAssembly playground, the tests of every crate, and
# markright on this repository's own documents.
#
# Usage: bash scripts/check.sh
# Needs: Rust with rustfmt and Clippy; the wasm32-unknown-unknown target for the playground's lints, which are skipped
# without it.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --package markright --no-default-features --all-targets --locked -- -D warnings
if rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown; then
  cargo clippy --package markright-wasm --target wasm32-unknown-unknown --locked -- -D warnings
else
  echo "check.sh: wasm32-unknown-unknown is not installed; skipping the playground's lints" >&2
fi
cargo test --workspace --locked
cargo run --quiet --package markright-cli --locked -- run
