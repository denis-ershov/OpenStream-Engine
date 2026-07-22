# OpenWrt 24.x — Cortex-A53 (release 14)

Актуальные `.ipk` в `ipk/` (только текущий release). Документация: [`docs/INDEX.md`](../../docs/INDEX.md).

## Установка 0.4.2-14

```bash
opkg install --force-reinstall /tmp/openstream-engine_0.4.2-14_aarch64_cortex-a53.ipk
opkg install /tmp/luci-app-openstream_0.4.2-14_all.ipk
# опционально RU:
# opkg install /tmp/luci-i18n-openstream-ru_0.4.2-14_all.ipk

/etc/init.d/streamproxyd restart
wget -qO- http://127.0.0.1:18080/api/status
```

LuCI → **Public Edge URL** = `http://<LAN_IP>:18080` (например `http://192.168.8.1:18080`).

VLC: `http://<LAN_IP>:18080/twitch/<channel>` — в master должны быть nested `http://LAN:18080/https://…`.

## Если Permission denied (exit 126)

```bash
chmod 0755 /usr/bin/streamproxyd /usr/libexec/openstream-* /etc/init.d/streamproxyd
/etc/init.d/streamproxyd restart
```

(r12+ postinst уже делает chmod; r14 pack чинит `+x` в архиве.)

## Релизы (кратко)

| Rel | Фикс |
|-----|------|
| 12+ | exec bits |
| 13 | rustls CryptoProvider (нет panic на Edge TLS) |
| **14** | master rewrite → nested media (strip) |

Полный чеклист: [docs/PERFORMANCE.md](../../docs/PERFORMANCE.md), [docs/COEXISTENCE.md](../../docs/COEXISTENCE.md).
