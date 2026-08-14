# OpenStream Engine

[Читать на русском языке](README.md)

OpenStream Engine is a router-level solution for OpenWrt that bypasses Twitch SSAI (Server-Side Ad Insertion) ads and unlocks high stream quality (1080p/1440p/Source) **without any client-side modifications** (no custom CA certificates, no client apps, no URL changes, no per-device VPNs).

---

## The Concept: Smart Modular Split Routing

Traditional ad-blocking methods require decryption of HTTPS traffic (MITM), which requires installing custom root certificates on all client devices (TVs, smartphones, consoles). This introduces significant security risks and is often impossible on closed platforms like Apple TV, WebOS, or Tizen.

OpenStream Engine implements **transparent modular routing** at the network core level (DNS/Routing):

| LuCI Preset | Token (GQL) | Master (Usher) | Media/Segments (CDN) | Banners (Ads) |
|---|---|---|---|---|
| 🇷🇺 **"Russia/CIS: No ads + 1440p"** *(Recommended)* | **Direct WAN (RU)** | **VPN EU** | **Direct WAN (RU)** | **DNS Block (0.0.0.0)** |
| 🇪🇺 **"Europe/US: Bypass ads via RU token"** | **VPN RU** | **Direct WAN (EU)** | **Direct WAN (EU)** | **DNS Block (0.0.0.0)** |
| 🌍 **"Quality Unlock: 1440p/Source"** | **Direct WAN** | **VPN EU/US** | **Direct WAN** | **DNS Block (0.0.0.0)** |
| 🛡️ **"Full Bypass: Complete VPN Routing"** | **VPN EU** | **VPN EU** | **VPN EU** | **DNS Block (0.0.0.0)** |
| ⚙️ **"Custom: Fine-grained matrix"** | *Custom* | *Custom* | *Custom* | *Custom* |

```text
Client ──► OpenWrt ──DNS/SNI usher.ttvnw.net──► VPN (EU) ──► Twitch (Quality master.m3u8)
              │
              ├──DNS/SNI gql.twitch.tv────────► Direct WAN ──► Twitch (Token show_ads:false)
              │
              ├──DNS edge.ads.twitch.tv───────► 0.0.0.0  ──► (Banners & ad trackers blocked)
              │
              └──DNS/SNI playlist/segments────► Direct WAN ──► CDN (Max speed direct stream)
```

---

## Key Benefits

### 🛡️ Security by Design
- **No MITM / TLS Decryption:** Client devices establish direct TLS connections to Twitch servers with original, valid certificates.
- **No CA Certificates Needed:** No need to trust or install custom certificates on your smart TVs, smartphones, or computers.
- **Strict Parameter Sanitization:** All custom UCI parameters are strictly validated to prevent command injections.

### ⚡ Performance & Resource Efficiency
- **Minimal RAM/CPU Usage:** Video segments do not pass through the local proxy server. Routing is handled natively by the kernel using `dnsmasq` and high-performance `nftset/ipset` tables.
- **VPN Bandwidth Savings:** Only small API requests (~10–20 KB per stream launch) are routed through your VPN. Video traffic uses your direct ISP link.

---

## Installation & Configuration (OpenWrt)

Pre-built `.ipk` packages for **Cortex-A53 (aarch64)** are available in [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/).

### 1. Installation

Copy the packages to your router and install them:
```bash
opkg update
opkg install openstream-engine_0.4.2-31_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-31_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-31_all.ipk
```

### 2. Configuration via LuCI Web UI

Go to **Services → OpenStream Engine** on your router:
- **Dashboard:** Monitor active modules and live traffic streams.
- **Twitch:** Select your preferred preset in 1-click (e.g. *«🇷🇺 Russia/CIS: No ads + 1440p/Source»*).
- **Diagnostics:** Verify DNS resolution and ad-blocking status instantly.

---

## License

MIT © 2026 Denis Ershov
