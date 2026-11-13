#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PRESET="${1:?usage: deploy-testnet.sh <live|demo> <contract-name> <patient-zero-name>...}"
CONTRACT="${2:?missing contract name}"
shift 2
[ "$#" -ge 1 ] || { echo "at least one patient zero name is required" >&2; exit 1; }

"$ROOT/scripts/build-contract.sh"
cd "$ROOT"
cargo run --quiet -p contagion-e2e --example testnet -- deploy "$PRESET" "$CONTRACT" "$@"
