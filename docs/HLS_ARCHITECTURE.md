# HLS Architecture

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

- **Master** — варианты качества; strip рекламы не применяется; при необходимости rewrite URL вариантов на proxy.
- **Media** — Segment Stripping.

## Segment Engine

Классифицирует URI (`.ts`, `.m4s`, `.mp4`, `.aac`) по контексту плейлиста. Содержимое сегментов не изменяется.

## Cache Engine

Кэширует:

- playlist / last playlist
- last media sequence
- last stripped playlist

TTL: 1–5 секунд (конфиг).

## Segment Stripping (Twitch)

Исходный плейлист с midroll → удаляются пары `#EXTINF` + URI рекламных сегментов и связанные DATERANGE/prefetch.

Клиент не получает ссылки на рекламу и ждёт следующий live-сегмент (ожидаемый freeze).

### Согласование MEDIA-SEQUENCE

При удалении сегментов sequence не «дырявится»: значение `#EXT-X-MEDIA-SEQUENCE` соответствует первому **оставшемуся** сегменту относительно исходной нумерации upstream (сохраняем sequence первого live-сегмента в окне, как в upstream для этого URI).
