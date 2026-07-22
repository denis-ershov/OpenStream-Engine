# Документация OpenStream Engine

**Research** toward OpenWrt Goal №1 (все клиенты, ноль действий на устройстве).  
Статус: **`[research]`** — [ADR 0003](adr/0003-goal1-router-only-tls.md) · кандидат [ADR 0004](adr/0004-geo-split-egress.md).

IPK 0.4.2-14 = lab archive (Edge/MITM), не claim Goal №1.

## С чего начать

| Документ | Содержание |
|----------|------------|
| [../README.md](../README.md) | Research front door |
| [adr/0003-goal1-router-only-tls.md](adr/0003-goal1-router-only-tls.md) | Цель №1 |
| [adr/0004-geo-split-egress.md](adr/0004-geo-split-egress.md) | Гипотеза geo-split |
| [research/OPENTWITCH_LAB.md](research/OPENTWITCH_LAB.md) | Gate E0–E4 |
| [research/TWITCH_TRAFFIC_MAP.md](research/TWITCH_TRAFFIC_MAP.md) | Карта трафика |
| [../research/twitch/autolab/README.md](../research/twitch/autolab/README.md) | Автолаб ПК |
| [ROADMAP.md](ROADMAP.md) | Stage R |

## Lab archive

| Документ | |
|----------|--|
| [adr/0002-playlist-edge.md](adr/0002-playlist-edge.md) | Edge (не Goal №1) |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Компоненты lab + Goal №1 |
| [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md) | Режимы lab |
| [COEXISTENCE.md](COEXISTENCE.md) | Соседи |
| [CHANGELOG.md](CHANGELOG.md) | История |

## Слои (lab code)

[HLS](HLS_ARCHITECTURE.md) · [DASH](DASH_ARCHITECTURE.md) · [PLUGIN](PLUGIN_ARCHITECTURE.md) · [SDK](SDK.md) · [PACKAGING](PACKAGING.md) · [BUILD](BUILD_OPENWRT.md) · [PERFORMANCE](PERFORMANCE.md)
