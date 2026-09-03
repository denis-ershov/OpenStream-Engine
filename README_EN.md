# OpenStream Engine 🧪

<p align="center">
  <b>Universal Cross-Platform Traffic Orchestrator & Declarative Policy Routing Engine</b><br>
  <i>«One Rule. Every Platform. Zero Overhead.»</i>
</p>

<p align="center">
  <a href="https://github.com/denis-ershov/OpenStream-Engine/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/denis-ershov/OpenStream-Engine/ci.yml?branch=main&label=CI&logo=github" alt="CI Status"></a>
  <a href="https://github.com/denis-ershov/OpenStream-Engine/releases"><img src="https://img.shields.io/badge/release-v2.1.0--r35-blue.svg?logo=openwrt" alt="Release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust" alt="Rust 1.80+"></a>
  <a href="https://openwrt.org/"><img src="https://img.shields.io/badge/OpenWrt-24.10%20(aarch64)-0099ff.svg?logo=openwrt" alt="OpenWrt 24.10"></a>
</p>

<p align="center">
  <a href="#-about-the-project--research-status">About</a> •
  <a href="#-key-features">Features</a> •
  <a href="#-system-architecture">Architecture</a> •
  <a href="#-quick-start-openwrt">Installation</a> •
  <a href="#-platform-support">Platforms</a> •
  <a href="#-repository-structure">Structure</a> •
  <a href="#-security-by-design">Security</a> •
  <a href="#-integrations--open-source-ecosystem">Integrations</a> •
  <a href="README.md">Русский</a>
</p>

---

> [!NOTE]
> ### 🧪 Research Project Status (Experimental Beta)
> **OpenStream Engine** is an open research laboratory and high-performance cross-platform **declarative policy-based traffic routing engine**.  
> The project investigates methods for intelligent, fine-grained routing of streaming services (Twitch, YouTube, Discord, Crunchyroll, etc.), local DPI desynchronization (Anti-DPI), and selective proxying **without TLS decryption (No MITM) and without installing custom Root CA certificates on user devices**.

---

## 🌟 Philosophy 2.1: "One Rule. Every Platform. Zero Overhead."

Most existing networking utilities fall into two extremes: either monolithic VPN clients forcing all traffic through a single remote tunnel (introducing high latency and buffering), or fragmented OS-specific scripts.

**OpenStream Engine 2.1** introduces a unified, modern architecture:
* **Universal Rule Specification (`.osrule.yaml`)**: A single declarative service rule executes deterministically across **OpenWrt routers, iOS, Android, macOS, Windows, and Linux**.
* **Ultra-Lightweight Native Rust Core (`openstream-core`)**: Zero-allocation Reverse Suffix Trie domain matching ($O(k)$) and Longest Prefix Match CIDR lookup ($O(1)$). Memory footprint is **< 2 MB RAM** with zero Garbage Collector pauses, guaranteeing instantaneous response times and full compliance with Apple's strict iOS NetworkExtension Jetsam ceiling (15–50 MB).
* **Multi-Tier Egress Strategies**: Within the same rule manifest, traffic is split so that bandwidth-heavy video CDNs stay on direct ISP WAN while authentication and API requests route through clean relays or local desynchronizers.

---

## 🚀 Key Features

### 1. Intelligent Hybrid Routing
* ⏩ **"Bypass" Action (Direct WAN Exclusions)**: Priority `ip daddr @bypass_targets return` rule at the very top of `mangle_prerouting` immediately excludes whitelisted services (banking, government portals, workplace VPNs) before Zapret2 or VPN processing.
* 🚀 **Zapret2 (`nfqws2`) Integration**: Local TCP/UDP packet desynchronization (bypassing YouTube 4K throttling and Discord Voice blocks) via NFQUEUE 1088 at line speed. Supports built-in presets and arbitrary user flags (`custom_args`).
* 🌐 **sing-box Tunneling**: Directs blocked resources to encrypted outbounds attached to TPROXY `:10888`.
* 🛡️ **Local StreamProxy (:8888)**: In-flight HLS/DASH manifest cleanup removing Server-Side Ad Insertion (SSAI) without buffering.
* ⛔ **DNS Sinkhole**: Immediate blocking of tracking domains and ad servers at the DNS level (`0.0.0.0` / `::`).

### 2. Universal Subscription & Server Manager
* **Modern Protocol Support**: VLESS (Reality, xHTTP, Vision), Hysteria 2 / hy2 (QUIC UDP), TUIC v5 (BBR), Shadowsocks 2022, Trojan, VMess.
* **Subscription Parser**: Direct import from clipboard, HTTPS URLs, Base64 strings, and Clash / Mihomo YAML formats (`proxies:`).
* ⚡ **URLTest Latency Selector**: Automated background RTT measurement (`https://cp.cloudflare.com/generate_204`) dynamically directing traffic to the fastest server.

