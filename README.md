# OpenStream Engine

Модульная платформа обработки HLS/DASH для OpenWrt. Плагины: Twitch, Kick/Trovo/YouTube (rules), DASH. SDK — статическая линковка (ADR 0001).

**Язык:** Rust · **Режим по умолчанию:** explicit HTTP(S) proxy · **Совместимость:** zapret / zapret2 / podkop

## Возможности v3.0 (0.4.0)

- HLS/DASH strip; master rewrite; prefetch; reload; singleflight coalesce.
- Observability: `GET /metrics`, `GET /api/events` (ring-buffer).
- Explicit proxy `:18080`; OpenWrt package + size-oriented release profile.
- SDK docs + plugin skeleton.

## Ограничения

- При рекламе плеер **ждёт** следующий live-сегмент (freeze ≈ длина midroll). Seamless backup-токены — не v1.
- Для HTTPS MITM нужен доверенный CA на клиентах.
- Не является обходом DPI — для доступа к Twitch используйте zapret/podkop.

## Быстрый старт (разработка)

```bash
cargo build --release -p streamproxyd
./target/release/streamproxyd --config config.example.yaml
```

Прокси: `http://127.0.0.1:18080` · API: `/api/status` · Events: `/api/events` · Metrics: `/metrics`

## OpenWrt (Cortex-A53 → ipk / apk)

```bash
./scripts/build-a53.sh --copy-pkg          # или --slim
# затем в OpenWrt SDK: make package/openstream-engine/compile
#                       make package/luci-app-openstream/compile
```

Подробно: [Сборка OpenWrt / LuCI](docs/BUILD_OPENWRT.md) (`.ipk` для opkg, `.apk` для OpenWrt 25.12+).

## Документация

- [Архитектура](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Сборка OpenWrt / LuCI (ipk, apk, A53)](docs/BUILD_OPENWRT.md)
- [Упаковка (кратко)](docs/PACKAGING.md)
- [SDK плагинов](docs/SDK.md)
- [Proxy](docs/PROXY_ARCHITECTURE.md)
- [Плагины](docs/PLUGIN_ARCHITECTURE.md)
- [HLS](docs/HLS_ARCHITECTURE.md)
- [DASH](docs/DASH_ARCHITECTURE.md)
- [Производительность](docs/PERFORMANCE.md)
- [Совместимость](docs/COEXISTENCE.md)
- [Changelog](docs/CHANGELOG.md)

## Лицензия

MIT © 2026 Denis Ershov
