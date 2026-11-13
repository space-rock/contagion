#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== core, simulator and contract unit tests"
cargo test --workspace --exclude contagion-e2e

echo "== build the contract wasm"
"$ROOT/scripts/build-contract.sh"

echo "== end-to-end tests against a local NEAR sandbox"
cargo test -p contagion-e2e

echo "== web app"
(cd web && pnpm install --frozen-lockfile && pnpm test && pnpm build)

echo "all checks passed"