### 3. Resilient Multi-DNS Failover & Bootstrap DNS
* **Elimination of DNS Deadlocks**: Dedicated static Bootstrap DNS resolvers (`77.88.8.8`, `1.1.1.1`) operate directly via WAN (`detour: direct`), guaranteeing fast DoH startup.
* **Cascading Failover**: Automatic seamless fallback to secondary DoH/DoT resolvers if the primary DNS experiences timeout or degradation.

### 4. Network Security & nftables Protocol Hardening
* **QUIC Blocking (UDP 443)**: `udp dport 443 reject` forces browsers into fast TCP TLS 1.3, ensuring 100% effectiveness for Zapret2 on YouTube 4K.
* **Direct DoH Blocking (TCP 853)**: Prevents DNS leaks bypassing the router's dnsmasq ruleset.
* **NTP Protection (UDP 123)**: Direct bypass ensures pristine system clock synchronization.

### 5. Discrete Component Updates & Automated Cron Jobs
* **One-Click Discrete Updates**: Independently update individual components (OSE Core, LuCI, sing-box, zapret2, rule store).
* **4 sing-box Flavors**: Stable, Extended (xHTTP), Tiny (<8 MB Flash / <15 MB RAM), Extended Compress (UPX compression saving 65% Flash).
* **Cron Auto-Updates**: Background scheduled sync with SHA-256 verification and automatic rollback (Safe Fallback).

### 6. 1-Click Self-Diagnostics
* Comprehensive real-time health verification for nftables rules, dnsmasq integration, sing-box TPROXY sockets, Zapret2 queues, and DNS leak tests.

### 7. Modern LuCI Web UI (Mobile First)
* Strictly engineered according to **Mobile First** principles (no legacy HTML tables).
* Responsive OLED Dark aesthetic (`#020617`, `#0b1329`) featuring adaptive cards, live graphs, and micro-animations.

---

## 📐 System Architecture

```text
                     ┌──────────────────────────────────────────────┐
                     │    Declarative Manifests (*.osrule.yaml)     │
                     └──────────────────────┬───────────────────────┘
                                            │
                     ┌──────────────────────▼───────────────────────┐
                     │   openstream-core (Zero-Alloc Trie, <2 MB)   │
                     └──────┬───────────────┬───────────────┬───────┘
                            │               │               │
             ┌──────────────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
             │   OpenWrt Router    │ │   Desktop   │ │   Mobile    │
             │ (nftables + dnsmasq)│ │(Windows/Lin)│ │ (iOS/Android)│
             └──────┬──────────────┘ └──────┬──────┘ └──────┬──────┘
                    │                       │               │
     ┌──────────────┴────────┬──────────────┴────────┬──────┴────────┐
     ▼                       ▼                       ▼               ▼
[ ⏩ Bypass ]         [ 🚀 Zapret2 ]          [ 🌐 sing-box ]  [ ⛔ Block ]
 Direct WAN           NFQUEUE 1088            urltest Selector  0.0.0.0
 (Exclusions)         (DPI Desync/Custom)     (Multi-DNS/Hy2)  (Sinkhole)
```

---

## 📋 Declarative Rule Example (`.osrule.yaml`)

```yaml
schema_version: "2.1"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.1.0"

matches:
  - group: "auth_token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree_egress" # Route via ad-free region (UA/AL/KZ)

  - group: "master_playlist"
    domains: ["usher.ttvnw.net"]
    strategy: "quality_unlock" # SmartDNS / EU VPN for 1080p60/1440p

  - group: "video_cdn"
    domains: ["*.live-video.net", "*.ttvnw.net"]
    action: "direct" # Direct unthrottled ISP WAN

  - group: "tracker_ads"
    domains: ["edge.ads.twitch.tv"]
    action: "block" # DNS Sinkhole (0.0.0.0)

strategies:
  adfree_egress:
    preference: ["proxy:al_clean", "proxy:ua_clean", "direct"]
  quality_unlock:
    preference: ["smartdns:eu", "proxy:de_fast", "direct"]
```

---

## 📱 Platform Support

| Platform | Tech Stack | Interception Mechanism | Status |
|---|---|---|---|
| **OpenWrt 24.10 / 23.05** | Rust + ucode RPC + LuCI JS | nftables + dnsmasq + NFQUEUE | **Stable Release** (Prebuilt IPKs) |
| **Linux Desktop / Server** | Rust (`openstream-backend-desktop`) | TUN (`tun-rs`) / systemd | **Supported** |
| **Windows 10 / 11** | Rust + Wintun driver | TUN adapter | **Supported** |
| **Android 10+** | Kotlin + NDK + JNI + Compose M3 | Android `VpnService` | **Implemented** (`platforms/android`) |
| **Apple iOS 17+ / macOS** | Swift 6 Strict Concurrency + UniFFI | `NEPacketTunnelProvider` | **Implemented** (`platforms/ios`) |

