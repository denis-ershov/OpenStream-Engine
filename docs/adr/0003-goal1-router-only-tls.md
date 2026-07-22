# ADR 0003 — Цель №1: только роутер, все клиенты, ноль действий на устройстве

- **Статус:** Accepted — Goal `[research]` (inspect TLS = blocked; кандидат = [ADR 0004](0004-geo-split-egress.md))
- **Дата:** 2026-07-22
- **Обновлено:** 2026-07-22 (MITM out; geo-split candidate)
- **Связь:** supersedes product claim of [ADR 0002](0002-playlist-edge.md); кандидат пути — [ADR 0004](0004-geo-split-egress.md)

## Цель №1 (строго)

1. Решение для **OpenWrt** (логика на роутере).
2. Клиенты: ТВ, ПК, телефон, приставки; **приложение или браузер** — полное покрытие.
3. **Ноль** действий на клиенте: ни CA, ни смена URL, ни companion/расширение, ни VPN-профиль на устройстве, ни «пакет только для браузера» как замена.

Компромиссы по п.3 **не принимаются**.

**Разрешено на роутере:** WireGuard/SOCKS/VPS exit, nft/DNS/SNI routing — это не установка VPN на ТВ/телефон.

## Ядро

Ради Goal №1 **разрешено** ломать `ose-proxy`, nft, режимы, ABI. Ограничение для *чтения* HTTPS — TLS на клиенте, не «нельзя трогать код».

## Инвариант TLS (content inspect)

Клиент проверяет сертификат сервера. Роутер на пути видит ciphertext.

- Passiveive decrypt / active MITM без trust на клиенте — **невозможны** как Goal №1.
- **MITM навсегда отвергнут** как продукт Goal №1.

Это **не** запрещает менять **маршрут** пакетов по SNI/DNS без терминации TLS ([ADR 0004](0004-geo-split-egress.md)).

## Матрица отвергнутых путей (не Goal №1)

| Путь | Почему отвергнут |
|------|------------------|
| MITM + CA | Действие на каждом устройстве; **rejected** |
| DNS hijack usher → IP роутера + fake cert | Без CA — TLS fail |
| nft divert HLS без CA | MITM без trust |
| Playlist Edge / companion | URL или расширение на клиенте |
| Browser-only IPK | Не покрывает app/TV |
| DPI-guess / block ad CDN | SSAI / ломает стрим |
| Публичный CA / эксплойты TLS | Вне продукта |

## Решение

1. Goal №1 — продуктовая цель №1.
2. **Content inspect** без клиента = permanently blocked (TLS).
3. **Кандидат:** geo-split egress без MITM — [ADR 0004](0004-geo-split-egress.md), статус `[research]` до gate E0–E4.
4. [ADR 0002](0002-playlist-edge.md) Edge/MITM — lab archive, не Goal №1.
5. Рефактор под geo-split — после эмпирики Lab; не развивать strip/MITM как Goal №1.

## Последствия

- README / Roadmap: research front; MITM rejected.
- Stage R (OpenTwitch Lab) — текущий фокус.
- H7 companion **не** решение Goal №1.

## Ссылки

- [0004-geo-split-egress.md](0004-geo-split-egress.md)
- [../research/OPENTWITCH_LAB.md](../research/OPENTWITCH_LAB.md)
- [ROADMAP.md](../ROADMAP.md)
- [0002-playlist-edge.md](0002-playlist-edge.md)
