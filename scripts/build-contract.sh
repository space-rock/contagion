#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT/scripts/env.sh"

if ! command -v cargo-near >/dev/null 2>&1; then
  echo "cargo-near not found, run scripts/setup-toolchain.sh first" >&2
  exit 1
fi

cd "$ROOT/contract"
cargo near build non-reproducible-wasm --profile wasm --locked
OUT="$ROOT/target/near/contagion_contract"
cp "$OUT/contagion_contract_abi.json" "$ROOT/deploy/abi.json"
echo "wasm: $OUT/contagion_contract.wasm"
echo "abi:  $ROOT/deploy/abi.json"
