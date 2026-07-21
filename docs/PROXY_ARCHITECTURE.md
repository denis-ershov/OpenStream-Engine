# Proxy Architecture

## Режимы (UCI / конфиг)

| Mode | Описание | По умолчанию |
|------|----------|--------------|
| `explicit` | Клиент указывает HTTP/HTTPS proxy или PAC | Да |
| `redirect_whitelist` | Opt-in nft redirect только к set `openstream_hls` | Нет |
| `off` | Только API/статистика | Нет |

## Порты

- Proxy: `0.0.0.0:18080` (не 80/443/53).
- API status: loopback (тот же процесс, путь `/api/status`).

## Поведение

1. **HTTP absolute-URI / CONNECT** — forward proxy.
2. Host из plugin whitelist + путь манифеста (`.m3u8` / `.mpd`) → MITM (при CA) → Manifest Engine → Plugin (тело ≤ `max_manifest_bytes`).
3. На whitelist-хостах пути **не** манифест (`.ts`/`.m4s`/…) → после TLS запрос проксируется **streaming** без полной буферизации тела.
4. Остальной CONNECT (не whitelist) — TCP-туннель без разбора.
5. `mode=off` — только `/api/status` (+ `/api/reload` недоступен для proxy-трафика, но reload остаётся).
6. `mode=redirect_whitelist` — fail-soft `nft -f` своего файла (Linux).
7. Master rewrite: при `proxy_public_url` варианты `#EXT-X-STREAM-INF` → `http://proxy/https://origin/...`; nested absolute разбирается `split_nested_absolute`.
8. `POST /api/reload` — перечитать YAML и пересобрать плагины (builder из `streamproxyd`).

## TLS MITM

- Persistent CA: пути `tls.ca_cert` / `tls.ca_key` (создаются при первом старте, если заданы).
- Кэш leaf-сертификатов по SNI/host.
- Whitelist MITM: Twitch CDN + Kick + Trovo (см. `host_in_whitelist`).
- Без доверенного CA strip HTTPS-манифестов невозможен.

## nftables

Только таблица `inet openstream`. См. [COEXISTENCE.md](COEXISTENCE.md).

## Метрики

`playlists`, `ads_found`, `segments_removed`, `active_streams`, `neighbors`.
