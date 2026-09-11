#!/bin/sh
# Проверка синтаксиса генерируемого набора nftables.
#
# Зачем: юнит-тесты проверяют наличие подстрок в выводе генератора, но НЕ то,
# что полученный файл принимает ядро. Именно так в проект попали правила с
# `redirect` в цепочке type filter и `reject` в prerouting — файл не загружался
# целиком, а ошибка глушилась `2>/dev/null`. Этот скрипт прогоняет реальный
# парсер nftables.
#
# Использование:
#   scripts/check-nft-syntax.sh            # локально (нужен nft)
#   CI: unshare -rn scripts/check-nft-syntax.sh   # без root-прав
#
# Требуется установленный `nft` (пакет nftables).

set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v nft >/dev/null 2>&1; then
	echo "SKIP: nft не установлен — проверка синтаксиса пропущена" >&2
	exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
	echo "SKIP: cargo не найден — нечем сгенерировать правила" >&2
	exit 0
fi

OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

echo "==> Генерация набора правил (все ветки: bypass, zapret2, vpn, streamproxy, reject)"
OPENSTREAM_NFT_OUT="$OUT_DIR/generated.nft" \
	cargo run --quiet --example dump_nft -p openstream-backend-openwrt

if [ ! -s "$OUT_DIR/generated.nft" ]; then
	echo "FAIL: генератор не создал файл правил" >&2
	exit 1
fi

echo "==> Проверка статической divert-таблицы"
if ! nft -c -f package/openwrt/files/openstream.nft; then
	echo "FAIL: package/openwrt/files/openstream.nft не проходит проверку" >&2
	exit 1
fi
echo "OK: divert-таблица валидна"

echo "==> Проверка сгенерированного набора"
if ! nft -c -f "$OUT_DIR/generated.nft"; then
	echo "FAIL: сгенерированный набор правил отвергнут парсером nftables" >&2
	echo "--- вывод ---"
	cat "$OUT_DIR/generated.nft"
	exit 1
fi

echo "OK: сгенерированный набор валиден"
