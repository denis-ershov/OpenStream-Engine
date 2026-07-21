# Архитектура OpenStream Engine

## Цель

Модульная система для OpenWrt, прозрачно обрабатывающая HLS- и DASH-манифесты. Логика сервисов инкапсулирована в плагинах; ядро универсально.

## Компоненты

```text
                Internet
                    │
             nftables (opt-in, inet openstream)
                    │
                    ▼
              streamproxyd
      ┌─────────────┼─────────────┐
      │             │             │
 Manifest      Segment       Cache
 (HLS/DASH)    Engine       Engine
      │             │             │
      └─────────────┼─────────────┘
                    │
              Plugin Manager
                    │
        ┌───────────┼───────────┐
        │           │           │
   plugin-twitch  plugin-hls  plugin-dash
                  (Kick/Trovo)  (MPD)
```

| Компонент | Crate | Зона ответственности |
|-----------|-------|----------------------|
| streamproxyd | `streamproxyd` | Демон, lifecycle, конфиг, reload плагинов |
| Proxy | `ose-proxy` | Explicit HTTP(S) proxy, MITM whitelist, `/api/reload` |
| HLS Manifest | `ose-manifest` | Парсинг/сериализация m3u8 |
| DASH Manifest | `ose-dash` | Парсинг/сериализация MPD, фильтр Period/AdaptationSet |
| MediaFilter | `ose-media` | Общий контракт HLS/DASH |
| Segment Engine | `ose-segment` | Классификация URL (HLS/CMAF); тело не трогаем |
| Cache Engine | `ose-cache` | TTL-кэш; `CacheKey` = URL + etag/hash |
| Ad Detector | `ose-detector` | Rules → Markers → Confidence (HLS) |
| Plugin API | `ose-plugin` | Trait `Plugin` (HLS + `process_mpd`) |
| Rules | `ose-rules` | YAML rulesets, host match, Kick/Trovo presets |
| Twitch | `ose-plugin-twitch` | Segment Stripping + master rewrite |
| HLS generic | `ose-plugin-hls` | RulesHlsPlugin (Kick/Trovo/custom) |
| DASH | `ose-plugin-dash` | Strip ad Period/AdaptationSet |
| API | `ose-api` | `/api/status`, метрики |
| Config | `ose-config` | YAML / UCI-совместимые настройки |

## Поток данных

1. Клиент запрашивает `.m3u8` / `.mpd` через explicit proxy (по умолчанию).
2. Proxy загружает манифест с CDN (egress может идти через zapret/podkop).
3. Plugin Manager выбирает плагин по `match_request` + `ManifestKind`.
4. Плагин: parse → detect/filter → serialize.
5. Клиенту отдаётся очищенный манифест; сегменты `.ts`/`.m4s` остаются на CDN (туннель/streaming без буферизации).

## Ключевые решения

- **Язык: Rust** — предсказуемый RSS на OpenWrt рядом с zapret/podkop.
- **Explicit proxy по умолчанию** — нулевой конфликт с DPI/PBR-стеком.
- **Своя nft-таблица `inet openstream`** — чужие таблицы не трогаем.
- **Плагины compile-time** — без динамической загрузки в v1–v2.
- **Strip без backup-токена** — клиент ждёт следующий live-сегмент (freeze ≈ длина midroll).

## Версии

Актуальная сверка с исходным планом, rust-pro ревью и детальный roadmap: **[ROADMAP.md](ROADMAP.md)**.

- **v1.0 / 0.1.x** — каркас Twitch Segment Stripping + proxy + LuCI.
- **v1.0-hardened** — passthrough сегментов, persistent CA, проводка UCI/режимов.
- **v1.1 / 0.2.0** — универсальные правила, Kick/Trovo/generic HLS.
- **v2.0 / 0.3.0** — DASH/MPD + MediaFilter + единый cache keyspace.
- **v3.0 / 0.4.0** — SDK (static ABI), coalescing, OpenMetrics/events, YouTube scaffold.
- **Далее** — полевая калибровка маркеров, feature-срез пакетов, seamless backup (opt-in).

См. также [DASH_ARCHITECTURE.md](DASH_ARCHITECTURE.md), [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md), [SDK.md](SDK.md), [PACKAGING.md](PACKAGING.md).
