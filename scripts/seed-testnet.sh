#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACT="${1:?usage: seed-testnet.sh <contract-name> <patient-zero-name>...}"
shift
[ "$#" -ge 1 ] || { echo "at least one patient zero name is required" >&2; exit 1; }

cd "$ROOT"
cargo run --quiet -p contagion-e2e --example testnet -- seed "$CONTRACT" "$@"
