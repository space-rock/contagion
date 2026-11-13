#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOL="$ROOT/.toolchain"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) TRIPLE=aarch64-apple-darwin ;;
  Darwin-x86_64) TRIPLE=x86_64-apple-darwin ;;
  Linux-x86_64) TRIPLE=x86_64-unknown-linux-gnu ;;
  Linux-aarch64) TRIPLE=aarch64-unknown-linux-gnu ;;
  *) echo "unsupported platform $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

mkdir -p "$TOOL"
if [ ! -x "$TOOL/cargo/bin/rustc" ]; then
  curl -sSf -o "$TOOL/rustup-init" "https://static.rust-lang.org/rustup/dist/$TRIPLE/rustup-init"
  curl -sSf -o "$TOOL/rustup-init.sha256" "https://static.rust-lang.org/rustup/dist/$TRIPLE/rustup-init.sha256"
  [ "$(cut -d' ' -f1 "$TOOL/rustup-init.sha256")" = "$(sha256 "$TOOL/rustup-init")" ] || { echo "rustup-init checksum mismatch" >&2; exit 1; }
  chmod +x "$TOOL/rustup-init"
  RUSTUP_HOME="$TOOL/rustup" CARGO_HOME="$TOOL/cargo" "$TOOL/rustup-init" -y --no-modify-path \
    --profile minimal --default-toolchain stable --target wasm32-unknown-unknown
fi

source "$ROOT/scripts/env.sh"
CARGO_NEAR_VERSION=0.23.0
if ! cargo near --version 2>/dev/null | grep -q " $CARGO_NEAR_VERSION\$"; then
  cargo install cargo-near --locked --version "$CARGO_NEAR_VERSION" --force
fi

echo "toolchain ready in $TOOL (nothing outside this folder was changed)"
