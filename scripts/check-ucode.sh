#!/bin/sh
# Проверка ucode-скриптов OpenStream.
#
# ВАЖНО: `ucode -c` (только компиляция) НЕ выявляет часть ошибок, которые
# возникают при исполнении. В этих скриптах так были пропущены:
#   * импорт несуществующего символа `dir` из модуля `fs` (есть только `lsdir`);
#   * экранированный дефис (`\-`) внутри класса символов регулярного выражения.
# Поэтому скрипты не только компилируются, но и ИСПОЛНЯЮТСЯ на синтетическом
# окружении. Это и есть регрессионная защита от подобных ошибок.
#
# Использование: scripts/check-ucode.sh  (требуется ucode + ucode-mod-fs/uci)

set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v ucode >/dev/null 2>&1; then
	echo "SKIP: ucode не установлен — проверка пропущена" >&2
	exit 0
fi

PLUGIN="luci-app-openstream/root/usr/share/rpcd/ucode/openstream.uc"
RENDER="package/openwrt/files/openstream-render.uc"

fail=0

echo "==> Компиляция"
for f in "$PLUGIN" "$RENDER"; do
	if ucode -c "$f"; then
		echo "OK (compile): $f"
	else
		echo "FAIL (compile): $f" >&2
		fail=1
	fi
done

echo "==> Исполнение (выявляет ошибки, невидимые при компиляции)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Синтетическое окружение, как на роутере
mkdir -p /etc/openstream/rules /etc/config /tmp/dnsmasq.d 2>/dev/null || true
[ -f /etc/config/openstream ] || : > /etc/config/openstream 2>/dev/null || true

# Плагин RPC: исполнение файла целиком (завершается top-level return)
if ucode "$PLUGIN" >/dev/null 2>"$WORK/plugin.err"; then
	echo "OK (execute): $PLUGIN"
else
	echo "FAIL (execute): $PLUGIN" >&2
	cat "$WORK/plugin.err" >&2
	fail=1
fi

# Рендерер: должен сгенерировать файлы без ошибок
if ucode "$RENDER" >/dev/null 2>"$WORK/render.err"; then
	echo "OK (execute): $RENDER"
else
	echo "FAIL (execute): $RENDER" >&2
	cat "$WORK/render.err" >&2
	fail=1
fi

exit "$fail"
