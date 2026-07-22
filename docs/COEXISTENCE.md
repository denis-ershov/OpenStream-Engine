# Совместимость: Playlist Edge + соседи

OpenStream **не** обходит DPI. Default — **Playlist Edge** (без CA): роутер отдаёт чистый m3u8, сегменты с CDN.  
Соседи на **egress** Edge/GQL/usher. Legacy MITM — отдельно ниже.

Оглавление docs: [INDEX.md](INDEX.md).

## Целевая цепочка (Edge)

```text
Клиент (VLC / companion)
  │  GET http://LAN_IP:18080/twitch/<channel>
  ▼
streamproxyd  (GQL + usher master + rewrite)
  │  nested media → strip
  │  egress
  ▼
zapret / podkop* / ByeDPI / sing-box → Twitch
  │
клиент ← clean master / clean media
клиент → CDN сегменты напрямую
```

**CA не требуется.**

Для strip обязателен rewrite master → nested:

- LuCI **Public Edge URL** = `http://LAN_IP:18080`, или
- запрос с Host = LAN (не `127.0.0.1` с другого хоста).

## UX: как клиент открывает стрим

| Канал | Нужен CA | Когда |
|-------|----------|--------|
| VLC / mpv URL `http://LAN:18080/twitch/channel` | Нет | Сейчас (H1) |
| Companion (браузер) — redirect playlist | Нет | Stage H7 |
| Transparent MITM | Да | Legacy |

Стоковое Twitch-приложение без companion и без CA **не** получит strip.

## Legacy: transparent MITM

```text
Клиент → CDN:443 ∈ @openstream_hls → nft → :18080 → SNI MITM → strip → egress
```

Нужен `/etc/openstream/ca.crt` на **каждом** клиенте. Без CA сайт/HLS ломаются.  
Divert только HLS CDN (не `www`/`gql` / не весь `*.twitch.tv`).

| Mode | Назначение |
|------|------------|
| `edge` (default) | Playlist Edge, без CA |
| `transparent` | nft + MITM (CA) |
| `redirect_whitelist` | алиас transparent |
| `explicit` | HTTP CONNECT |
| `off` | API only |

## Hostlists / DoH

См. compose + remote в [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md).

| Где DoH | Влияние |
|---------|---------|
| Клиент → публичный DoH | dnsmasq nftset не кормится; **Edge не зависит** от этого |
| Роутер через dnsmasq | nftset для legacy divert работает |
| Роутер DoH в обход dnsmasq | как клиентский DoH для nftset |

## Соседи

| Стек | Поведение |
|------|-----------|
| zapret / ByeDPI | Happy-path egress Edge |
| podkop / netshift / forkop | Edge egress может идти в TUN |
| SSClash / OpenClash / Mihomo | То же |
| PassWall / HomeProxy / sing-box / xray | Детект в `/api/status` |
| tpws/redsocks + transparent MITM на те же dst | Конфликт только в MITM-режиме |

`coexistence_ok` / `mode_hint` в `/api/status` — эвристика, не гарантия маршрута.

## Проверка Edge (поле)

```bash
# статус / соседи
wget -qO- http://127.0.0.1:18080/api/status

# master с LAN IP роутера (с ПК или с роутера подставьте свой LAN):
wget -qO- "http://192.168.8.1:18080/twitch/CHANNEL" | head -40
# ожидаются строки http://192.168.8.1:18080/https://…

# метрики после проигрывания в VLC
wget -qO- http://127.0.0.1:18080/metrics | grep -E 'playlists|ads_found|segments_removed'
```

| Метрика | Ожидание |
|---------|----------|
| `playlists_total` | растёт (master + media) |
| `ads_found_total` | ≥1 при midroll |
| Nested URL в master | да |

См. [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md), [PERFORMANCE.md](PERFORMANCE.md), [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md).
