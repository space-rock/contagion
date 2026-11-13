#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT/scripts/env.sh"

fail() { echo "$1" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || fail "docker is required"
docker info >/dev/null 2>&1 || fail "the docker daemon is not running, start Docker first"
grep -Eq '^repository = "https?://' "$ROOT/contract/Cargo.toml" || fail "add repository = \"<github url>\" to [package] in contract/Cargo.toml"
[ -z "$(git -C "$ROOT" status --porcelain)" ] || fail "commit your changes first: the build is tied to the commit"
git -C "$ROOT" remote get-url origin >/dev/null 2>&1 || fail "add the GitHub remote as origin first"
git -C "$ROOT" fetch --quiet origin
[ -n "$(git -C "$ROOT" branch -r --contains HEAD)" ] || fail "push the current commit first: verifiers fetch it from GitHub"

python3 -I "$ROOT/scripts/build-icon.py"
cd "$ROOT/contract"
cargo near build reproducible-wasm --out-dir "$ROOT/target/near/contagion_contract"
echo "reproducible wasm: $ROOT/target/near/contagion_contract/contagion_contract.wasm"
