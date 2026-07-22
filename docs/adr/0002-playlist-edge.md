# ADR 0002 — Playlist Edge без клиентского CA

- **Статус:** Accepted
- **Дата:** 2026-07-22
- **Обновлено:** 2026-07-22 (r14: rewrite / Host)

## Контекст

Transparent MITM (nft divert HLS → `:18080`) требует установки CA роутера на каждый клиент. На практике почти никто этого не делает; широкий divert `*.twitch.tv` ломал открытие сайта без CA.

TLS не позволяет читать/менять HTTPS на роутере без доверия клиента к MITM-CA (mitmproxy/SSLsplit).

## Решение

**Default = Playlist Edge (`mode: edge`):**

1. Клиент (VLC / companion / nested URL) запрашивает `GET http://router:18080/twitch/<channel>`.
2. Роутер резолвит GQL PlaybackAccessToken + usher, отдаёт **master** m3u8.
3. Варианты в master переписываются на `http://router:18080/https://cdn/…`  
   (`proxy_public_url` **или** `http://{Host}` запроса, если Host не loopback),  
   чтобы **media** m3u8 тоже шли через Edge (**strip только на media**).
4. Сегменты в media playlist остаются absolute CDN URL — медиа не через роутер.
5. Transparent MITM + CA — **opt-in advanced** (`mode: transparent`, `mitm=1`).

Hostlists (per-service + custom + optional GitHub) обслуживают companion redirect targets и legacy divert, не заменяют Edge.

## Последствия

- Стоковое Twitch-приложение без companion и без CA **не** получит strip (нет plaintext).
- Без rewrite master → nested strip на midroll не сработает (`ads_found=0` при растущих playlists).
- Запрос через `127.0.0.1` без заданного `proxy_public_url` не даёт LAN-совместимого rewrite — задайте Public Edge URL в LuCI.
- GQL Client-ID / schema могут меняться у Twitch — Edge API потребует сопровождения (см. Stage G seamless).
- rustls 0.23 требует явный CryptoProvider (`ring`) в процессе (иначе panic на GQL/usher).

## Ссылки

- [PROXY_ARCHITECTURE.md](../PROXY_ARCHITECTURE.md)
- [COEXISTENCE.md](../COEXISTENCE.md)
- [ROADMAP.md](../ROADMAP.md) Stage H
