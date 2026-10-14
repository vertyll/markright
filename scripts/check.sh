#!/usr/bin/env bash
# Runs every check CI runs, from the repository root, and stops at the first that fails: the formatting, Clippy on
# the workspace, on the library without its features and on the WebAssembly playground, the tests of every crate, the
# API documentation, markright on this repository's own documents, and the type check and tests of the site's
# scripts, Biome's formatting and lints of the scripts and styles. It leaves site/assets/lcov.info, the scripts'
# coverage, for tools that read it.
#
# Usage: bash scripts/check.sh
# Needs: Rust with rustfmt and Clippy, and Node; the wasm32-unknown-unknown target for the playground's lints, which
# are skipped without it. The tools of the scripts and styles are fetched at the versions pinned in
# site/assets/package.json.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --package markright-core --no-default-features --all-targets --locked -- -D warnings
if rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown; then
  cargo clippy --package markright-wasm --target wasm32-unknown-unknown --locked -- -D warnings
else
  echo "check.sh: wasm32-unknown-unknown is not installed; skipping the playground's lints" >&2
fi
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --quiet
cargo run --quiet --package markright --locked -- check
(cd site/assets && npm ci --ignore-scripts --no-audit --no-fund && npm run check && node --test --experimental-test-coverage \
  --test-reporter=spec --test-reporter-destination=stdout \
  --test-reporter=lcov --test-reporter-destination=lcov.info)
