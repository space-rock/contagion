#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WASM="$ROOT/target/near/contagion_contract/contagion_contract.wasm"

if [ ! -f "$WASM" ]; then
  "$ROOT/scripts/build-contract.sh"
fi

cd "$ROOT"
cargo run --quiet -p contagion-e2e --example demo
