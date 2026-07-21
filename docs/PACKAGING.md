# OpenWrt packaging & size profiles

Кратко: профили Cargo, feature-срезы, пути пакетов.  
**Полная инструкция по сборке `.ipk` / `.apk` и LuCI:** [BUILD_OPENWRT.md](BUILD_OPENWRT.md).

## Пакеты в репозитории

| Путь | Пакет | Arch |
|------|--------|------|
| `package/openwrt/` | `openstream-engine` (бинарь + init + UCI) | target (aarch64 / …) |
| `luci-app-openstream/` | `luci-app-openstream` | `all` (Lua) |

Версия пакета = workspace (`0.4.2`).

## Профили Cargo

| Profile | Назначение |
|---------|------------|
| `release` | LTO + `opt-level=z` + strip + `panic=abort` — размер для OpenWrt |
| `release-fast` | `opt-level=3` — стенд/x86 |

```bash
# Cortex-A53 / aarch64 OpenWrt (musl)
cargo zigbuild --release -p streamproxyd --target aarch64-unknown-linux-musl

# Только Twitch (слабые SoC)
cargo zigbuild --release -p streamproxyd --target aarch64-unknown-linux-musl \
  --no-default-features --features slim-twitch
```

См. [PERFORMANCE.md](PERFORMANCE.md), [BUILD_OPENWRT.md](BUILD_OPENWRT.md).

## Проверка размера

```bash
ls -lh target/aarch64-unknown-linux-musl/release/streamproxyd
# runtime: RSS ≤ ~30 МБ под нагрузкой (полевые цифры — в PERFORMANCE.md)
```
