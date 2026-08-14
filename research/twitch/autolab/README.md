# OpenTwitch Autolab (ПК)

Автоматический захват трафика Twitch в браузере (Playwright) + клиентские тесты E0–E4 без MITM.

См. [OPENTWITCH_LAB.md](../../../docs/research/OPENTWITCH_LAB.md), [ADR 0004](../../../docs/adr/0004-geo-split-egress.md).

## Установка

```bash
cd research/twitch/autolab
python -m venv .venv
# Windows:
.venv\Scripts\activate
# Linux/macOS:
# source .venv/bin/activate
pip install -r requirements.txt
playwright install chromium
```

## Запуск

```bash
# Только браузер → карта хостов (E0)
python run_lab.py --channel gohamedia --browser-only

# Браузер + клиент (E3/E4 local); E1/E2 skip без SOCKS
python run_lab.py --channel gohamedia --duration 60

# Полный прогон с VPS SOCKS (E1)
python run_lab.py --channel gohamedia --socks5 socks5://127.0.0.1:1080 --duration 90

# Видимый браузер (age-gate / captcha)
python run_lab.py --channel gohamedia --headed --browser-only
```

Артефакты: `../results/<session_id>/` — `capture.har`, `network.json`, `flow_map.json`, `REPORT.md`, `report.json`.

## Ограничения

- Midroll за N секунд не гарантирован.
- Age-gate / login могут потребовать `--headed` и ручной клик; профиль: `user_data/`.
- Не ставит ничего на ТВ/телефон — только lab на ПК.
- Без `--socks5` гейты E1–E2 = `skipped`.

## Проверка действующей DNS-стратегии роутера

```bash
# ПК подключён к LAN роутера; 192.168.1.1 — локальный dnsmasq.
python router_dns_probe.py --channel gohamedia --resolver 192.168.1.1 --duration 600 --output router-report.json
```

`router_dns_probe.py` не использует DoH, системный DNS, VPN, proxy или MITM.
Он получает A-записи только от заданного DNS роутера и закрепляет их за TLS с
исходным SNI. Поэтому проверяется реальная связка dnsmasq/OpenStream/Forkop.
`pass` означает `show_ads=false` у токена и отсутствие SSAI-маркеров во всех
опрошенных media playlist. Запускайте отдельно после применения каждого пресета;
не меняйте Forkop в ходе одного прогона.
