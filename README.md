# OpenStream Engine

**Исследовательский** проект: путь к пакету **OpenWrt**, который убирает/избегает рекламу стриминга **без действий на клиентах** и покрывает **все** устройства в LAN.

Статус Цели №1: **`[research]`** — не «уже работает на всех ТВ».

## Цель №1

| Требование | |
|------------|--|
| Логика на роутере (OpenWrt) | да |
| ТВ, ПК, телефон, приставки; app и browser | да |
| Без CA, расширений, смены URL, VPN-профиля на устройстве | да |
| MITM / подмена сертификатов | **отвергнуто** |

См. [ADR 0003](docs/adr/0003-goal1-router-only-tls.md).

## Почему не MITM и не «Edge URL»

- Читать/менять HTTPS m3u8 на роутере без trust на клиенте **нельзя** (TLS).
- Playlist Edge / companion требуют действия на клиенте → не Цель №1 ([ADR 0002](docs/adr/0002-playlist-edge.md) = lab archive).

## Гипотеза исследования: geo-split

Не трогать TLS-содержимое. Маршрутизировать на роутере только лёгкие API-запросы (GQL / usher) через VPS в регионе без SSAI-ads; сегменты — напрямую через ISP.

См. [ADR 0004](docs/adr/0004-geo-split-egress.md). Подтверждается gate-тестами E0–E4.

```text
Client ──► OpenWrt ──gql/usher──► VPS (ad-free region) ──► Twitch
              └──video-weaver/CDN──► ISP ──► segments
```

## Сейчас в репозитории (Stage R)

| Компонент | Назначение |
|-----------|------------|
| [OpenTwitch Lab](docs/research/OPENTWITCH_LAB.md) | Протокол E0–E4, матрица стран |
| [Traffic Map](docs/research/TWITCH_TRAFFIC_MAP.md) | Что откуда и куда |
| [`research/twitch/autolab/`](research/twitch/autolab/) | Playwright + PC client: карта трафика и тесты |
| `streamproxyd` / IPK 0.4.2 | **Lab archive** (Edge/MITM) — не claim Цели №1 |

## Как участвовать

```bash
cd research/twitch/autolab
python -m venv .venv
# Windows: .venv\Scripts\activate
pip install -r requirements.txt
playwright install chromium
python run_lab.py --channel CHANNEL --browser-only
# с VPS SOCKS:
python run_lab.py --channel CHANNEL --socks5 socks5://127.0.0.1:1080
```

Документы: [INDEX](docs/INDEX.md) · [ROADMAP](docs/ROADMAP.md) · [CHANGELOG](docs/CHANGELOG.md).

## Не обещаем сейчас

- Рабочий ads-free на всех клиентах «из коробки» сегодня.
- Strip/MITM без клиентских действий.
- Продакшн OpenWrt geo-split до прохождения E0–E4.

## Лицензия

MIT © 2026 Denis Ershov
