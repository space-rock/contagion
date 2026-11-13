#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACT="${1:?usage: redeploy-testnet.sh <contract-name> [--reproducible] (needs the deployer key, so it only works before burn-keys.sh)}"

python3 -I "$ROOT/scripts/build-icon.py"
if [ "${2:-}" = "--reproducible" ]; then
  "$ROOT/scripts/build-reproducible.sh"
else
  "$ROOT/scripts/build-contract.sh"
fi
cd "$ROOT"
cargo run --quiet -p contagion-e2e --example testnet -- redeploy "$CONTRACT"
