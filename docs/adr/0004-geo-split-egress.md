# ADR 0004 — Geo-split egress (кандидат Goal №1 без MITM)

- **Статус:** Accepted — verified (R3 Smart Geo-Split)
- **Дата:** 2026-07-22
- **Обновлено:** 2026-08-13 (утвержден оптимальный Smart Split)
- **Связь:** кандидат к [ADR 0003](0003-goal1-router-only-tls.md); не заменяет TLS-инвариант для content inspect

## Контекст

Цель №1: OpenWrt-пакет, все клиенты (ТВ / ПК / телефон / приставки; app / browser), **ноль** действий на клиенте.  
MITM / Edge / companion **отвергнуты** как продукт Goal №1 (требуют CA или URL/расширение).

Content inspection HTTPS на роутере без trust на клиенте невозможна ([ADR 0003](0003-goal1-router-only-tls.md)).

## Гипотеза & Оптимальный маршрут (Smart Geo-Split)

Не читать TLS. Менять только **маршрут** TCP/DNS для отдельных хостов. 
Вместо полной проксификации API (`gql` + `usher`) оптимальным решением является **сплит по доменам (R3)**:

| Трафик | Маршрут | Почему? |
|--------|---------|---------|
| `gql.twitch.tv` | **Direct РФ ISP** | Выпускается токен для RU региона (где по умолчанию **нет рекламы**). |
| `usher.ttvnw.net` | **Европейский VPN/SmartDNS** | Обход ограничений на 1080p/1440p/Source качества. |
| `*.playlist.ttvnw.net` / CDN | **Direct РФ ISP** | Медиа-плейлисты качеств и сами видеосегменты идут напрямую (нет рекламы и не тратится трафик VPS). |

Клиент по-прежнему устанавливает TLS к настоящему Twitch (валидный cert). Роутер: DNS/nftset или SNI-route → WireGuard/SOCKS **на роутере** (не VPN-профиль на ТВ/телефоне).

Объём через VPS: порядка десятков КБ (только master playlist), не видеопоток.

```text
Client ──► OpenWrt ──DNS/SNI usher.ttvnw.net──► VPS (EU) ──► Twitch (Quality master.m3u8)
              │
              ├──DNS/SNI gql.twitch.tv────────► РФ ISP   ──► Twitch (Token show_ads:false)
              │
              └──DNS/SNI playlist/segments────► РФ ISP   ──► CDN (Direct streams)
```

Финальный список хостов подтвержден сессией Autolab E0.

## Gate до продукта OpenWrt

| ID | Вопрос | Pass |
|----|--------|------|
| E0 | Карта хостов/порядка с реальной сессии | [OPENTWITCH_LAB](../research/OPENTWITCH_LAB.md) (pass) |
| E1 | Token+master через VPS; media/segments с ISP | pass |
| E2 | Минимальный host set через VPS | pass (R3: только `usher.ttvnw.net`) |
| E3 | Нет ads-маркеров / midroll при split | pass |
| E4 | Source/1440p в master | pass |

Инструмент: [`research/twitch/autolab/`](../../research/twitch/autolab/).

## Решение

1. MITM / CA / companion / Edge — **не** продукт Goal №1.
2. Кандидат Goal №1 = **geo-split egress** (этот ADR) переведен в статус **verified** (оптимальный сплит R3).
3. Для реализации на OpenWrt рекомендуется:
   - Добавить `usher.ttvnw.net` во входящий интерфейс VPN (через `dnsmasq` + `nftset`).
   - Исключить `gql.twitch.tv` из маршрутизации VPN (пускать напрямую через локальный WAN).
   - Исключить домены HLS CDN (`*.playlist.ttvnw.net`, `*.live-video.net`, `*.cloudfront.net`) из VPN.

## Риски

- Изменение политики авторизации токенов (Twitch может начать проверять IP запроса к медиа-сегментам и блокировать его при несовпадении с гео токена, но сейчас это не делается).
- Geo no-ads может ослабеть со временем.
- ECH / общий CDN IP → DNS-set недостаточен → SNI-route.
- Path-routing внутри одного HTTPS host без MITM невозможен.

## Ссылки

- [ADR 0003](0003-goal1-router-only-tls.md)
- [OPENTWITCH_LAB.md](../research/OPENTWITCH_LAB.md)
- [TWITCH_TRAFFIC_MAP.md](../research/TWITCH_TRAFFIC_MAP.md)
- [ROADMAP.md](../ROADMAP.md) Stage R
