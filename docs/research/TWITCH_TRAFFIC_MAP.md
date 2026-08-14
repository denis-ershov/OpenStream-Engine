# Twitch Traffic Map

Живая карта: **что откуда и куда** в цепочке стрима.  
Заполняется вручную (ниже) или автолабом → `flow_map.json` ([autolab](../../research/twitch/autolab/)).

Статус ячеек: **топология подтверждена только E0** (сессия 20260813T143118Z_gaules).
Она не подтверждает отсутствие SSAI-рекламы. Итоги повторной проверки и ограничения
автолаба приведены в [TWITCH_AD_BLOCK_AUDIT.md](TWITCH_AD_BLOCK_AUDIT.md).

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
  → edge.ads.twitch.tv (Ad server / Banners → DNS Block 0.0.0.0)
```

## Таблица шагов

| # | Когда | DNS/SNI host | Method/path | Размер порядка | Geo? | Egress / Route | Примечание |
|---|--------|--------------|-------------|----------------|------|----------------|------------|
| 1 | pre-play | `www.twitch.tv`, `assets.twitch.tv` | `GET /<channel>` | ~MB (HTML/JS) | да | Direct (WAN) | Загрузка основного приложения |
| 2 | token | `gql.twitch.tv` | `POST /gql` | ~KB | да | **Direct (RU WAN)** | PlaybackAccessToken. На глобальных/монетизируемых каналах (`show_ads: true`) не защищает от SSAI |
| 3 | master | `usher.ttvnw.net` | `GET /api/v2/channel/hls/…m3u8` | ~KB | да | **SmartDNS / VPN (EU)** | Мастер-плейлист (разблокировка 1080p/1440p/Source). В ЕС вставляет SSAI |
| 4 | media | `*.playlist.ttvnw.net` | `GET /v1/playlist/…m3u8` | ~KB | да | Direct (WAN) | Медиа-плейлист качества со вшитыми тегами `twitch-stitched-ad` |
| 5 | segment | `*.live-video.net`, `*.cloudfront.hls.ttvnw.net` | `GET /v1/segment/…ts` | MB/s | да | Direct (WAN) | Видеосегменты стрима и рекламы (раздаются с одних CDN) |
| 6 | ads/tracking | `edge.ads.twitch.tv`, `countess.twitch.tv` | `*` | ~KB | нет | **DNS BLOCK** | Баннеры и ad-трекеры (Sinkhole `0.0.0.0`). Не влияет на серверные HLS-сегменты (SSAI) |

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
