# Proxy Architecture

## Место в стеке роутера

OpenStream — **Playlist Edge** на роутере (не замена zapret/podkop):

```text
клиент → GET :18080/twitch/<channel>
  → GQL + usher master
  → rewrite variants → http://router:18080/https://cdn/…media.m3u8
  → клиент тянет media через nested
  → strip на media; сегменты → CDN напрямую
```

CA **не** нужен. Legacy MITM: [COEXISTENCE.md](COEXISTENCE.md), [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md).

## Режимы (UCI / конфиг)

| Mode | Описание | По умолчанию |
|------|----------|--------------|
| `edge` | Playlist Edge API + nested `/https://…` | **Да** |
| `transparent` | nft divert + SNI MITM (нужен CA) | Legacy |
| `redirect_whitelist` | Алиас `transparent` | — |
| `explicit` | HTTP CONNECT proxy | Отладка |
| `off` | Только `/api/*` | Нет |

## Порты / API

| Endpoint | Назначение |
|----------|------------|
| Listen `0.0.0.0:18080` | HTTP Edge / API |
| `GET /twitch/<channel>` | Edge: GQL + usher master (+ rewrite) |
| `GET /https://host/path.m3u8` | Nested fetch + strip (media / variants) |
| `GET /api/status` | Плагины, соседи, счётчики |
| `GET /api/events` | Кольцевой буфер событий |
| `GET /metrics` | OpenMetrics |
| `POST /api/reload` | Hot-reload конфига/плагинов |

## Master rewrite (`proxy_public_url`)

Strip Twitch работает **только на media** playlist. Чтобы media шли через роутер:

1. LuCI → **Public Edge URL** = `http://192.168.8.1:18080` (ваш LAN), **или**
2. Запрос с Host = LAN IP:порт (Edge сам выставит `proxy_base`).

Результат в master:

```text
#EXT-X-STREAM-INF:...
http://192.168.8.1:18080/https://…ttvnw.net/….m3u8
```

Проверка:

```bash
wget -qO- "http://192.168.8.1:18080/twitch/CHANNEL" | head -40
# должны быть nested URL на роутер, не «голые» CDN variants
```

Если открывать Edge только как `http://127.0.0.1:18080/…`, Host = loopback → rewrite **не** ставится (warn в логе) — задайте `proxy_public_url`.

Кэш манифеста учитывает `proxy_base` в ключе (`url|pb:…`).

## Hostlists

Compose: `/usr/libexec/openstream-compose-hostlist` → `/var/run/openstream/hostlist-effective.txt`

- Shipped: `/usr/share/openstream/hostlists/{twitch,kick,trovo,youtube}.txt`
- Remote cache: `/etc/openstream/hostlists-remote/` (опционально, GitHub raw, 12ч)
- UCI: `list hostlist_services`, `list custom_domain`, `hostlist_remote*`

Для companion targets и **только** при `mode=transparent` для nft set.  
dnsmasq nftset: `ttvnw.net` / `jtvnw.net` / `live-video.net` — **не** `/twitch.tv/`.

## TLS (клиент Edge → Twitch)

- rustls 0.23 + **явный** `CryptoProvider` (`ring`) при старте `streamproxyd` и в `ose-proxy`.
- Без этого: panic на GQL/usher → при `panic=abort` падает весь демон (procd crash loop).
- Корни: `webpki-roots` (системный CA-bundle на OpenWrt желателен для прочих TLS).

## TLS MITM (legacy)

- `mode=transparent` + `mitm=1` + CA на клиентах.
- Whitelist SNI: CDN-like (`ttvnw.net`, weaver, …), не `www`/`gql.twitch.tv`.

## nftables

Только `inet openstream` при transparent. Edge таблицу не поднимает.

## Типичные симптомы

| Симптом | Причина |
|---------|---------|
| Panic `CryptoProvider` | Бинарь &lt; r13 |
| `playlists&gt;0`, `ads_found=0`, в master нет nested URL | Нет rewrite (`proxy_public_url` / Host) |
| `playlists&gt;0`, nested есть, ads=0 | Нет midroll в окне замера **или** детект не сработал |
| Exit 126 Permission denied | Нет `+x` на бинаре (pack с Windows; r12+ postinst/chmod) |
| CONNECTION_REFUSED + crash loop | Смотри panic / exit 126 выше |
