# OpenStream Engine

Модульная платформа обработки HLS/DASH для OpenWrt. Плагины: Twitch, Kick/Trovo/YouTube (rules), DASH. SDK — статическая линковка (ADR 0001).

**Цель №1:** только роутер · все клиенты · ноль действий на устройстве → **`[blocked]` TLS** ([ADR 0003](docs/adr/0003-goal1-router-only-tls.md)).  
Ядро можно менять ради цели — TLS на клиенте это не обходит.

**Lab (не Goal №1):** Playlist Edge · IPK 0.4.2-14 · zapret / podkop / …

## Lab-возможности (требуют действия клиента)

- Edge: `GET /twitch/<channel>` — VLC/companion URL (не стоковое приложение «как есть»).
- Optional MITM + CA.
- HLS/DASH strip; hostlists; `/metrics`, `/api/events`; LuCI.

## Ограничения

- **Цель №1 не достигнута** — честного пути без клиента нет ([ADR 0003](docs/adr/0003-goal1-router-only-tls.md)).
- Lab: при рекламе freeze ≈ midroll; seamless — Stage G.
- Не обход DPI. [COEXISTENCE.md](docs/COEXISTENCE.md).

## Быстрый старт (lab / разработка)

```bash
cargo build --release -p streamproxyd
./target/release/streamproxyd --config config.example.yaml
curl -s "http://127.0.0.1:18080/twitch/CHANNEL" | head
```

## OpenWrt

[`dist/openwrt-24.10-a53/`](dist/openwrt-24.10-a53/) · [BUILD_OPENWRT.md](docs/BUILD_OPENWRT.md)

## Документация

- [Оглавление](docs/INDEX.md) · [Цель №1 / ADR 0003](docs/adr/0003-goal1-router-only-tls.md) · [Roadmap](docs/ROADMAP.md)
- [Архитектура](docs/ARCHITECTURE.md) · [Proxy](docs/PROXY_ARCHITECTURE.md) · [Changelog](docs/CHANGELOG.md)

## Лицензия

MIT © 2026 Denis Ershov
