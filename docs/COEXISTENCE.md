# Сосуществование с соседними сервисами (Podkop, Forkop, Zapret, Zapret2, PassWall, OpenClash)

OpenStream Engine спроектирован по принципу **абсолютной изоляции и бесконфликтного сосуществования** с другими сетевыми пакетами OpenWrt.

---

## 1. Архитектурный принцип бесконфликтности

| Сервис | Что делает сервис | Как взаимодействует с OpenStream Engine | Статус совместимости |
|---|---|---|---|
| **Zapret / Zapret2 (`nfqws`)** | Обход DPI (TCP desync / fake SNI) на уровне сырых сокетов | OpenStream Engine направляет тяжелый видеопоток (`live-video.net`) напрямую через провайдера. Если провайдер замедляет или блокирует CDN, zapret прозрачно десинкает TCP-сессию. Конфликта нет, так как OpenStream не перехватывает порты. | 🟢 **100% совместимо** |
| **Podkop / Forkop / NetShift** | PBR (Policy Based Routing) через Sing-box / Xray / VPN | OpenStream автоматически обнаруживает nftset соседа (`vpn_domains`, `forkop_domains`, `netshift_domains`) и направляет в него **только** `usher.ttvnw.net` (плейлист качеств), не перегружая VPN видеопотоком. | 🟢 **100% совместимо** (см. рекомендацию ниже) |
| **PassWall / HomeProxy / OpenClash** | Маршрутизация на основе правил | Совместим через автоматическое сопоставление nftset/ipset. | 🟢 **100% совместимо** |
| **ByeDPI (`ciadpi`)** | Локальный SOCKS5 desync прокси | Не создает nftables правил, нет пересечений. | 🟢 **100% совместимо** |

---

## 2. Важная рекомендация по настройке списков Podkop / Forkop

### ⚠️ Проблема общего домена `twitch.tv`
Если в конфигурации Podkop / Forkop / NetShift домен `twitch.tv` добавлен **целиком**:
1. Запросы к `gql.twitch.tv` (токен) пойдут через зарубежный VPN, и Twitch выдаст токен с рекламой.
2. Видеосегменты CDN также могут пойти через VPN, расходуя трафик и снижая скорость.

### ✅ Решение
* **Удалите `twitch.tv` из общих списков Podkop / Forkop / PassWall.**
* Позвольте OpenStream Engine точечно управлять маршрутами:
  * `gql.twitch.tv` — прямой РФ IP (токен без рекламы).
  * `usher.ttvnw.net` — в VPN-сет Podkop (`4#inet#fw4#vpn_domains`).
  * `live-video.net` — прямой WAN (максимальная скорость видео).
  * `edge.ads.twitch.tv` — DNS Sinkhole (`0.0.0.0`).

---

## 3. Автоматическое обнаружение VPN-наборов (`detect_vpn_set`)

Служба OpenStream Engine автоматически сканирует таблицу `inet fw4` и находит активные сеты соседних пакетов:
* `4#inet#fw4#vpn_domains` (Podkop / PBR)
* `4#inet#fw4#forkop_domains` (Forkop)
* `4#inet#fw4#netshift_domains` (NetShift)
* `4#inet#fw4#passwall_vpn` (PassWall)
* `4#inet#fw4#unblock_domains`

Если имя сета не указано вручную в UCI, система автоматически подставит найденный сет соседа.

---

## 4. Изоляция файлов конфигурации

* Конфигурация генерируется исключительно в `/tmp/dnsmasq.d/openstream.conf` и `/etc/dnsmasq.d/openstream.conf`.
* Чужие файлы (`podkop.conf`, `zapret.conf` и т.д.) **никогда не перезаписываются**.
* При остановке или удалении OpenStream Engine удаляется только `openstream.conf`, после чего `dnsmasq reload` возвращает DNS в исходное состояние.
