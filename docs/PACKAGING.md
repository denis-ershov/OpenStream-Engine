# OpenWrt packaging & size profiles

Кратко: профили Cargo, feature-срезы, пути пакетов.  
**Полная инструкция:** [BUILD_OPENWRT.md](BUILD_OPENWRT.md). Оглавление: [INDEX.md](INDEX.md).

## Пакеты в репозитории

| Путь | Пакет | Arch |
|------|--------|------|
| `package/openwrt/` | `openstream-engine` (бинарь + init + UCI) | target (aarch64 / …) |
| `luci-app-openstream/` | `luci-app-openstream` | `all` (Lua) |
| `luci-app-openstream` i18n | `luci-i18n-openstream-ru` | `all` |

Версия пакета = workspace (`0.4.2`), release IPK сейчас **14**.

### Значимые release IPK (0.4.2)

| Rel | Суть |
|-----|------|
| 7+ | LuCI Size/Description через встроенный `Packages.gz` |
| 8 | opkg meta только в engine (без clash с luci) |
| 9 | Playlist Edge + hostlists |
| 12+ | `+x` на бинаре (Windows pack), postinst chmod |
| 13 | rustls `CryptoProvider` (Edge TLS) |
| **14** | Edge master rewrite / auto `proxy_base` из Host |

`scripts/pack-ipk-a53.sh` оставляет в `dist/.../ipk/` только текущий release; `fix-ipk-exec-bits.py` чинит mode исполняемых файлов в архиве.

### Режим ловли Twitch

Default: **`edge`** — `GET /twitch/<channel>`, без клиентского CA; сегменты с CDN.  
Обязательно для strip: **Public Edge URL** / LAN Host (см. [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md)).  
Hostlists: `/usr/share/openstream/hostlists/*.txt` → compose → `/var/run/openstream/hostlist-effective.txt`.  
Legacy MITM: `mode=transparent` + CA. См. [COEXISTENCE.md](COEXISTENCE.md), [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md).

### LuCI Software: Size / Description

После `opkg install ./….ipk` поля в «Установлено» пустые, если пакета нет в opkg available lists.  
`scripts/pack-ipk-a53.sh` (r7+) встраивает `Packages.gz` и `/usr/libexec/openstream-refresh-opkg-list`  
(вызов из postinst и `streamproxyd` start). `Size:` в CONTROL по-прежнему нельзя.

## Профили Cargo

| Profile | Назначение |
|---------|------------|
| `release` | LTO + `opt-level=z` + strip + `panic=abort` — размер для OpenWrt |
| `release-fast` | `opt-level=3` — стенд/x86 |

`panic=abort`: любой panic в worker (например rustls без provider) **убивает процесс** — поэтому CryptoProvider обязателен с r13.

```bash
cargo zigbuild --release -p streamproxyd --target aarch64-unknown-linux-musl

cargo zigbuild --release -p streamproxyd --target aarch64-unknown-linux-musl \
  --no-default-features --features slim-twitch

SKIP_BUILD=1 OPENSTREAM_RELEASE=14 bash scripts/pack-ipk-a53.sh
```

См. [PERFORMANCE.md](PERFORMANCE.md), [BUILD_OPENWRT.md](BUILD_OPENWRT.md).

## Проверка размера

```bash
ls -lh target/aarch64-unknown-linux-musl/release/streamproxyd
# runtime: RSS ≤ ~30 МБ под нагрузкой (полевые цифры — в PERFORMANCE.md)
```
