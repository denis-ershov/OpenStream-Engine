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
  <a href="#-о-проекте-и-исследовательский-статус">О проекте</a> •
  <a href="#-ключевые-возможности">Возможности</a> •
  <a href="#-архитектура">Архитектура</a> •
  <a href="#-быстрый-старт-openwrt">Установка</a> •
  <a href="#-платформы">Платформы</a> •
  <a href="#-структура-репозитория">Структура</a> •
  <a href="#-безопасность">Безопасность</a> •
  <a href="#-интеграции-и-используемые-компоненты">Интеграции</a> •
  <a href="README_EN.md">English</a>
</p>

---

> [!NOTE]
> ### 🧪 Исследовательский статус (Experimental Beta / Research Project)
> **OpenStream Engine** — это открытый исследовательский проект и высокопроизводительный кроссплатформенный движок **декларативной маршрутизации сетевого трафика (Policy-Based Routing)**.  
> Проект исследует методы интеллектуальной гранулярной маршрутизации медиасервисов (Twitch, YouTube, Discord, Crunchyroll и др.), локальной десинхронизации DPI (Anti-DPI) и выборочного туннелирования **без расшифровки TLS (No MITM) и без установки сторонних корневых CA-сертификатов на клиентские устройства**.

---

## 🌟 Философия 2.1: «One Rule. Every Platform. Zero Overhead.»

Большинство существующих сетевых утилит делятся на две крайности: либо тяжелые VPN-клиенты, заворачивающие весь трафик в один туннель и создающие задержки, либо узкоспециализированные скрипты под конкретную операционную систему.

**OpenStream Engine 2.1** предлагает унифицированный подход:
* **Единый формат правил (`.osrule.yaml`)**: одно и то же правило сервиса детерминированно исполняется на **OpenWrt-роутерах, iOS, Android, macOS, Windows и Linux**.
* **Сверхлегкое нативное Rust-ядро (`openstream-core`)**: прямое сопоставление FQDN через Zero-Allocation Reverse Suffix Trie ($O(k)$) и поиск подсетей по Longest Prefix Match ($O(1)$). Потребление памяти **< 2 МБ RAM** без сборщика мусора (Zero GC), что гарантирует мгновенный отклик и совместимость с жестким лимитом Apple NetworkExtension Jetsam (15–50 МБ).
* **Многоуровневые стратегии выхода**: в рамках одного правила домен может направляться по оптимальному сетевому пути без перегрузки VPN-каналов.

---

## 🚀 Ключевые возможности

### 1. Интеллектуальная гибридная маршрутизация
* ⏩ **Действие «Bypass» (Прямое исключение в WAN)**: Приоритетное правило `ip daddr @bypass_targets return` в самом верху цепочки `mangle_prerouting` исключает выбранные сервисы (банки, Госуслуги, рабочая почта) из Zapret2 и VPN до их обработки.
* 🚀 **Интеграция с Zapret2 (`nfqws2`)**: Локальная десинхронизация TCP/UDP пакетов (обход замедления YouTube 4K, Discord Voice) через NFQUEUE 1088 без снижения скорости провайдера. Поддержка готовых пресетов и произвольных пользовательских флагов (`custom_args`).
* 🌐 **Туннелирование sing-box**: Направление заблокированных ресурсов в зашифрованные туннели с привязкой к TPROXY `:10888`.
* 🛡️ **Локальный StreamProxy (:8888)**: Очистка потоков HLS/DASH от серверной рекламы (SSAI) без буферизации.
* ⛔ **DNS Sinkhole**: Мгновенная блокировка трекеров и рекламных сетей на уровне DNS (`0.0.0.0` / `::`).

### 2. Универсальный менеджер подписок и серверов
* **Поддержка протоколов нового поколения**: VLESS (Reality, xHTTP, Vision), Hysteria 2 / hy2 (QUIC UDP), TUIC v5 (BBR), Shadowsocks 2022, Trojan, VMess.
* **Импорт подписок**: Прямой парсинг ссылок из буфера обмена, подписок по URL `https://...`, Base64-списков и конфигураций Clash / Mihomo YAML (`proxies:`).
* ⚡ **URLTest Latency Selector**: Автоматический фоновый замер задержки серверов (`https://cp.cloudflare.com/generate_204`) и автопереключение на самый быстрый узел.

