#!/usr/bin/env bash
# Build host po2lmo (Linux). Prefer running inside Docker on Windows.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/scripts/po2lmo"
OUT="${1:-$ROOT/scripts/bin/po2lmo}"
mkdir -p "$(dirname "$OUT")"
make -C "$SRC" clean po2lmo
cp -f "$SRC/po2lmo" "$OUT"
chmod +x "$OUT"
echo "Built $OUT"
