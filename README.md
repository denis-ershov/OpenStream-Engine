# OpenStream Engine

Модульная платформа обработки HLS/DASH для OpenWrt. Плагины: Twitch, Kick/Trovo/YouTube (rules), DASH. SDK — статическая линковка (ADR 0001).

**Язык:** Rust · **Default:** Playlist Edge (без CA) · **IPK:** 0.4.2-14 · **Совместимость:** zapret / ByeDPI / podkop·netshift·forkop / SSClash / …

Twitch: Segment Stripping из media `m3u8` (freeze ≈ midroll). Сегменты — с CDN напрямую.

## Возможности

- **Playlist Edge:** `GET /twitch/<channel>` на роутере — чистый m3u8 без установки CA.
- Master rewrite → nested media (Public Edge URL / LAN Host) — иначе strip не видит midroll.
- Optional transparent MITM (nft + CA) для нативных приложений.
- HLS/DASH strip; hostlists per-service + custom + remote GitHub.
- Observability: `GET /metrics`, `GET /api/events`.
- OpenWrt package + LuCI.

## Ограничения

- При рекламе плеер **ждёт** следующий live-сегмент (freeze ≈ midroll). Seamless — opt-in (Stage G).
- Стоковое Twitch-приложение без companion и без CA не получит strip.
- Не обход DPI — доступ через zapret/podkop. См. [docs/COEXISTENCE.md](docs/COEXISTENCE.md), [docs/adr/0002-playlist-edge.md](docs/adr/0002-playlist-edge.md).

## Быстрый старт (разработка)

```bash
cargo build --release -p streamproxyd
./target/release/streamproxyd --config config.example.yaml
# Edge (для rewrite задайте proxy_public_url в конфиге):
curl -s "http://127.0.0.1:18080/twitch/CHANNEL" | head
```

API: `/api/status` · Events: `/api/events` · Metrics: `/metrics` · Edge: `/twitch/<channel>`

## OpenWrt (Cortex-A53 → ipk)

Каталог [`dist/openwrt-24.10-a53/`](dist/openwrt-24.10-a53/).

```bash
./scripts/build-a53.sh --copy-pkg
FORCE_BUILD=1 bash scripts/pack-ipk-a53.sh   # или SKIP_BUILD=1 если бинарь уже есть
```

Подробно: [Сборка OpenWrt / LuCI](docs/BUILD_OPENWRT.md).

## Документация

- [Оглавление docs](docs/INDEX.md) · [Архитектура](docs/ARCHITECTURE.md) · [Roadmap](docs/ROADMAP.md)
- [Proxy](docs/PROXY_ARCHITECTURE.md) · [Совместимость](docs/COEXISTENCE.md) · [ADR 0002 Edge](docs/adr/0002-playlist-edge.md)
- [Changelog](docs/CHANGELOG.md) · [Performance](docs/PERFORMANCE.md)

## Лицензия

MIT © 2026 Denis Ershov
