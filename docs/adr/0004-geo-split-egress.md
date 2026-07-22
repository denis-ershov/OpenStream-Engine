# ADR 0004 — Geo-split egress (кандидат Goal №1 без MITM)

- **Статус:** Accepted — hypothesis `[research]`
- **Дата:** 2026-07-22
- **Связь:** кандидат к [ADR 0003](0003-goal1-router-only-tls.md); не заменяет TLS-инвариант для content inspect

## Контекст

Цель №1: OpenWrt-пакет, все клиенты (ТВ / ПК / телефон / приставки; app / browser), **ноль** действий на клиенте.  
MITM / Edge / companion **отвергнуты** как продукт Goal №1 (требуют CA или URL/расширение).

Content inspection HTTPS на роутере без trust на клиенте невозможна ([ADR 0003](0003-goal1-router-only-tls.md)).

## Гипотеза

Не читать TLS. Менять только **маршрут** TCP для небольшого набора API-хостов:

| Трафик (стартовая гипотеза) | Exit |
|-----------------------------|------|
| `gql.twitch.tv` | VPS в регионе без SSAI-ads (PL/DE/…) |
| `usher.ttvnw.net` | тот же VPS |
| `video-weaver*` / сегменты CDN | ISP напрямую |

Клиент по-прежнему устанавливает TLS к настоящему Twitch (валидный cert). Роутер: DNS/nftset или SNI-route → WireGuard/SOCKS **на роутере** (не VPN-профиль на ТВ/телефоне).

Объём через VPS: порядка десятков КБ (token + master), не видеопоток.

```text
Client ──► OpenWrt ──SNI gql/usher──► VPS ──► Twitch API
              │
              └──SNI weaver/CDN──► ISP ──► segments
```

Финальный список хостов — только после [TWITCH_TRAFFIC_MAP](../research/TWITCH_TRAFFIC_MAP.md) / autolab E0 (не фиксировать `gql`+`usher` как закон до захвата).

## Gate до продукта OpenWrt

| ID | Вопрос | Pass |
|----|--------|------|
| E0 | Карта хостов/порядка с реальной сессии | [OPENTWITCH_LAB](../research/OPENTWITCH_LAB.md) |
| E1 | Token+master через VPS; media/segments с ISP | HTTP 200, стрим |
| E2 | Минимальный host set через VPS | ⊆ Traffic Map |
| E3 | Нет ads-маркеров / midroll при split | pass |
| E4 | Source/1440p в master | pass |

Инструмент: [`research/twitch/autolab/`](../../research/twitch/autolab/).

## Решение

1. MITM / CA / companion / Edge — **не** продукт Goal №1.
2. Кандидат Goal №1 = **geo-split egress** (этот ADR); статус `[research]` до E0–E4.
3. Lab `streamproxyd` Edge остаётся архивом ([ADR 0002](0002-playlist-edge.md)), не claim Goal №1.
4. После pass E0–E3 — OpenWrt: policy routing (dnsmasq+nftset, при коллизиях SNI-router), не strip-движок.

## Риски

- IP-binding токена / playlist → E1 fail → расширить VPS на media.m3u8 (не сегменты) или отвергнуть гипотезу.
- Geo no-ads может ослабеть со временем.
- ECH / общий CDN IP → DNS-set недостаточен → SNI-route.
- Path-routing внутри одного HTTPS host без MITM невозможен.

## Ссылки

- [ADR 0003](0003-goal1-router-only-tls.md)
- [OPENTWITCH_LAB.md](../research/OPENTWITCH_LAB.md)
- [TWITCH_TRAFFIC_MAP.md](../research/TWITCH_TRAFFIC_MAP.md)
- [ROADMAP.md](../ROADMAP.md) Stage R
