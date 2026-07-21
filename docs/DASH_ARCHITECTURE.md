# DASH Architecture

## Manifest Engine (`ose-dash`)

Лёгкий XML-разбор MPD (`quick-xml`) в owned-дерево с round-trip serialize.

Модель:

- `Mpd` → корневой `MPD`
- `Period` (дети MPD)
- `AdaptationSet` / `Representation` / дескрипторы (`AssetIdentifier`, `Role`, `SupplementalProperty`, …)

Неизвестные элементы сохраняются в дереве и сериализуются обратно.

## Фильтрация рекламы

`DashFilterRules` + `filter_ad_nodes`:

| Цель | Эвристики |
|------|-----------|
| Period | id/markers (`ad`, `midroll`, …); AssetIdentifier / Essential/SupplementalProperty со scheme markers (`urn:scte:dash:ad`, …) |
| AdaptationSet | `Role` value advertisement/ad; scheme markers в атрибутах |

CMAF-сегменты (`.m4s` / init `.mp4`) **не** буферизуются — только passthrough (`ose-segment`).

## Plugin (`ose-plugin-dash`)

- Match: `ManifestKind::Dash` (`.mpd`)
- Стадии: `filter_dash` / `process_mpd`
- Включение: `dash.enabled` (по умолчанию `true`)

## MediaFilter (`ose-media`)

Общий trait `apply_hls` / `apply_dash` + `ManifestKind` / `FilterOutcome` — единый контракт для HLS Entry и DASH Node.

## Proxy

Пути `.mpd` обрабатываются как манифесты (cap `max_manifest_bytes`), content-type `application/dash+xml`. Cache key: URL + ETag или FNV body hash.

См. [PLUGIN_ARCHITECTURE.md](PLUGIN_ARCHITECTURE.md), [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md).
