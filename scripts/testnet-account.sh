#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NAME="${1:?usage: testnet-account.sh <name> (creates <name>.testnet through the testnet faucet)}"
KEYS="$ROOT/.testnet/$NAME.json"
LOG="$ROOT/.testnet/$NAME.create.log"

mkdir -p "$ROOT/.testnet"
if [ -f "$KEYS" ]; then
  echo "keys for $NAME.testnet already exist at $KEYS"
  exit 0
fi

near account create-account sponsor-by-faucet-service "$NAME.testnet" \
  autogenerate-new-keypair print-to-terminal network-config testnet create > "$LOG" 2>&1 || {
  echo "creating $NAME.testnet failed, see $LOG" >&2
  exit 1
}

python3 -I - "$LOG" "$KEYS" "$NAME.testnet" <<'PY'
import json, re, sys
log, out, account = sys.argv[1:4]
text = open(log).read()
public = re.search(r"Public Key: (ed25519:\S+)", text).group(1)
secret = re.search(r"SECRET KEYPAIR: (ed25519:\S+)", text).group(1)
json.dump({"account_id": account, "public_key": public, "private_key": secret}, open(out, "w"))
PY
chmod 600 "$KEYS" "$LOG"
echo "created $NAME.testnet, keys saved to .testnet/$NAME.json"
