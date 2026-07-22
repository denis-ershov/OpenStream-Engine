# Сборка OpenWrt / LuCI пакетов

Документ описывает, как собрать **OpenStream Engine** и **LuCI** для роутеров, в том числе **ARM Cortex-A53** (`aarch64`), в форматах:

| Формат | Менеджер | Типичные прошивки |
|--------|----------|-------------------|
| `.ipk` | `opkg` | OpenWrt ≤ 24.10, многие форки |
| `.apk` | `apk` | OpenWrt **25.12+** (не Android APK) |

Связанные файлы: `package/openwrt/`, `luci-app-openstream/`, [PACKAGING.md](PACKAGING.md), [PERFORMANCE.md](PERFORMANCE.md), [INDEX.md](INDEX.md).

Актуальный IPK release: **14** (`OPENSTREAM_RELEASE` / `PKG_RELEASE`).

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
# ipk (пример: 0.4.2-14)
opkg install --force-reinstall ./openstream-engine_0.4.2-14_aarch64_cortex-a53.ipk
opkg install ./luci-app-openstream_0.4.2-14_all.ipk

# apk (25.12+)
apk add openstream-engine luci-app-openstream

/etc/init.d/streamproxyd enable
/etc/init.d/streamproxyd start

wget -qO- http://127.0.0.1:18080/api/status
```

LuCI: **Services → OpenStream**.

**Default = Playlist Edge (без CA):**

1. Задайте **Public Edge URL** = `http://<LAN_IP>:18080` (например `http://192.168.8.1:18080`).
2. VLC / mpv: `http://<LAN_IP>:18080/twitch/<channel>`.
3. В master должны быть nested URL `http://LAN:18080/https://…` — иначе strip media не сработает.
4. Проверка: [COEXISTENCE.md](COEXISTENCE.md), [PERFORMANCE.md](PERFORMANCE.md).

Legacy MITM (`mode=transparent`): HTTP proxy не нужен; CA с роутера на клиенты; узкий hostlist.  
Соседи (zapret/podkop): [COEXISTENCE.md](COEXISTENCE.md).

Оглавление документации: [INDEX.md](INDEX.md).

---

## Формат `.ipk` (OpenWrt 24.10 / opkg)

Официальный `scripts/ipkg-build` (**не** Debian `ar`):

```text
.ipk = gzip( tar( ./debian-binary, ./data.tar.gz, ./control.tar.gz ) )
```

- `data.tar.gz` / `control.tar.gz`: `tar --format=gnu`
- каталог пакета: файлы + подкаталог `CONTROL/` (`control`, `conffiles`, `postinst`, `prerm`)
- проверка: `file *.ipk` → **`gzip compressed data`**, не `Debian binary package`

У нас: vendored `scripts/ipkg-build` + `scripts/pack-ipk-a53.sh`.

> Release `-1`/`-2` были в формате `ar` → LuCI/opkg 24.x: `Malformed package file`. Нужен **`-3`+**.  
> **`Size:` в CONTROL** → `Checksum or size mismatch` (нужен **-6+**, Size только в `Packages`).  
> **LuCI Size/Description = `-`** после локальной установки: opkg `status` не хранит Description; LuCI берёт поля из available lists. Release **-7+** кладёт `Packages.gz` в пакет и пишет список через `openstream-refresh-opkg-list`.


```bash
./scripts/build-a53.sh              # full features (нужен zig / zigbuild)
./scripts/build-a53.sh --slim
./scripts/build-a53.sh --copy-pkg   # → package/openwrt/streamproxyd

# Упаковка .ipk без SDK (OpenWrt ≤24.10 / opkg):
SKIP_BUILD=1 bash scripts/pack-ipk-a53.sh
bash scripts/pack-ipk-a53.sh --slim

# Артефакты: dist/openwrt-24.10-a53/{bin,ipk}/
```

На Windows host удобнее Docker `messense/cargo-zigbuild` (см. `dist/openwrt-24.10-a53/README.md`).  
Скрипт **не** создаёт `.apk`: для OpenWrt 25.12+ используйте SDK (§3.4).

### Готовые артефакты 0.4.2

После сборки (`scripts/pack-ipk-a53.sh`):

| Пакет | Arch |
|-------|------|
| `openstream-engine_0.4.2-N_aarch64_cortex-a53.ipk` | aarch64_cortex-a53 |
| `luci-app-openstream_0.4.2-N_all.ipk` | all (EN) |
| `luci-i18n-openstream-ru_0.4.2-N_all.ipk` | all (RU `.lmo`) |

В OpenWrt SDK `luci.mk` сам собирает `luci-i18n-openstream-ru` из `po/ru/`.

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
| `Permission denied` / exit **126** | Нет `+x` на `/usr/bin/streamproxyd` (pack с Windows); r12+ postinst; `chmod 0755` |
| Panic `CryptoProvider` / crash loop на `/twitch/` | Бинарь &lt; r13; нужен rustls `ring::install_default` |
| `playlists&gt;0`, `ads_found=0`, нет nested в master | Нет rewrite — задайте Public Edge URL / откройте с LAN Host (r14+) |
| `Checksum or size mismatch` | В CONTROL не должно быть `Size:` (только в `Packages`); нужен release ≥6 |
| LuCI Installed: Size/Description = `-` | Локальный `.ipk` не в opkg lists; нужен release ≥7 + `/usr/libexec/openstream-refresh-opkg-list` |
| `check_data_file_clashes` Packages.gz | meta только в engine (r8+), не дублировать в luci-app |

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
