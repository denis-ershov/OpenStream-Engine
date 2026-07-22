# HLS Architecture

Оглавление: [INDEX.md](INDEX.md). Связь с Edge: [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md).

## Manifest Engine (`ose-manifest`)

Поддерживаемые теги:

- `#EXTM3U`, `#EXT-X-VERSION`
- `#EXTINF`
- `#EXT-X-MEDIA-SEQUENCE`
- `#EXT-X-TARGETDURATION`
- `#EXT-X-DISCONTINUITY`
- `#EXT-X-PROGRAM-DATE-TIME`
- `#EXT-X-DATERANGE`
- `#EXT-X-PREFETCH` / `#EXT-X-TWITCH-PREFETCH`
- `#EXT-X-ENDLIST`
- `#EXT-X-STREAM-INF` (master)
- прочие строки сохраняются как opaque tags

## Master vs Media

| Kind | Strip рекламы | Rewrite |
|------|---------------|---------|
| **Master** | Нет | Варианты → `proxy_base/https://…` при заданном `proxy_base` |
| **Media** | Да (`strip_ad_segments`) | Обычно нет; URI сегментов absolute CDN |

### Playlist Edge

1. `GET /twitch/<channel>` → usher **master**.
2. Rewrite вариантов на nested Edge URL.
3. Плеер периодически запрашивает **media** через nested → здесь растут `ads_found` / `segments_removed`.

Без шага 2 метрики: `playlists_total`↑, `ads_found=0`.

## Segment Engine

Классифицирует URI (`.ts`, `.m4s`, `.mp4`, `.aac`) по контексту плейлиста. Содержимое сегментов не изменяется.

## Cache Engine

Кэширует playlist / last media sequence / stripped body.  
TTL: 1–5 с. Ключ: URL + etag или body hash; при rewrite — суффикс `|pb:{proxy_base}`.

## Segment Stripping (Twitch)

Исходный **media** плейлист с midroll → удаляются пары `#EXTINF` + URI рекламных сегментов и связанные DATERANGE/prefetch.

Клиент не получает ссылки на рекламу и ждёт следующий live-сегмент (ожидаемый freeze). Seamless backup — Stage G (opt-in).

### Согласование MEDIA-SEQUENCE

При удалении сегментов sequence не «дырявится»: `#EXT-X-MEDIA-SEQUENCE` соответствует первому **оставшемуся** сегменту относительно исходной нумерации upstream.
