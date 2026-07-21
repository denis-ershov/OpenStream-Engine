# Сборка OpenWrt / LuCI пакетов

Документ описывает, как собрать **OpenStream Engine** и **LuCI** для роутеров, в том числе **ARM Cortex-A53** (`aarch64`), в форматах:

| Формат | Менеджер | Типичные прошивки |
|--------|----------|-------------------|
| `.ipk` | `opkg` | OpenWrt ≤ 24.10, многие форки |
| `.apk` | `apk` | OpenWrt **25.12+** (не Android APK) |

Связанные файлы: `package/openwrt/`, `luci-app-openstream/`, [PACKAGING.md](PACKAGING.md), [PERFORMANCE.md](PERFORMANCE.md).

---

## 1. Что собирается

```
openstream-engine          → /usr/bin/streamproxyd + init + UCI + nft + uci2yaml
luci-app-openstream        → веб-UI (arch=all, зависит от openstream-engine)
```

Бинарь — **Rust musl**, статически линкуется с плагинами (ADR 0001).  
LuCI — **чистый Lua**, кросс-компиляция CPU не нужна.

### Целевые SoC (рекомендуемые triple)

| Железо | Rust target | Примечание |
|--------|-------------|------------|
| **Cortex-A53** 64-bit | `aarch64-unknown-linux-musl` | GL.iNet MT3000, NanoPi R2S/R4S aarch64, Xiaomi AX*, … |
| ARMv7 (Cortex-A7/A9) | `armv7-unknown-linux-musleabihf` | Старые 32-bit |
| MIPS softfloat | `mipsel-unknown-linux-musl` | Очень старые; предпочтительно `slim-twitch` |

Cortex-A53 в OpenWrt почти всегда = **aarch64** + **musl**.

---

## 2. Host: кросс-сборка бинаря (без полного SDK)

Нужны: Rust stable, `cargo-zigbuild` (или OpenWrt SDK rustc), Zig.

```bash
# из корня репозитория
rustup target add aarch64-unknown-linux-musl
cargo install cargo-zigbuild --locked

# Полная сборка (Twitch + HLS + DASH)
cargo zigbuild --release -p streamproxyd \
  --target aarch64-unknown-linux-musl

# Slim — только Twitch (меньше .ipk/.apk)
cargo zigbuild --release -p streamproxyd \
  --target aarch64-unknown-linux-musl \
  --no-default-features --features slim-twitch

ls -lh target/aarch64-unknown-linux-musl/release/streamproxyd
file target/aarch64-unknown-linux-musl/release/streamproxyd
# ожидается: ELF 64-bit LSB executable, ARM aarch64, statically linked / musl
```

Альтернатива без Zig (если есть `aarch64-linux-musl-gcc`):

```bash
cargo build --release -p streamproxyd --target aarch64-unknown-linux-musl
```

Готовый бинарь положите туда, откуда его подхватит пакет (см. §3):

```bash
cp target/aarch64-unknown-linux-musl/release/streamproxyd \
  package/openwrt/streamproxyd
```

Или задайте путь:

```bash
export OPENSTREAM_BIN="$PWD/target/aarch64-unknown-linux-musl/release/streamproxyd"
```

Скрипт-хелпер: `scripts/build-a53.sh` (см. §7).

---

## 3. Пакет `openstream-engine` через OpenWrt buildroot / SDK

### 3.1 Подключить feed

В каталоге OpenWrt (или SDK):

```bash
# вариант A: local feed
mkdir -p feeds/openstream
# скопировать/symlink:
#   package/openwrt  → feeds/openstream/openstream-engine
#   luci-app-openstream → feeds/openstream/luci-app-openstream

# feeds.conf.default или feeds.conf:
# src-link openstream /path/to/OpenStream-Engine/feeds-layout
```

Удобная раскладка (пример):

```text
feeds/openstream/
  openstream-engine/     ← содержимое package/openwrt/
  luci-app-openstream/   ← содержимое luci-app-openstream/
```

```bash
./scripts/feeds update openstream
./scripts/feeds install -a -p openstream
```

### 3.2 Prebuilt binary

`package/openwrt/Makefile` **не** компилирует Rust внутри buildroot (нет гарантированного rustc).  
Перед `make package/.../compile` положите бинарь:

```bash
# в дереве пакета (относительно Makefile пакета):
cp "$OPENSTREAM_BIN" feeds/openstream/openstream-engine/streamproxyd
# или:
export OPENSTREAM_BIN=/abs/path/to/streamproxyd
```

Makefile копирует `$(OPENSTREAM_BIN)` или `./streamproxyd` в staging.

### 3.3 Сборка `.ipk` (opkg / OpenWrt 23.05–24.10)

Выберите target под A53, например:

```text
Target System: MediaTek ARM / Rockchip / Qualcomm … (aarch64_cortex-a53 и т.п.)
Target Profile: ваш роутер
```

```bash
make menuconfig
# Network → openstream-engine
# LuCI → Applications → luci-app-openstream

make package/openstream-engine/compile V=s
make package/luci-app-openstream/compile V=s
```

Артефакты (пути зависят от target):

```text
bin/packages/<arch>/openstream/openstream-engine_0.4.2-1_<arch>.ipk
bin/packages/<arch>/openstream/luci-app-openstream_*.ipk
# luci часто в luci feed: bin/packages/<arch>/luci/...
```

