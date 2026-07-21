# Performance notes

Цели OpenWrt (рядом с zapret/podkop): **RSS ≤ ~30 МБ**, parse/strip **≪ 1 мс** на типичном media playlist, CPU умеренный на A53.

## Criterion benches (host)

```bash
cargo bench -p ose-manifest --bench parse_hls
cargo bench -p ose-plugin-twitch --bench strip_twitch
cargo bench -p ose-dash --bench filter_mpd
```

Ориентиры на x86_64 desktop (порядок величины, не SLA устройства):

| Bench | Ожидание |
|-------|----------|
| `hls_parse_media` | десятки µs |
| `twitch_strip_midroll` | десятки–сотни µs |
| `dash_filter_ad_period` | сотни µs |

На aarch64/mips цифры выше; фиксировать после полевого замера.

## Feature flags (размер бинаря)

```bash
# Полная сборка (default)
cargo build --release -p streamproxyd

# Только Twitch (слабые mips)
cargo build --release -p streamproxyd --no-default-features --features slim-twitch

# HLS без DASH
cargo build --release -p streamproxyd --no-default-features --features "plugin-twitch,plugin-hls"
```

| Feature | Содержимое |
|---------|------------|
| `plugin-twitch` | Twitch strip |
| `plugin-hls` | Kick/Trovo/YouTube + `rules_file` |
| `plugin-dash` | DASH plugin + proxy MPD inspect |
| `slim-twitch` | alias → только `plugin-twitch` (с `--no-default-features`) |

Без `plugin-dash` пути `.mpd` **не** буферизуются как манифест (streaming/tunnel).

## Полевой замер RSS

```bash
# на роутере
ps | grep streamproxyd
cat /proc/$(pidof streamproxyd)/status | grep -E 'VmRSS|VmPeak'
```

Обновить этот документ фактическими цифрами после прогона на SoC.
