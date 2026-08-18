# OpenStream Engine 🧪 (Beta / Research Project)

[Читать на русском языке](README.md)

> [!NOTE]
> **Research Project Status (Experimental Beta):**
> OpenStream Engine is an OpenWrt research toolkit designed to investigate router-level SSAI (Server-Side Ad Insertion) mitigation and 1080p60/1440p/Source quality unlocking for live streaming platforms (Twitch, etc.) **without TLS decryption (No MITM) and without installing custom Root CA certificates on client devices**.

---

## 🔬 Research & Live Testing Results

Based on our recent audit and live probes across top Twitch streams (20k+ concurrent viewers):
1. **Generic Geo-Split Limitations:** Twitch has updated its SSAI policy. Direct Russian ISP egress now receives `show_ads: true` and stitched Amazon video ads on partner channels. Naive DNS-only redirect to generic RU DNS is no longer sufficient for ad-free playback.
2. **Verified Working Strategies:**
   * **Strategy 1: Geo-Split via Clean-Proxy / Ad-Free VPN (Zero-CA for all devices):** Routes authorization tokens (`gql.twitch.tv`) through ad-free VPN egresses (e.g. Ukraine, Albania, Kazakhstan), master playlists (`usher.ttvnw.net`) through EU/SmartDNS for 1080p60/1440p quality, and heavy video segments directly through local ISP WAN.
   * **Strategy 2: Playlist Edge / Local Manifest Stripping (100% Guaranteed for custom players):** Local `streamproxyd` daemon running on port 18080 fetches manifests, strips `#EXT-X-DATERANGE:CLASS="twitch-stitched-ad"` blocks, and serves clean HLS (`http://router:18080/twitch/<channel>`). Ideal for VLC, Kodi, SmartTube, TiviMate, MPV, and streamlink.

---

## Split Routing Architecture

```text
                               ┌────────────────── OpenStream Engine ──────────────────┐
                               │                                                       │
Client (TV/Phone/PC) ──────────┼──► DNS/SNI gql.twitch.tv ────► Ad-Free VPN (UA/AL/KZ) ──► Twitch (Token show_ads:false)
(No CA, valid TLS)             │                                                       │
                               ├──► DNS/SNI usher.ttvnw.net ──► EU VPN / SmartDNS ────► Twitch (Unlock 1440p/Source)
                               │                                                       │
                               ├──► DNS/SNI cdn/live-video ───► Direct WAN (ISP) ──────► CDN (Full Speed Stream)
                               │                                                       │
                               └──► DNS edge.ads.twitch.tv ───► 0.0.0.0 (Sinkhole)       (Banners/trackers blocked)
```

### LuCI Preset Matrix

| LuCI Preset | Token (GQL) | Master (Usher) | Media & Segments (CDN) | Banners (Ads) |
|---|---|---|---|---|
| 🛡️ **"Geo-Split via Clean-Proxy"** *(Recommended)* | **Ad-Free VPN (UA/AL/KZ)** | **SmartDNS / EU VPN** | **Direct WAN (ISP)** | **DNS Block (0.0.0.0)** |
| ⚡ **"Playlist Edge (Manifest Strip)"** | **via streamproxyd** | **via streamproxyd** | **Direct CDN (via Edge)** | **Stripped from HLS** |
| 🌍 **"Quality Unlock: 1440p/Source"** | **Direct WAN** | **SmartDNS / EU VPN** | **Direct WAN (ISP)** | **DNS Block (0.0.0.0)** |
| ⚙️ **"Custom: Fine-grained Matrix"** | *Custom* | *Custom* | *Custom* | *Custom* |

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
- **Twitch:** Select your preferred preset in 1-click (e.g. *«🛡️ Geo-Split via Clean-Proxy / Ad-Free VPN»*).
- **Diagnostics:** Verify DNS resolution and ad-blocking status instantly.

---

## License

MIT © 2026 Denis Ershov
