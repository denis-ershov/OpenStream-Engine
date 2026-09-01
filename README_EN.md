# OpenStream Engine 🧪 (Beta / Research Project)

[Читать на русском языке](README.md)

> [!NOTE]
> **Research Project Status (Experimental Beta):**  
> OpenStream Engine is an open research lab and declarative network traffic policy engine. The project explores selective service-based routing (Twitch, YouTube, Crunchyroll, etc.), server-side ad insertion (SSAI) mitigation, quality unlocking (1080p60 / 1440p / Source), and local DPI desynchronization (Anti-DPI) **without TLS decryption (No MITM) and without installing custom Root CA certificates on client devices**.

---

## 🌟 OpenStream 2.0 Vision: "One Rule. Every Platform. Zero Overhead."

Most existing networking utilities fall into two extremes: either monolithic VPN clients forcing all traffic through a single remote tunnel, or fragmented OS-specific scripts.

**OpenStream Engine 2.0** introduces a cross-platform declarative **Service-Based Policy Routing Engine**:
* **Universal Rule Specification (`.osrule.yaml`)**: A single declarative rule runs deterministically across **OpenWrt, iOS, Android, macOS, Windows, and Linux**.
* **Ultra-Lightweight Rust Core (`openstream-core`)**: Zero-allocation Reverse Suffix Trie ($O(k)$) domain matching and CIDR routing tables. Under 2 MB RAM usage without any Garbage Collector overhead — making it ideal for budget 128 MB RAM routers and Apple's strict iOS NetworkExtension sandbox (15–50 MB RAM ceiling).
* **Hybrid Egress Strategies**: Within the same rule manifest, one domain can route to a secure tunnel (`Proxy`), another directly with local TLS ClientHello fragmentation (`DpiEvasiveDirect`), video streams at unthrottled direct ISP speeds (`Direct`), and telemetry/ads to `Block` (0.0.0.0 sinkhole).

---

## 🔬 Analysis of Reference Projects

During the architecture design of OpenStream 2.0, we benchmarked and analyzed several key open-source solutions:

| Project | Stack | Strengths | Architectural Limitations | OpenStream 2.0 Approach |
|---|---|---|---|---|
| **Podkop** | OpenWrt + `sing-box` (Go) | User-friendly LuCI web interface, geosite/geoip list support. | Large Go binary (25–40 MB), high RAM footprint due to Go runtime GC. Cannot run on flash-constrained routers or iOS NetworkExtension sandbox. | Adopt the clean LuCI UX pattern, but implement the core in native lightweight Rust (<2 MB RAM). |
| **Forkop** | OpenWrt + sing-box + Zapret + ByeDPI | **Hybrid approach**: combines VPN tunnels (VLESS, Hysteria2) with local Anti-DPI tools. | Multiple separate background daemons glued with bash scripts; potential race conditions in nftables/iptables. | **Built-in hybrid strategies**: rules natively decide whether to tunnel, desync DPI locally, or route direct within a single unified engine. |
| **Zapret / Zapret2** (`bol-van`) | C + NFQUEUE (`nfqws`) + `tpws` | Advanced low-level packet modifications (TCP segmentation, fake payload, SNI split). | Requires root, Linux NFQUEUE or WinDivert; complex syntax; cannot run inside iOS sandbox. | Model as a platform Capability (`L4_PACKET_DESYNC`) for routers and adapt basic SNI splitting in userspace for mobile. |
| **SpoofDPI** (`xvzc`) / **ByeDPI** | Go / C (Android VpnService) | Local `ClientHello` segmentation at SNI boundaries bypasses ISP DPI without remote VPN servers. | Standalone proxies without policy routing or multi-platform rule ecosystems. | Integrate `DpiEvasiveDirect` directly into OpenStream Core for unthrottled 1 Gbps direct streaming without VPS costs. |
| **tun-rs** | Rust async Tokio TUN | Cross-platform TUN with native Apple iOS support (`AsyncDevice::from_fd`). | Requires strict descriptor lifecycle handling. | Official networking foundation for OpenStream 2.0 mobile backends. |

---

## 📋 Declarative Rule Example (`.osrule.yaml`)

```yaml
schema_version: "2.0"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.0.0"

matches:
  - group: "auth_token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree_egress" # Route to zero-ad region (UA/AL/KZ)

  - group: "master_playlist"
    domains: ["usher.ttvnw.net"]
    strategy: "quality_unlock" # Route to EU/SmartDNS for 1080p/1440p

  - group: "video_cdn"
    domains: ["*.live-video.net", "*.ttvnw.net"]
    action: "direct" # Direct ISP WAN for full speed and zero lag

  - group: "ads"
    domains: ["edge.ads.twitch.tv"]
    action: "block" # DNS Sinkhole (0.0.0.0)

strategies:
  adfree_egress:
    preference: ["geo:al", "geo:ua", "geo:kz", "proxy:clean_relay", "direct"]
  quality_unlock:
    preference: ["smartdns:eu", "geo:de", "direct"]
```

---

## 🚀 Current Status: OpenWrt Ready (Release 0.4.2-35)

For routers running OpenWrt 24.10 (**Cortex-A53 / aarch64**), a stable package release with LuCI Web UI is readily available.

### LuCI Preset Matrix

| LuCI Preset | Token (GQL) | Master (Usher) | Media & Segments (CDN) | Banners (Ads) |
|---|---|---|---|---|
| 🛡️ **"Geo-Split via Clean-Proxy"** *(Recommended)* | **Ad-Free VPN (UA/AL/KZ)** | **SmartDNS / EU VPN** | **Direct WAN (ISP)** | **DNS Block (0.0.0.0)** |
| ⚡ **"Playlist Edge (Manifest Strip)"** | **via streamproxyd** | **via streamproxyd** | **Direct CDN (via Edge)** | **Stripped from HLS** |
| 🌍 **"Quality Unlock: 1440p/Source"** | **Direct WAN** | **SmartDNS / EU VPN** | **Direct WAN (ISP)** | **DNS Block (0.0.0.0)** |
| ⚙️ **"Custom: Fine-grained Matrix"** | *Custom* | *Custom* | *Custom* | *Custom* |

### Installation on OpenWrt 24.10

Packages are in [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/):
```bash
opkg update
opkg install openstream-engine_0.4.2-35_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-35_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-35_all.ipk
```

After installation, access **Services → OpenStream Engine** in LuCI.

---

## 🛣️ Roadmap

```text
[x] Phase 1: Specification 2.0, openstream-rule & openstream-core crates, Trie matcher, and reference rulesets.
[x] Phase 2: OpenWrt network adapter refactoring under the NetworkBackend trait (dnsmasq, nftables, LuCI).
[x] Phase 3: iOS Proof-of-Concept (Swift 6 + NetworkExtension PacketTunnelProvider via UniFFI).
[x] Phase 4: Public GitHub Community Rule Store with CI linting (osrule lint) and Ed25519 signatures.
[x] Phase 5: Android (VpnService + Kotlin Coroutines + NDK), macOS, and Windows backends (tun-rs).
```

---

## 🛡️ Security by Design

* **No MITM & No Root CA**: User devices require no custom root certificates and verify authentic TLS certificates directly.
* **SecOps System Domain Guard**: Core validation forbids intercepting critical system update and financial domains (`*.apple.com`, `windowsupdate.com`, banking resources) without explicit user opt-in.
* **Ed25519 Cryptographic Signatures**: Official and community rule packages are signed to prevent malicious route tampering.

---

## License

MIT © 2026 Denis Ershov