Установка на роутере:

```bash
opkg update
opkg install ./openstream-engine_0.4.2-1_aarch64_cortex-a53.ipk
opkg install ./luci-app-openstream_*.ipk
```

Имя arch в `.ipk` задаёт OpenWrt (`aarch64_cortex-a53`, `aarch64_generic`, …) — важно совпадение с `opkg print-architecture`.

### 3.4 Сборка `.apk` (apk / OpenWrt 25.12+)

Тот же Makefile пакета: buildroot сам упакует в **`.apk`**, если SDK/ветка с пакетным менеджером apk.

```bash
make package/openstream-engine/compile V=s
make package/luci-app-openstream/compile V=s

# типичный выход:
# bin/packages/<arch>/openstream/openstream-engine-0.4.2-r1.apk
# (точное имя зависит от include/package-pack.mk ветки)
```

На устройстве:

```bash
apk add --allow-untrusted ./openstream-engine-*.apk
apk add --allow-untrusted ./luci-app-openstream-*.apk
# или через custom feed + packages.adb
```

> **Не путать** с Android Application Package. Это Alpine/OpenWrt `apk` v3.

### 3.5 ImageBuilder (включить в прошивку)

```bash
make image PROFILE="..." PACKAGES="openstream-engine luci-app-openstream"
```

Нужен feed с уже собранными пакетами или локальный `bin/packages`.

---

## 4. Сборка только LuCI

`luci-app-openstream` — `LUCI_PKGARCH:=all`, зависит от `+openstream-engine`.

```bash
# в OpenWrt buildroot после feeds install
make package/luci-app-openstream/compile V=s
```

Без полного SDK можно собрать вручную архив `all`-пакета (структура как у LuCI), но **рекомендуется** SDK/buildroot — корректные CONTROL/metadata для ipk и apk.

Локализация:

```bash
# po → lmo обычно делает luci.mk при сборке пакета
ls luci-app-openstream/po/{en,ru}/openstream.po
```

---

## 5. Feature-срезы и размер

| Features | Когда |
|----------|--------|
| default (`plugin-twitch,plugin-hls,plugin-dash`) | Универсальный роутер A53 с запасом flash |
| `slim-twitch` | Только Twitch, минимум flash/RAM |

Соберите нужный бинарь **до** упаковки (§2), затем один раз `make package/openstream-engine/compile`.

Проверка на устройстве после установки:

```bash
streamproxyd --help
ps | grep streamproxyd
cat /proc/$(pidof streamproxyd)/status | grep VmRSS
```

Цели — [PERFORMANCE.md](PERFORMANCE.md).

---

## 6. Установка и первый запуск

```bash
# ipk
opkg install openstream-engine luci-app-openstream

# apk (25.12+)
apk add openstream-engine luci-app-openstream

/etc/init.d/streamproxyd enable
/etc/init.d/streamproxyd start

# API
wget -qO- http://127.0.0.1:18080/api/status
```

LuCI: **Services → OpenStream**.  
Клиенты: HTTP(S) proxy `http://<router-ip>:18080`.  
HTTPS MITM: установить CA с роутера на устройства (см. [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md)).

Сосуществование с zapret/podkop: [COEXISTENCE.md](COEXISTENCE.md) — режим **explicit** по умолчанию.

---

## 7. Скрипт host-сборки A53

```bash
./scripts/build-a53.sh              # full features
./scripts/build-a53.sh --slim       # slim-twitch
./scripts/build-a53.sh --copy-pkg   # + скопировать в package/openwrt/streamproxyd
```

Дальше — сборка пакета в SDK (§3). Скрипт **не** создаёт `.ipk`/`.apk` сам: формат и ABI зависят от ветки OpenWrt.

---

## 8. CI

`.github/workflows/ci.yml`:

- `cargo test` / clippy
- artifact `streamproxyd-aarch64-musl` (Cortex-A53-совместимый)
- soft-fail armv7 via zigbuild

Скачанный artifact можно подставить как `OPENSTREAM_BIN` в SDK.

---

## 9. Частые ошибки

| Симптом | Причина |
|---------|---------|
| `opkg: Architecture not OK` | Бинарь/пакет под другой arch (`aarch64_generic` vs `aarch64_cortex-a53`) — пересоберите под тот же SDK target |
| `apk: unexpected end / bad package` | Поставили `.ipk` в систему с `apk` или наоборот |
| Пакет без `streamproxyd` / пустой `/usr/bin` | Не положили prebuilt до `Compile` |
| LuCI есть, демон нет | Не установлен `openstream-engine` |
| `Exec format error` | Собран не musl / не aarch64 (например glibc host) |

---

## 10. Чеклист релиза A53

1. [ ] `PKG_VERSION` в `package/openwrt/Makefile` = workspace version  
2. [ ] `cargo zigbuild … aarch64-unknown-linux-musl` (+ slim при необходимости)  
3. [ ] `file` / `readelf -h` = AArch64  
4. [ ] SDK target = ваш `aarch64_cortex-a53` (или эквивалент)  
5. [ ] `make package/openstream-engine/compile` → `.ipk` **или** `.apk`  
6. [ ] `make package/luci-app-openstream/compile`  
7. [ ] Установка на устройство + `/api/status` + VmRSS  
8. [ ] Запись размера в [PERFORMANCE.md](PERFORMANCE.md) / changelog
