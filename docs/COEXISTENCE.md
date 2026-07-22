# Совместимость: lab Edge + соседи

OpenStream **не** обходит DPI.

**Цель №1** (все клиенты, ноль действий на устройстве): **`[blocked]` TLS** — [ADR 0003](adr/0003-goal1-router-only-tls.md).  
Этот документ описывает **lab** Edge/MITM и соседей — **не** закрытие Goal №1.

Оглавление: [INDEX.md](INDEX.md).

## Lab-цепочка (Edge)

```text
Клиент (VLC / companion)          ← действие на клиенте
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

Для lab strip нужен rewrite master → nested (Public Edge URL / LAN Host).

## UX lab (не Goal №1)

| Канал | Действие на клиенте | Goal №1? |
|-------|---------------------|----------|
| VLC / mpv URL | Открыть URL роутера | Нет |
| Companion | Расширение | Нет |
| Transparent MITM | CA в trust store | Нет |
| Стоковое приложение, ноль действий | — | **Goal №1 blocked** |

## Legacy: transparent MITM

```text
Клиент → CDN:443 ∈ @openstream_hls → nft → :18080 → SNI MITM → strip → egress
```

Нужен `/etc/openstream/ca.crt` на **каждом** клиенте. Без CA сайт/HLS ломаются.  
Divert только HLS CDN (не `www`/`gql` / не весь `*.twitch.tv`).

| Mode | Назначение |
|------|------------|
| `edge` (lab default пакета) | Playlist Edge; **не** Goal №1 |
| `transparent` | nft + MITM (CA); **не** Goal №1 |
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

См. [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md), [PERFORMANCE.md](PERFORMANCE.md), [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md), [adr/0003-goal1-router-only-tls.md](adr/0003-goal1-router-only-tls.md).
