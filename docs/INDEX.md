# Документация OpenStream Engine

Актуальная база: **0.4.2**, IPK release **14**. Default UX: **Playlist Edge** (без клиентского CA).

## С чего начать

| Документ | Содержание |
|----------|------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Компоненты, поток Edge, ключевые решения |
| [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md) | ADR: почему Edge, а не MITM по умолчанию |
| [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md) | Режимы, API, hostlists, rewrite, MITM legacy |
| [COEXISTENCE.md](COEXISTENCE.md) | zapret / podkop / sing-box + проверка Edge |
| [ROADMAP.md](ROADMAP.md) | Stage H и ближайший спринт |
| [CHANGELOG.md](CHANGELOG.md) | История изменений по релизам IPK |

## Архитектура по слоям

| Документ | Слой |
|----------|------|
| [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md) | m3u8, master vs media, strip |
| [DASH_ARCHITECTURE.md](DASH_ARCHITECTURE.md) | MPD, Period/AdaptationSet |
| [PLUGIN_ARCHITECTURE.md](PLUGIN_ARCHITECTURE.md) | Trait Plugin, Twitch/HLS/DASH |
| [SDK.md](SDK.md) | Авторам плагинов (ABI 3) |
| [adr/0001-plugin-abi.md](adr/0001-plugin-abi.md) | Статическая линковка |

## Сборка и поле

| Документ | Содержание |
|----------|------------|
| [PACKAGING.md](PACKAGING.md) | Профили Cargo, feature-срезы, release IPK |
| [BUILD_OPENWRT.md](BUILD_OPENWRT.md) | Cross A53, `.ipk` / `.apk`, установка |
| [PERFORMANCE.md](PERFORMANCE.md) | Benches, RSS, полевой чеклист Edge |

## Edge за 30 секунд

1. Клиент: `GET http://LAN_IP:18080/twitch/<channel>` (VLC / companion).
2. Роутер: GQL token → usher **master** → rewrite вариантов на nested `/https://…`.
3. Плеер тянет **media** через роутер → strip рекламы; сегменты — с CDN напрямую.
4. `proxy_public_url` в LuCI (или Host LAN в запросе) обязателен для rewrite; иначе strip не сработает.
