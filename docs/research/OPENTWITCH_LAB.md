# OpenTwitch Lab

Протокол исследования цепочки Twitch **без MITM**. Цель: найти Goal1-совместимый механизм ([ADR 0004](../adr/0004-geo-split-egress.md)).

Оглавление docs: [INDEX.md](../INDEX.md). Карта хостов: [TWITCH_TRAFFIC_MAP.md](TWITCH_TRAFFIC_MAP.md).  
Автолаб ПК: [`research/twitch/autolab/`](../../research/twitch/autolab/).

## Цепочка (эталон)

```text
Client
  → DNS / TLS SNI
  → gql.twitch.tv          (PlaybackAccessToken, …)
  → usher.ttvnw.net        (master.m3u8)
  → playlist / weaver hosts (media.m3u8)
  → video-weaver / CDN     (segments)
```

Уточняется захватом (E0). Не считать список хостов финальным до autolab/HAR.

## Gate-тесты

| ID | Вопрос | Pass | Инструмент |
|----|--------|------|------------|
| **E0** | Хосты + порядок с живой сессии | gql+usher (и др.) в flow_map | `run_lab.py` browser |
| **E1** | Token+master через SOCKS VPS; media/segments direct | HTTP 200 | `run_lab.py --socks5` |
| **E2** | Минимальный set хостов через VPS для no-ads | ⊆ Map | отчёт + ручная сводка |
| **E3** | Ads markers / midroll при split | нет `stitched` / ad DATERANGE (или нет midroll) | media download |
| **E4** | Source / 1440p в master | варианты есть | parse master |

Без pass E0–E3 — **не** собирать продакшн OpenWrt geo-split.

## Матрица стран (шаблон)

Заполнять после прогонов (SOCKS exit в стране):

| Страна | Ads | Source | 1440 | 1080 | 403 | Дата | Примечание |
|--------|-----|--------|------|------|-----|------|------------|
| PL | ? | ? | ? | ? | ? | | |
| DE | ? | ? | ? | ? | ? | | |
| US | ? | ? | ? | ? | ? | | |
| ISP home | да | да | нет | да | нет | 2026-08-13 | E3 fail (обнаружены ads markers), E4 pass |

## Комбо-маршруты (Результаты E1-E4)

По результатам автоматического тестирования комбо-маршрутов (сессия `20260813T145126Z_gaules`):

| Маршрут | GQL (Token) | Usher (Master) | Media | Segments | Качество (1080p) | Реклама? | Вывод / Статус |
|---------|-------------|----------------|-------|----------|------------------|----------|----------------|
| **R0** (Direct) | RU (direct) | RU (direct) | RU | RU | Да | Нет | Чистый РФ путь. Работает, так как в РФ нет рекламы. |
| **R1** (Base Split) | EU (proxy) | EU (proxy) | RU | RU | Да | Да (при midroll) | Базовый geo-split. Получаем рекламу, так как токен европейский. |
| **R3** (Smart Split) | RU (direct) | EU (proxy) | RU | RU | Да | **Нет** | **Рекомендуемый.** Токен RU (без рекламы) + Master EU (высокое качество). |
| **R2** (Reverse Split) | EU (proxy) | RU (direct) | RU | RU | Да | Да | Реверсивный split. Реклама есть из-за GQL EU токена. |

### Ключевой вывод
Для обхода рекламы при сохранении качественного потока (1080p/1440p) на роутере необходимо направить:
- `gql.twitch.tv` -> **Direct (РФ IP)** (получение токена без SSAI видеорекламы)
- `usher.ttvnw.net` -> **VPN/SmartDNS (Европейский IP)** (разблокировка 1080p/1440p/Source качеств)
- `*.playlist.ttvnw.net` и CDN-домены -> **Direct (РФ IP)** (прямой поток на максимальной скорости)
- `edge.ads.twitch.tv` -> **DNS Block / 0.0.0.0** (блокировка баннеров и ad-трекеров)

Path-routing внутри одного HTTPS host без MITM недоступен; разделение по доменам GQL (RU) и Usher (EU) полностью решает задачу без дешифрации TLS.

## HTTP/3

На lab-стенде опционально drop `udp/443`, чтобы браузер ушёл на TCP/H2 (проще HAR). Не требование продукта.

## Автолаб

```bash
cd research/twitch/autolab
pip install -r requirements.txt && playwright install chromium
python run_lab.py --channel CHANNEL --browser-only
python run_lab.py --channel CHANNEL --socks5 socks5://127.0.0.1:1080
```

См. [autolab/README.md](../../research/twitch/autolab/README.md).

## Связь с Goal №1

- MITM / Edge / companion — не Goal №1 ([ADR 0003](../adr/0003-goal1-router-only-tls.md)).
- Кандидат — geo-split ([ADR 0004](../adr/0004-geo-split-egress.md)).
