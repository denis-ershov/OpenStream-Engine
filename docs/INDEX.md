# Документация OpenStream Engine

Актуальная база: **0.4.2**, IPK **14**.

**Цель №1** (только роутер, все клиенты, ноль действий на устройстве): **`[blocked]` TLS** — [ADR 0003](adr/0003-goal1-router-only-tls.md).  
Lab-код (Edge/MITM) **не** закрывает Goal №1.

## С чего начать

| Документ | Содержание |
|----------|------------|
| [adr/0003-goal1-router-only-tls.md](adr/0003-goal1-router-only-tls.md) | **Цель №1** + TLS-инвариант (читать первым) |
| [ROADMAP.md](ROADMAP.md) | Цель №1, lab Stage H, спринт |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Компоненты; Goal №1 vs lab Edge |
| [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md) | Lab Playlist Edge (не Goal №1) |
| [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md) | Режимы, API, rewrite, MITM lab |
| [COEXISTENCE.md](COEXISTENCE.md) | Соседи + lab Edge smoke |
| [CHANGELOG.md](CHANGELOG.md) | История IPK / docs |

## Архитектура по слоям

| Документ | Слой |
|----------|------|
| [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md) | m3u8, master vs media, strip |
| [DASH_ARCHITECTURE.md](DASH_ARCHITECTURE.md) | MPD |
| [PLUGIN_ARCHITECTURE.md](PLUGIN_ARCHITECTURE.md) | Trait Plugin |
| [SDK.md](SDK.md) | Авторам плагинов (ABI 3) |
| [adr/0001-plugin-abi.md](adr/0001-plugin-abi.md) | Статическая линковка |

## Сборка и поле

| Документ | Содержание |
|----------|------------|
| [PACKAGING.md](PACKAGING.md) | Профили Cargo, release IPK |
| [BUILD_OPENWRT.md](BUILD_OPENWRT.md) | Cross A53, `.ipk` |
| [PERFORMANCE.md](PERFORMANCE.md) | RSS, чеклист lab Edge |

## Lab Edge (не Цель №1) — 30 секунд

Требует **действия клиента** (открыть URL / companion) → не Goal №1.

1. `GET http://LAN_IP:18080/twitch/<channel>`
2. Master rewrite → nested media → strip; сегменты с CDN.
3. Без Public Edge URL / LAN Host rewrite strip media не сработает.