### 3. Отказоустойчивый Multi-DNS Failover и Bootstrap DNS
* **Устранение DNS-дедлоков**: Выделенный пул статических Bootstrap-резолверов (`77.88.8.8`, `1.1.1.1`) работает напрямую через WAN (`detour: direct`), гарантируя мгновенный старт DoH-серверов.
* **Каскадный Failover**: Автоматическое переключение на резервные резолверы при сбоях или деградации основного DNS.

### 4. Сетевая безопасность и фильтрация в nftables
* **Блокировка QUIC (UDP 443)**: `udp dport 443 reject` — форсирует TCP TLS 1.3 в браузерах, обеспечивая 100% эффективность десинхронизации Zapret2 для YouTube 4K.
* **Блокировка прямого DoH (TCP 853)**: Предотвращает неконтролируемые утечки DNS в обход маршрутизации роутера.
* **Защита NTP (UDP 123)**: Прямой пропуск системного времени без искажений.

### 5. Раздельные обновления и фоновое автообновление (Cron)
* **Раздельные обновления**: Независимое обновление каждого компонента в 1 клик (ядро OSE, LuCI, sing-box, zapret2, каталог правил).
* **4 сборки sing-box**: Stable, Extended (xHTTP), Tiny (<8 МБ Flash / <15 МБ RAM), Extended Compress (UPX сжатие для экономии 65% Flash).
* **Cron Автообновление**: Фоновая синхронизация по расписанию с проверкой контрольных сумм SHA-256 и безопасным откатом (Safe Fallback).

### 6. Самодиагностика в 1 клик (Self-Diagnostics)
* Экспресс-проверка работоспособности nftables, dnsmasq, сокетов sing-box, очередей Zapret2 и отсутствия утечек DNS прямо в веб-интерфейсе.

### 7. Современный LuCI Web UI (Mobile First)
* Разработан строго по принципам **Mobile First** (без архаичных HTML-таблиц).
* Адаптивный дизайн OLED Dark (`#020617`, `#0b1329`) с карточным представлением, живыми графиками и микроанимациями.

---

## 📐 Архитектура системы

```text
                     ┌──────────────────────────────────────────────┐
                     │    Декларативные манифесты (*.osrule.yaml)   │
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
 (Исключения)         (DPI Desync/Custom)     (Multi-DNS/Hy2)  (Sinkhole)
```

---

## 📋 Пример декларативного правила (`.osrule.yaml`)

```yaml
schema_version: "2.1"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.1.0"

matches:
  - group: "auth_token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree_egress" # Выход через регион без рекламы (UA/AL/KZ)

  - group: "master_playlist"
    domains: ["usher.ttvnw.net"]
    strategy: "quality_unlock" # SmartDNS / EU VPN для доступа к 1080p60/1440p

  - group: "video_cdn"
    domains: ["*.live-video.net", "*.ttvnw.net"]
    action: "direct" # Прямой канал от провайдера на максимальной скорости

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

## 📱 Кроссплатформенная поддержка

| Платформа | Стек реализации | Механизм перехвата | Статус |
|---|---|---|---|
| **OpenWrt 24.10 / 23.05** | Rust + ucode RPC + LuCI JS | nftables + dnsmasq + NFQUEUE | **Стабильный релиз** (Готовые IPK) |
| **Linux Desktop / Server** | Rust (`openstream-backend-desktop`) | TUN (`tun-rs`) / systemd | **Поддерживается** |
| **Windows 10 / 11** | Rust + Wintun driver | TUN adapter | **Поддерживается** |
| **Android 10+** | Kotlin + NDK + JNI + Compose M3 | Android `VpnService` | **Реализовано** (`platforms/android`) |
| **Apple iOS 17+ / macOS** | Swift 6 Strict Concurrency + UniFFI | `NEPacketTunnelProvider` | **Реализовано** (`platforms/ios`) |

---

## ⚡ Быстрый старт: Установка на OpenWrt

Готовые пакеты для архитектуры **aarch64 (Cortex-A53)** доступны в каталоге [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/):

```bash
# 1. Обновите список пакетов роутера
opkg update

# 2. Установите ядро и веб-интерфейс
opkg install openstream-engine_0.4.2-35_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-35_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-35_all.ipk

