#!/usr/bin/env bash
# Builds the documentation site: writes its pages from the repository into site/content, compiles the playground to
# WebAssembly with the licenses of what it includes, bundles the site's scripts and styles, then runs Zola in site/
# with the arguments given. When it serves, esbuild rebundles the scripts and styles on every change.
#
# Usage: bash scripts/site.sh          the same as serve
#        bash scripts/site.sh serve    serve it at http://127.0.0.1:1111/, rebuilt on every change of site/
#        bash scripts/site.sh build    build it into site/public
# It checks the tools first and says how to get any that is missing.
# Needs: Rust with the wasm32-unknown-unknown target; wasm-bindgen-cli at the version of the wasm-bindgen crate in
# Cargo.lock; Zola 0.23.6; Node. The tools of the scripts and styles are fetched at the versions pinned in
# site/assets/package.json.
set -euo pipefail
cd "$(dirname "$0")/.."

usage="usage: bash scripts/site.sh [serve | build] [Zola's options]"
[ "$#" -gt 0 ] || set -- serve
if [ "$1" != serve ] && [ "$1" != build ]; then
  echo "$usage" >&2
  exit 2
fi

# Every tool, before any work, with how to get the one that is missing or at another version.
bindgen=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
missing=0
need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "site.sh: $1 is missing; $2" >&2
    missing=1
  elif [ -n "${3:-}" ] && ! "$1" --version | grep -qF "$3"; then
    echo "site.sh: $1 is $("$1" --version), not $3; $2" >&2
    missing=1
  fi
}
need wasm-bindgen "install it with: cargo install wasm-bindgen-cli --version $bindgen --locked" "$bindgen"
need zola "get 0.23.6 from https://github.com/getzola/zola/releases/tag/v0.23.6" "0.23.6"
need npm "install Node from https://nodejs.org/"
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  echo "site.sh: the wasm32-unknown-unknown target is missing; add it with: rustup target add wasm32-unknown-unknown" >&2
  missing=1
fi
[ "$missing" -eq 0 ] || exit 2

cargo xtask site
cargo build --release --locked --package markright-wasm --target wasm32-unknown-unknown
rm -rf site/static/wasm
wasm-bindgen --target web --no-typescript --out-dir site/static/wasm \
  target/wasm32-unknown-unknown/release/markright_wasm.wasm
cargo xtask licenses markright-wasm wasm32-unknown-unknown > site/static/wasm/THIRD-PARTY-LICENSES.txt
(cd site/assets && npm ci --ignore-scripts --no-audit --no-fund && npm run build)
cd site
if [ "$1" = build ]; then
  exec zola "$@"
fi
# Serving, esbuild watches the scripts and styles beside Zola, and both stop with the script, however it is stopped.
trap 'trap - EXIT INT TERM; kill 0' EXIT INT TERM
(cd assets && npm run watch) &
zola "$@" &
wait "$!"
