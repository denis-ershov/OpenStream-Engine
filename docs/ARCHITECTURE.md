# Архитектура OpenStream Engine

## Цель №1 vs lab

**Цель №1** (строго): вся логика на роутере; все клиенты; **ноль** действий на устройстве.  
Статус: **`[blocked]` TLS** — [ADR 0003](adr/0003-goal1-router-only-tls.md).

Ядро (`ose-proxy`, `streamproxyd`, nft, ABI) **можно переписывать** ради Goal №1; смена ядра **не** обходит проверку сертификата на клиенте.

Ниже — **lab**-архитектура текущего пакета (Edge/MITM). Она **не** обещает Goal №1.

## Lab-компоненты

```text
     Client (VLC / companion)     ← действие на клиенте = не Goal №1
              │  GET /twitch/<channel>
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

   Lab MITM (opt-in): nft divert + CA на клиентах  ← тоже не Goal №1
```

См. [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md), [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md), [COEXISTENCE.md](COEXISTENCE.md).

| Компонент | Crate | Зона ответственности |
|-----------|-------|----------------------|
| streamproxyd | `streamproxyd` | Демон, lifecycle, конфиг, `CryptoProvider` (rustls), reload |
| Proxy | `ose-proxy` | Edge API, nested fetch, optional MITM/CONNECT |
| HLS Manifest | `ose-manifest` | Парсинг/сериализация m3u8 |
| DASH Manifest | `ose-dash` | Парсинг/сериализация MPD |
| MediaFilter | `ose-media` | Общий контракт HLS/DASH |
| Segment Engine | `ose-segment` | Классификация URL; тело не трогаем |
| Cache Engine | `ose-cache` | TTL-кэш; ключ = URL + etag/hash (+ `proxy_base`) |
| Ad Detector | `ose-detector` | Rules → Markers (HLS) |
| Plugin API | `ose-plugin` | Trait `Plugin` |
| Rules | `ose-rules` | YAML rulesets |
| Twitch | `ose-plugin-twitch` | Segment Stripping + master rewrite |
| HLS generic | `ose-plugin-hls` | Kick/Trovo/custom |
| DASH | `ose-plugin-dash` | Strip ad Period/AdaptationSet |
| Neighbors | `ose-neighbors` | Детект zapret/podkop/… |
| API | `ose-api` | `/api/status`, метрики |
| Config | `ose-config` | YAML / UCI |

## Поток данных (lab Edge)

1. Клиент **сам** → `GET http://LAN_IP:18080/twitch/<channel>`.
2. GQL + usher **master**; `proxy_base` → rewrite на nested.
3. Media через nested → strip; сегменты absolute CDN.
4. Egress через соседей (zapret/podkop/…).

Без шага 1 (клиентский URL) стоковое приложение Goal №1 не покрывается.

## Ключевые решения

- **Цель №1 + TLS blocked** — [ADR 0003](adr/0003-goal1-router-only-tls.md).
- **Rust** — предсказуемый RSS на OpenWrt.
- **Lab Edge** — [ADR 0002](adr/0002-playlist-edge.md); не Goal №1.
- **Strip только на media**; master — rewrite.
- **rustls 0.23**: явный `ring` CryptoProvider.
- **Плагины compile-time** — [ADR 0001](adr/0001-plugin-abi.md).

## Версии

[ROADMAP.md](ROADMAP.md), [INDEX.md](INDEX.md). IPK lab: **0.4.2-14**. Goal №1: blocked.
