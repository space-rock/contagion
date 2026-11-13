#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACT="${1:?usage: burn-keys.sh <contract-name>}"

echo "This deletes every access key of $CONTRACT.testnet. The contract can never be changed or redeployed afterwards."
read -r -p "Type the contract name to confirm: " CONFIRM
[ "$CONFIRM" = "$CONTRACT" ] || { echo "aborted" >&2; exit 1; }

cd "$ROOT"
cargo run --quiet -p contagion-e2e --example testnet -- burn "$CONTRACT"
