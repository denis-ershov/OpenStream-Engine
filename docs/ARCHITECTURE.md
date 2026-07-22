# Архитектура OpenStream Engine

## Цель

Модульная система для OpenWrt: чистые HLS/DASH-манифесты на роутере. Логика сервисов — в плагинах; ядро универсально.

## Компоненты

```text
     Client (VLC / companion / browser)
              │  GET /twitch/<channel>   (Playlist Edge, без CA)
              ▼
         streamproxyd
              │  GQL + usher master
              │  rewrite → /https://cdn/…media.m3u8
              │  (strip только на media)
              ▼
         Client ← clean master
              │  GET nested media через :18080
              ▼
         streamproxyd → fetch CDN media → strip → client
              │
              └── segments: client → CDN напрямую

   Legacy (opt-in): nft divert + MITM + CA на клиентах
```

Default: **Playlist Edge** — CA не нужен. MITM — advanced.  
См. [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md), [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md), [COEXISTENCE.md](COEXISTENCE.md).

| Компонент | Crate | Зона ответственности |
|-----------|-------|----------------------|
| streamproxyd | `streamproxyd` | Демон, lifecycle, конфиг, `CryptoProvider` (rustls), reload |
| Proxy | `ose-proxy` | Edge API, nested fetch, optional MITM/CONNECT |
| HLS Manifest | `ose-manifest` | Парсинг/сериализация m3u8 |
| DASH Manifest | `ose-dash` | Парсинг/сериализация MPD |
| MediaFilter | `ose-media` | Общий контракт HLS/DASH |
| Segment Engine | `ose-segment` | Классификация URL; тело не трогаем |
| Cache Engine | `ose-cache` | TTL-кэш; ключ = URL + etag/hash (+ `proxy_base` при rewrite) |
| Ad Detector | `ose-detector` | Rules → Markers → Confidence (HLS) |
| Plugin API | `ose-plugin` | Trait `Plugin` |
| Rules | `ose-rules` | YAML rulesets |
| Twitch | `ose-plugin-twitch` | Segment Stripping + master rewrite |
| HLS generic | `ose-plugin-hls` | Kick/Trovo/custom |
| DASH | `ose-plugin-dash` | Strip ad Period/AdaptationSet |
| Neighbors | `ose-neighbors` | Детект zapret/podkop/… для `/api/status` |
| API | `ose-api` | `/api/status`, метрики |
| Config | `ose-config` | YAML / UCI |

## Поток данных (Edge)

1. Клиент → `GET http://LAN_IP:18080/twitch/<channel>` (не полагаться на `127.0.0.1` с другого устройства).
2. Proxy: GQL `PlaybackAccessToken` + usher **master** m3u8.
3. `proxy_base` = `proxy_public_url` **или** `http://{Host}` запроса (не loopback) → rewrite `#EXT-X-STREAM-INF` URI на nested `http://router:18080/https://…`.
4. Плеер запрашивает nested media → Proxy fetch + **strip** → ответ; сегменты в media остаются absolute CDN.
5. Egress к Twitch идёт через соседей (zapret/podkop/sing-box).

Без шага 3 плеер ходит за media на CDN напрямую → `ads_found=0` при растущем `playlists_total` (считаются masters).

## Ключевые решения

- **Rust** — предсказуемый RSS на OpenWrt.
- **Edge-first без CA** — MITM только opt-in ([ADR 0002](adr/0002-playlist-edge.md)).
- **Strip только на media playlist**; master — rewrite + учёт в метриках.
- **rustls 0.23**: явный `ring` CryptoProvider при старте (иначе panic на первом TLS).
- **Своя nft `inet openstream`** — только для legacy transparent.
- **Плагины compile-time** ([ADR 0001](adr/0001-plugin-abi.md)).
- **Strip без backup-токена** по умолчанию (seamless — Stage G).

## Версии

См. **[ROADMAP.md](ROADMAP.md)**, **[INDEX.md](INDEX.md)**. Stage H — Playlist Edge. IPK: **0.4.2-14**.
