# Архитектура OpenStream Engine

## Цель №1 vs lab

**Цель №1** `[research]`: OpenWrt · все клиенты · ноль действий на устройстве · **без MITM**.  
[ADR 0003](adr/0003-goal1-router-only-tls.md) · кандидат geo-split [ADR 0004](adr/0004-geo-split-egress.md).

```text
Goal №1 (hypothesis):
  Client ──► OpenWrt ──gql/usher──► VPS ──► Twitch
                └──weaver/CDN──► ISP ──► segments
  (no TLS termination, no CA on client)
```

**Lab archive** (не Goal №1): Playlist Edge / MITM strip — [ADR 0002](adr/0002-playlist-edge.md).

Research tools: [OPENTWITCH_LAB](research/OPENTWITCH_LAB.md), [autolab](../research/twitch/autolab/).

## Lab-компоненты (0.4.x archive)

| Компонент | Crate | |
|-----------|-------|--|
| streamproxyd | `streamproxyd` | демон (lab Edge/MITM) |
| Proxy | `ose-proxy` | Edge, nested, optional MITM |
| Manifest / plugins | `ose-manifest`, `ose-plugin-*` | strip HLS/DASH |
| … | см. crates/ | |

Поток lab Edge: клиент **сам** открывает `/twitch/<channel>` → не Goal №1.

## Ключевые решения

- MITM **rejected** для Goal №1.
- Geo-split исследуется до OpenWrt routing package.
- Ядро можно менять после E0–E4.
- Плагины compile-time — [ADR 0001](adr/0001-plugin-abi.md).

## Версии

[ROADMAP.md](ROADMAP.md) Stage R · [INDEX.md](INDEX.md).
