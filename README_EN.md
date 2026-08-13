# OpenStream Engine

[Читать на русском языке](README.md)

OpenStream Engine is a router-level solution for OpenWrt that bypasses Twitch SSAI (Server-Side Ad Insertion) ads and unlocks high stream quality (1080p/1440p/Source) **without any client-side modifications** (no custom CA certificates, no client apps, no URL changes, no per-device VPNs).

---

## The Concept: Smart Geo-Split (R3)

Traditional ad-blocking methods require decryption of HTTPS traffic (MITM), which requires installing custom root certificates on all client devices (TVs, smartphones, consoles). This introduces significant security risks and is often impossible on closed platforms like Apple TV, WebOS, or Tizen.

OpenStream Engine implements **Smart Geo-Split (R3)** at the network level (DNS/Routing):

| Traffic | Routing | Why? |
|---------|---------|------|
| `gql.twitch.tv` | **Direct WAN (RU IP)** | Twitch issues an access token for the Russian region where ads are disabled by default. |
| `usher.ttvnw.net` | **European VPN / SmartDNS** | Unlocks regional restrictions, enabling 1080p/1440p/Source stream qualities. |
| `*.playlist.ttvnw.net` / CDN | **Direct WAN (RU IP)** | Playlists and video segments (`.ts`) load directly, consuming 0% of your VPN bandwidth and preserving local ISP speeds. |

```text
Client ──► OpenWrt ──DNS/SNI usher.ttvnw.net──► VPN (EU) ──► Twitch (Quality master.m3u8)
              │
              ├──DNS/SNI gql.twitch.tv────────► Direct WAN ──► Twitch (Token show_ads:false)
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
- **No local tables overhead:** When running in `geo_split` mode, the local NAT redirect tables in nftables are completely destroyed to save firewall processing power.

---

## Project Structure

```text
├── crates/                    # Rust source code of the proxy engine daemon
├── docs/                      # Architecture Decision Records (ADR) and research documentation
├── luci-app-openstream/       # LuCI Web UI (Lua / Model / Controller)
├── package/                   # OpenWrt package source files (Makefile, config, init scripts)
├── research/                  # Autolab testing framework for routing combinations
└── scripts/                   # Package building and deployment tools
```

---

## Installation & Configuration (OpenWrt)

Pre-built `.ipk` packages for **Cortex-A53 (aarch64)** are available in [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/).

### 1. Installation

Copy the packages to your router and install them:
```bash
opkg update
opkg install openstream-engine_0.4.2-14_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-14_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-14_all.ipk
```

### 2. Configuration via UCI / CLI

To configure the router via CLI, run:
```bash
# Enable the service
uci set openstream.main.enabled='1'

# Set routing mode to Smart Geo-Split
uci set openstream.proxy.mode='geo_split'

# Set the name of your VPN nftset or ipset
uci set openstream.proxy.geo_split_vpn_set='4#inet#fw4#vpn_domains'

# Apply configuration
uci commit openstream
/etc/init.d/streamproxyd restart
```

### 3. Configuration via LuCI Web UI

Go to **Services → OpenStream** on your router:
- Select **Smart Geo-Split** as the operation mode.
- Enter your VPN nftset/ipset name in the **Smart Geo-Split VPN Set** input field.
- Save & Apply.

---

## License

MIT © 2026 Denis Ershov