# 3. Перезапустите веб-сервер LuCI
/etc/init.d/rpcd restart
/etc/init.d/uhttpd restart
```

После установки перейдите в веб-интерфейс: **Службы → OpenStream Engine**.

---

## 📂 Структура репозитория

```text
├── crates/
│   ├── openstream-rule/          # AST парсер, схема .osrule.yaml, Ed25519 подписи
│   ├── openstream-core/          # Zero-Alloc Reverse Suffix Trie, LPM IP-дерево
│   ├── openstream-backend-openwrt# Генераторы nftables, dnsmasq, Zapret2, sing-box
│   ├── openstream-backend-desktop# TUN сетевой адаптер для настольных ОС
│   ├── openstream-ffi/           # UniFFI Swift-биндинги для iOS и macOS
│   ├── openstream-jni/           # NDK JNI-мост для Android
│   ├── ose-proxy/                # Локальный HTTP/HLS прокси-движок
│   └── streamproxyd/             # Системный демон CLI
├── luci-app-openstream/          # LuCI Web UI (ucode RPC демон + LuCI JS OLED Dark)
│   └── root/
│       ├── usr/share/rpcd/ucode/ # Серверный RPC плагин (openstream.uc)
│       └── www/luci-static/      # routing.js, servers.js, monitor.js, services.js, updates.js
├── platforms/
│   ├── android/                  # Android приложение (VpnService + Jetpack Compose M3)
│   └── ios/                      # iOS приложение (NetworkExtension + SwiftUI)
├── rules/                        # Каталог декларативных правил (Twitch, YouTube, Crunchyroll)
├── dist/                         # Готовые скомпилированные IPK пакеты для OpenWrt
├── docs/                         # Полная архитектурная документация и CHANGELOG
└── scripts/                      # Скрипты упаковки IPK и верификации
```

---

## 🛡️ Безопасность (Security by Design)

* **Zero MITM**: Движок принципиально не расшифровывает TLS-трафик и не требует установки сторонних CA-сертификатов на клиентские устройства. Все SSL/TLS соединения проверяются напрямую конечными серверами.
* **SecOps защита системных зон**: На уровне ядра запрещен перехват критических зон (`*.apple.com`, `windowsupdate.com`, банковские и платежные домены) без явного подтверждения администратором.
* **Криптографические подписи**: Пакеты каталога правил подписываются ключами Ed25519 для защиты от подмены маршрутов.

---

## 🤝 Участие в разработке

Мы приветствуем вклад сообщества!
- Ознакомьтесь с [Руководством контрибьютора](CONTRIBUTING.md).
- Ознакомьтесь с [Политикой безопасности](SECURITY.md).
- Посетите [Кодекс поведения](CODE_OF_CONDUCT.md).
- История версий и изменений доступна в [CHANGELOG.md](docs/CHANGELOG.md).

---

## 🔗 Интеграции и используемые компоненты

Архитектура OpenStream Engine построена на оркестрации и глубокой интеграции проверенных решений с открытым исходным кодом:

* **[bol-van/zapret2](https://github.com/bol-van/zapret2)** (автор `@bol-van`):  
  Инструмент десинхронизации пакетов на уровне L4. В OpenStream интегрирован демон `nfqws2` через выделенную очередь `NFQUEUE 1088` со встроенными пресетами для YouTube 4K, Discord Voice и поддержкой произвольных пользовательских аргументов.
* **[SagerNet/sing-box](https://github.com/SagerNet/sing-box)** (проект `@SagerNet`):  
  Универсальная кроссплатформенная прокси-платформа. Используется в качестве исходящего туннельного движка (TPROXY `:10888`), обеспечивая поддержку протоколов VLESS Reality/xHTTP, Hysteria 2, TUIC v5, Shadowsocks 2022, а также селектора задержки `urltest` и отказоустойчивого Multi-DNS.
* **[1andrevich/zapret2-openwrt](https://github.com/1andrevich/zapret2-openwrt)**:  
  Официальный фид и сборки `nfqws2` для экосистемы OpenWrt.
* **[tun-rs](https://github.com/meh/rust-tun)**:  
  Кроссплатформенный асинхронный сетевой драйвер TUN для системного перехвата пакетов в настольных ОС (Windows, Linux, macOS).
* **[mozilla/uniffi-rs](https://github.com/mozilla/uniffi-rs)**:  
  Инструментарий генерации высокопроизводительных FFI-биндингов между Rust-ядром и нативными клиентскими стеками (Swift 6 Strict Concurrency для iOS и JNI/Kotlin для Android).

---

## 📄 Лицензия

Проект распространяется под свободной лицензией **[MIT](LICENSE)** © 2026 Denis Ershov.

