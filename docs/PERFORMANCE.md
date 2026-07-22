# Performance notes

Цели OpenWrt (рядом с zapret/podkop): **RSS ≤ ~30 МБ**, parse/strip **≪ 1 мс** на типичном media playlist, CPU умеренный на A53.

Оглавление: [INDEX.md](INDEX.md).

## Criterion benches (host)

```bash
cargo bench -p ose-manifest --bench parse_hls
cargo bench -p ose-plugin-twitch --bench strip_twitch
cargo bench -p ose-dash --bench filter_mpd
```

Ориентиры на x86_64 desktop (порядок величины, не SLA устройства):

| Bench | Ожидание |
|-------|----------|
| `hls_parse_media` | десятки µs |
| `twitch_strip_midroll` | десятки–сотни µs |
| `dash_filter_ad_period` | сотни µs |

На aarch64/mips цифры выше; фиксировать после полевого замера.

## Feature flags (размер бинаря)

```bash
cargo build --release -p streamproxyd
cargo build --release -p streamproxyd --no-default-features --features slim-twitch
```

| Feature | Содержимое |
|---------|------------|
| `plugin-twitch` | Twitch strip |
| `plugin-hls` | Kick/Trovo/YouTube + `rules_file` |
| `plugin-dash` | DASH plugin + proxy MPD inspect |
| `slim-twitch` | только `plugin-twitch` |

## Размер артефактов (host cross, 0.4.2)

Cortex-A53 / `aarch64-unknown-linux-musl`, `release` (LTO + `opt-level=z` + `panic=abort`):

| Артефакт | Размер |
|----------|--------|
| `streamproxyd` (full) | ~3.1 МБ (ELF stripped) |
| `streamproxyd` (`slim-twitch`) | ~3.0 МБ |
| `openstream-engine_*.ipk` | ~1.7 МБ |
| `luci-app-openstream_*.ipk` | ~5–7 КБ |
| `luci-i18n-openstream-ru_*.ipk` | ~2.6 КБ |

Каталог: `dist/openwrt-24.10-a53/` (в `ipk/` — только текущий release, сейчас **14**).

---

## Полевой прогон: idle RSS (GL-MT6000, 2026-07-22)

Устройство: **GL.iNet GL-MT6000** (Filogic 830, 4× Cortex-A53), OpenWrt, `streamproxyd` full.

```text
VmPeak:    13564 kB
VmRSS:      2872 kB
top:       VIRT≈13452  RES≈1% RAM  CPU≈0%
```

| SoC | Сборка | VmRSS idle | VmPeak | CPU idle | VmRSS 1× Twitch Edge | Дата |
|-----|--------|------------|--------|----------|----------------------|------|
| GL-MT6000 (A53) | full 0.4.2 | **~2.8 МБ** | **~13.3 МБ** | ~0 % | *TBD* | 2026-07-22 |

Idle RSS **≪ 30 МБ**. Замер под Edge (master+media strip) — после стабильного nested rewrite (r14+).

```bash
cat /proc/$(pidof streamproxyd)/status | grep -E 'VmRSS|VmPeak'
top -b -n1 | grep streamproxyd
```

---

## Полевой прогон: Edge smoke (хронология)

### Ранний MITM/explicit (`playlists_total=0`)

Strip не видел HLS — трафик мимо Engine или MITM без CA. Неактуально как default.

### Edge без CryptoProvider (&lt; r13)

`GET /twitch/…` → panic rustls → `panic=abort` → procd restart loop.  
Фикс: `ring::default_provider().install_default()` (r13).

### Edge без master rewrite (&lt; r14)

`playlists_total` растёт (masters), `ads_found=0`: media шли на CDN.  
Фикс: `proxy_public_url` / Host → nested rewrite (r14).

### Успешный Edge (ожидание r14+)

| Метрика | Успех |
|---------|--------|
| `openstream_playlists_total` | ≥1 (лучше растёт при media refresh) |
| `openstream_ads_found_total` | ≥1 при midroll |
| `openstream_segments_removed_total` | ≥1 при midroll |
| Master body | nested `http://LAN:18080/https://…` |
| Лог | нет panic `CryptoProvider` |

---

## Чеклист Edge на роутере

```bash
opkg list-installed | grep openstream
# openstream-engine … 0.4.2-14

/etc/init.d/streamproxyd restart
wget -qO- http://127.0.0.1:18080/api/status

# Public Edge URL в LuCI или проверка с LAN:
wget -qO- "http://$(uci get network.lan.ipaddr):18080/twitch/CHANNEL" | head -40

# VLC: тот же URL, дождаться midroll
wget -qO- http://127.0.0.1:18080/metrics | grep -E 'playlists|ads_found|segments'
logread -e streamproxyd | grep -iE 'edge|stripped|CryptoProvider|panic|warn'
```

Legacy transparent (CA): `scripts/smoke-transparent-mt6000.sh`, nft set, CA на клиенте.

---

## Связанные документы

- [PROXY_ARCHITECTURE.md](PROXY_ARCHITECTURE.md) — Edge / rewrite / MITM  
- [COEXISTENCE.md](COEXISTENCE.md) — соседи  
- [ROADMAP.md](ROADMAP.md) — Stage E / H  
- [PACKAGING.md](PACKAGING.md) — release IPK