---

## ⚡ Quick Start: OpenWrt Installation

Precompiled packages for **aarch64 (Cortex-A53)** are available in [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/):

```bash
# 1. Update package lists
opkg update

# 2. Install engine and LuCI web app
opkg install openstream-engine_0.4.2-35_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-35_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-35_all.ipk

# 3. Restart LuCI web server
/etc/init.d/rpcd restart
/etc/init.d/uhttpd restart
```

Navigate to **Services → OpenStream Engine** in the LuCI interface.

---

## 📂 Repository Structure

```text
├── crates/
│   ├── openstream-rule/          # AST parser, .osrule.yaml schema, Ed25519 signatures
│   ├── openstream-core/          # Zero-Alloc Reverse Suffix Trie, LPM IP tree
│   ├── openstream-backend-openwrt# nftables, dnsmasq, Zapret2, sing-box generators
│   ├── openstream-backend-desktop# Cross-platform desktop TUN network adapter
│   ├── openstream-ffi/           # UniFFI Swift bindings for iOS and macOS
│   ├── openstream-jni/           # NDK JNI bridge for Android
│   ├── ose-proxy/                # Local HTTP/HLS proxy engine
│   └── streamproxyd/             # Core CLI background daemon
├── luci-app-openstream/          # LuCI Web UI (ucode RPC daemon + OLED Dark JS views)
│   └── root/
│       ├── usr/share/rpcd/ucode/ # Server-side RPC plugin (openstream.uc)
│       └── www/luci-static/      # routing.js, servers.js, monitor.js, services.js, updates.js
├── platforms/
│   ├── android/                  # Android client (VpnService + Jetpack Compose M3)
│   └── ios/                      # iOS client (NetworkExtension + SwiftUI)
├── rules/                        # Catalog of service rule manifests (Twitch, YouTube, Crunchyroll)
├── dist/                         # Precompiled ready-to-flash IPK packages for OpenWrt
├── docs/                         # Architecture specifications and CHANGELOG
└── scripts/                      # IPK packaging and verification automation
```

---

## 🛡️ Security by Design

* **Zero MITM**: The engine deliberately avoids TLS interception and never requires installing custom root certificates on user devices.
* **SecOps Protected Domains**: The core actively prohibits intercepting critical infrastructure (`*.apple.com`, `windowsupdate.com`, banking/payment portals) without explicit administrative override.
* **Ed25519 Signatures**: Rule catalogs are cryptographically signed to prevent malicious route injections.

---

## 🤝 Contributing

Contributions are warmly welcome!
- See the [Contributing Guidelines](CONTRIBUTING.md).
- Review our [Security Policy](SECURITY.md).
- Read our [Code of Conduct](CODE_OF_CONDUCT.md).
- Version history is documented in [CHANGELOG.md](docs/CHANGELOG.md).

---

## 🔗 Integrations & Open Source Ecosystem

The OpenStream Engine architecture relies on deep orchestration and integration with battle-tested open source networking technologies:

* **[bol-van/zapret2](https://github.com/bol-van/zapret2)** by `@bol-van`:  
  The leading Layer 4 packet desynchronization engine. OpenStream natively orchestrates `nfqws2` via `NFQUEUE 1088` with built-in presets (YouTube 4K, Discord Voice) and arbitrary custom CLI arguments.
* **[SagerNet/sing-box](https://github.com/SagerNet/sing-box)** by `@SagerNet`:  
  Universal proxy platform powering OpenStream's encrypted egress (attached to TPROXY `:10888`). Provides cutting-edge protocols (VLESS Reality/xHTTP, Hysteria 2, TUIC v5, Shadowsocks 2022), `urltest` latency auto-selection, and multi-DNS routing.
* **[1andrevich/zapret2-openwrt](https://github.com/1andrevich/zapret2-openwrt)**:  
  OpenWrt package feeds and optimized builds for `nfqws2`.
* **[tun-rs](https://github.com/meh/rust-tun)**:  
  Cross-platform asynchronous TUN driver powering userspace packet capture on desktop platforms (Windows, Linux, macOS).
* **[mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs)**:  
  Zero-overhead multi-language bindings bridging the native Rust core to Swift 6 (iOS NetworkExtension) and Kotlin/JNI (Android).

---

## 📄 License

Distributed under the **[MIT License](LICENSE)** © 2026 Denis Ershov.

