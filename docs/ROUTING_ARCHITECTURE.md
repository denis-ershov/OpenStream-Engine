# Архитектура модульной маршрутизации OpenStream Engine

Документ фиксирует архитектуру раздельной маршрутизации (Split Routing), актуальную матрицу пресетов для Twitch и принципы модульности для других платформ.

---

## 1. Концепция Split Routing (Без MITM / Без CA)

Традиционные подходы (Edge, локальный MITM-прокси) требуют подмены TLS-сертификатов на клиентах или изменения URL в плеерах.
В OpenStream Engine реализован подход **чистой маршрутизации на уровне сети**:
1. **Токен авторизации (GQL / Token):** Запрашивается через Ad-Free гео-туннель (Албания, Украина, Казахстан и др.), где видеореклама (SSAI) отключена на стороне платформы.
2. **Мастер-плейлист качеств (Usher / Master):** Запрашивается через VPN-интерфейс / SmartDNS Tier-1 региона (Европа/США), разблокируя разрешения 1080p, 1440p и Source.
3. **Медиа-плейлисты и видеосегменты (CDN / Video Streams):** Скачиваются клиентом напрямую через провайдера на полной физической скорости без нагрузки на VPN.
4. **Баннеры и ad-трекеры (Ads / Trackers):** Блокируются локально через DNS Sinkhole (`0.0.0.0`).

```text
               ┌───────────────── OpenStream Engine ──────────────────┐
               │                                                      │
Browser / TV ──┼──► DNS gql.twitch.tv ────────► Ad-Free VPN (UA/AL) ─► Twitch (No Ads Token)
               │                                                      │
               ├──► DNS usher.ttvnw.net ──────► EU VPN / SmartDNS ───► Twitch (1440p Master)
               │                                                      │
               ├──► DNS edge.ads.twitch.tv ───► 0.0.0.0 (Sinkhole)    (Ad Trackers Blocked)
               │                                                      │
               └──► DNS live-video.net ───────► Direct WAN (ISP) ───► CDN (Full Speed Stream)
```

---

## 2. Поддерживаемые сценарии (Пресеты)

1. **🛡️ `clean_proxy_geosplit` (Geo-Split через Clean-Proxy / Ad-Free VPN):**
   - Token: `Ad-Free VPN (UA/AL/KZ)`
   - Master: `SmartDNS / EU VPN`
   - Segments: `Direct WAN (ISP)`
   - Ads: `Block (0.0.0.0)`
2. **⚡ `manifest_strip_edge` (Playlist Edge — локальный стриппинг манифестов):**
   - Точка входа: `http://router:18080/twitch/<channel>`
   - Роутер вырезает теги `#EXT-X-DATERANGE:CLASS="twitch-stitched-ad"` и рекламные сегменты.
   - Для плееров: VLC, Kodi, SmartTube, TiviMate, MPV, streamlink.
3. **🌍 `smartdns_quality_unlock` (Разблокировка 1080p60/1440p/Source):**
   - Token: `Direct WAN`
   - Master: `SmartDNS / EU VPN`
   - Segments: `Direct WAN (ISP)`
   - Ads: `Block (0.0.0.0)`
4. **⚙️ `custom` (Пользовательская матрица):**
   - Свободный выбор шлюза для каждого функционального узла.

---

## 3. Гарантии стабильности и защита от сбоев

1. **Валидация интерфейса:** суффикс `@interface` добавляется в `dnsmasq` только при физическом наличии интерфейса в `/sys/class/net/`.
2. **Предварительный тест конфигурации:** перед перезапуском `dnsmasq` всегда выполняется `dnsmasq --test`. При любой ошибке файл `openstream.conf` автоматически откатывается, гарантируя бесперебойную работу интернета и DNS на роутере.
3. **Точечная модификация `/etc/hosts`:** системный файл `/etc/hosts` никогда не перезаписывается целиком; записи добавляются и удаляются исключительно по маркеру `# openstream`.
