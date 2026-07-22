# ADR 0002 — Playlist Edge без клиентского CA

- **Статус:** Accepted — **`[lab]`**, не закрывает Цель №1
- **Дата:** 2026-07-22
- **Обновлено:** 2026-07-22 (Goal №1 → [ADR 0003](0003-goal1-router-only-tls.md))

## Контекст

Transparent MITM требует CA на каждом клиенте. Широкий divert `*.twitch.tv` ломал сайт.

TLS не позволяет читать/менять HTTPS на роутере без trust на клиенте.

## Решение (lab)

**Lab default пакета = Playlist Edge (`mode: edge`):**

1. Клиент **сам** запрашивает `GET http://router:18080/twitch/<channel>` (VLC / companion / nested).
2. Роутер: GQL + usher **master**.
3. Rewrite вариантов на nested `http://router:18080/https://…` (`proxy_public_url` или Host).
4. Strip на **media**; сегменты absolute CDN.
5. MITM + CA — lab opt-in (`mode: transparent`).

Это **требует действия на клиенте** → **не** Цель №1 ([ADR 0003](0003-goal1-router-only-tls.md)).

## Последствия

- Не обещать покрытие стоковых приложений/ТВ без касания устройства.
- Без rewrite — `ads_found=0` при растущих playlists.
- GQL schema / Client-ID могут меняться.
- rustls CryptoProvider обязателен (r13+).

## Ссылки

- [0003-goal1-router-only-tls.md](0003-goal1-router-only-tls.md) — Цель №1
- [PROXY_ARCHITECTURE.md](../PROXY_ARCHITECTURE.md)
- [ROADMAP.md](../ROADMAP.md) Stage H `[lab]`
