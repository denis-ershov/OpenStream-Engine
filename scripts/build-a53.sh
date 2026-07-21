#!/usr/bin/env bash
# Host cross-build streamproxyd for ARM Cortex-A53 (aarch64 musl).
# Does not produce .ipk/.apk — use OpenWrt SDK after this (see docs/BUILD_OPENWRT.md).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TARGET="${OPENSTREAM_TARGET:-aarch64-unknown-linux-musl}"
SLIM=0
COPY_PKG=0

for arg in "$@"; do
  case "$arg" in
    --slim) SLIM=1 ;;
    --copy-pkg) COPY_PKG=1 ;;
    -h|--help)
      echo "Usage: $0 [--slim] [--copy-pkg]"
      echo "  --slim      --no-default-features --features slim-twitch"
      echo "  --copy-pkg  copy binary to package/openwrt/streamproxyd"
      exit 0
      ;;
    *)
      echo "Unknown arg: $arg" >&2
      exit 1
      ;;
  esac
done

rustup target add "$TARGET" >/dev/null

FEATURES=()
if [[ "$SLIM" -eq 1 ]]; then
  FEATURES=(--no-default-features --features slim-twitch)
fi

if command -v cargo-zigbuild >/dev/null 2>&1 || cargo zigbuild -h >/dev/null 2>&1; then
  echo "==> cargo zigbuild --release -p streamproxyd --target $TARGET ${FEATURES[*]:-}"
  cargo zigbuild --release -p streamproxyd --target "$TARGET" "${FEATURES[@]}"
else
  echo "==> cargo-zigbuild not found; trying cargo build (needs musl linker)"
  cargo build --release -p streamproxyd --target "$TARGET" "${FEATURES[@]}"
fi

BIN="$ROOT/target/$TARGET/release/streamproxyd"
ls -lh "$BIN"
file "$BIN" || true

if [[ "$COPY_PKG" -eq 1 ]]; then
  cp -f "$BIN" "$ROOT/package/openwrt/streamproxyd"
  echo "Copied to package/openwrt/streamproxyd"
  echo "Set OPENSTREAM_BIN=$ROOT/package/openwrt/streamproxyd for SDK builds"
fi

echo "Done. Next: OpenWrt SDK → make package/openstream-engine/compile (ipk or apk)."
