#!/usr/bin/env bash
# Build the static site from the YAML data in ./data.
#
# Nothing about the programs is hardcoded in the generator: it re-reads ./data
# on every run, so after editing a YAML file you just re-run this.
#
#   ./build.sh            build the site into ./dist
#   ./build.sh --serve    build, then serve dist/ at http://localhost:8000
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

echo "▸ Building generator (release)…"
cargo build --release --manifest-path site-gen/Cargo.toml

echo "▸ Generating site…"
./site-gen/target/release/site-gen

if [[ "${1:-}" == "--serve" ]]; then
  echo "▸ Serving at http://localhost:8000  (Ctrl-C to stop)"
  cd dist
  python3 -m http.server 8000
fi
