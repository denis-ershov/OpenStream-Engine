# OpenStream Engine — Roadmap

Актуальная база: **0.4.2** (IPK lab **14**). Оглавление: [INDEX.md](INDEX.md).

Связанные: [ARCHITECTURE.md](ARCHITECTURE.md), [adr/0003-goal1-router-only-tls.md](adr/0003-goal1-router-only-tls.md), [adr/0004-geo-split-egress.md](adr/0004-geo-split-egress.md), [research/OPENTWITCH_LAB.md](research/OPENTWITCH_LAB.md), [research/TWITCH_TRAFFIC_MAP.md](research/TWITCH_TRAFFIC_MAP.md).

## Легенда

`[x]` done · `[~]` partial · `[ ]` todo · `[blocked]` impossible · `[research]` active · `[lab]` not Goal №1

---

## Цель №1 — `[research]`

[ADR 0003](adr/0003-goal1-router-only-tls.md) · кандидат [ADR 0004](adr/0004-geo-split-egress.md).

| Требование | |
|------------|--|
| OpenWrt / роутер | да |
| Все клиенты (TV/PC/phone/STB; app/browser) | да |
| Ноль действий на клиенте | да |
| MITM | **rejected** |

Content inspect HTTPS без клиента = permanently blocked. Кандидат: **geo-split egress** (gql/usher → VPS, segments → ISP).

---

## Stage R — OpenTwitch Research (фокус)

| ID | Задача | Статус |
|----|--------|--------|
| R0 | ADR 0003/0004; Traffic Map + Lab | `[x]` |
| R1 | Autolab Playwright + PC E0–E4 | `[x]` |
| R2 | E0: живая сессия → Map | `[ ]` |
| R3 | E1–E4 с VPS SOCKS | `[ ]` |
| R4 | Матрица стран | `[ ]` |
| R5 | OpenWrt policy routing (после R3) | `[ ]` |

Автолаб: [`research/twitch/autolab/`](../research/twitch/autolab/).

---

## Ближайший спринт

1. `[x]` Research docs + ADR 0004 + README research front.
2. `[x]` Autolab `run_lab.py`.
3. `[ ]` Прогон E0–E4 на живом канале + SOCKS.
4. `[ ]` Не развивать MITM/Edge/H7 как Goal №1.
5. `[lab]` IPK Edge — archive only.

---

## Lab archive (не Цель №1)

Edge/MITM: [ADR 0002](adr/0002-playlist-edge.md). Stage H/F/G ниже — исторический lab, не продукт Goal №1.

### Stage H — Playlist Edge `[lab]`

| ID | Статус |
|----|--------|
| H0–H6, H9–H10 | `[x]` lab |
| H7 companion | `[ ]` · не Goal №1 |
| H8 VLC hint | `[~]` |

### Stage A–G (кратко)

Платформа 0.4.x (MITM/plugins/DASH/SDK/perf) — `[x]` / `[~]` как раньше; MITM path = `[lab]` rejected for Goal №1. Детали в git history / PERFORMANCE.

### Вне скоупа Goal №1

| Тема | |
|------|--|
| HTTPS inspect без клиента | blocked TLS |
| MITM/Edge/companion as Goal №1 | rejected |
| Turbo / TLS exploits | out |

Пакет lab: **0.4.2-14**. Goal №1: **research** (geo-split).
