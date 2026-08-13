# Twitch Traffic Map

Живая карта: **что откуда и куда** в цепочке стрима.  
Заполняется вручную (ниже) или автолабом → `flow_map.json` ([autolab](../../research/twitch/autolab/)).

Статус ячеек: **заполнено по результатам успешного E0** (сессия 20260813T143118Z_gaules).

## Эталонная цепочка

```text
Browser / App
  → DNS (имена?)
  → TLS SNI (host?)
  → gql.twitch.tv
  → usher.ttvnw.net
  → *.playlist.ttvnw.net / live-video.net (?)
  → video-weaver*.ttvnw.net / CDN
  → segments (.ts / .m4s)
```

## Таблица шагов

| # | Когда | DNS/SNI host | Method/path | Размер порядка | Geo? | VPS? | Примечание |
|---|--------|--------------|-------------|----------------|------|------|------------|
| 1 | pre-play | `www.twitch.tv`, `assets.twitch.tv` | `GET /<channel>` | ~MB (HTML/JS) | да | нет | Загрузка основного приложения |
| 2 | token | `gql.twitch.tv` | `POST /gql` | ~KB | да | hypot. yes | PlaybackAccessToken |
| 3 | master | `usher.ttvnw.net` | `GET /api/v2/channel/hls/…m3u8` | ~KB | да | hypot. yes | Мастер-плейлист с вариантами качеств |
| 4 | media | `*.playlist.ttvnw.net` | `GET /v1/playlist/…m3u8` | ~KB | да | hypot. yes | Медиа-плейлист конкретного качества (SSAI маркеры) |
| 5 | segment | `*.live-video.net`, `*.cloudfront.hls.ttvnw.net` | `GET /v1/segment/…ts` | MB/s | да | no (goal) | Видеосегменты (.ts / .m4s) |

Порядок во времени: timeline из HAR / `network.json`. Учитывать poll media и prefetch.

## Инструкции захвата

### A. Браузер (ручной fallback)

1. Чистый профиль; по желанию отключить QUIC.
2. DevTools → Network: `gql`, `usher`, `m3u8`, `ttvnw`, `weaver`.
3. Preserve log → открыть канал → export HAR → `research/twitch/results/har/`.
4. Выписать host, path, order, status, size.

### B. Роутер без MITM

1. dnsmasq log DNS во время старта стрима.
2. Опционально: `tshark` на ClientHello SNI (без decrypt).
3. Список уникальных SNI: старт → 2 мин → midroll.

### C. Вне браузера

1. Autolab / tokendump + usherdump.
2. Media URL из master; маркеры ads.
3. Hosts в URI сегментов.

### D. Артефакты

- Эта таблица (обновить после сессии).
- `research/twitch/results/<session>/SESSION.md` + `flow_map.json`.
- Неизвестный host в HAR → строка в таблице **до** фиксации VPS-доменов в ADR 0004.

## Автозаполнение

```bash
python research/twitch/autolab/run_lab.py --channel CHANNEL --browser-only
# → results/<id>/flow_map.json, map_fragment.md
```

См. [OPENTWITCH_LAB.md](OPENTWITCH_LAB.md), [ADR 0004](../adr/0004-geo-split-egress.md).
