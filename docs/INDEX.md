# Документация OpenStream Engine 2.1

Универсальный кроссплатформенный оркестратор сетевого трафика и декларативных политик маршрутизации (**«One Rule. Every Platform. Zero Overhead.»**).

---

## 📚 Основная архитектурная документация (2.1)

| Документ | Назначение и содержание |
|---|---|
| [../README.md](../README.md) | Обзор возможностей, витрина 2.1, установка на роутеры и мобильные ОС |
| [ARCHITECTURE.md](ARCHITECTURE.md) | **Манифест архитектуры 2.1**: Смена парадигмы, кроссплатформенные слои, матрица действий |
| [POLICY_ROUTING_ARCHITECTURE.md](POLICY_ROUTING_ARCHITECTURE.md) | **Детальная спецификация маршрутизации**: nftables, dnsmasq, Bypass-сеты, Zapret2, sing-box 4 сборок, Multi-DNS Failover, подписки |
| [adr/0003-goal1-router-only-tls.md](adr/0003-goal1-router-only-tls.md) | Принцип «Zero MITM»: отказ от подмены сертификатов и сторонних корневых CA на клиентах |
| [CHANGELOG.md](CHANGELOG.md) | Хронологический журнал изменений и версий проекта |

---

## 🛠️ Платформенная экосистема

* **OpenWrt / Linux**: `openstream-backend-openwrt`, `luci-app-openstream` (ucode RPC, LuCI JS OLED Dark без таблиц).
* **Desktop (Windows, macOS, Linux)**: `openstream-backend-desktop`, `streamproxyd`.
* **Android**: `openstream-jni`, Jetpack Compose Material 3 UI, `OpenStreamVpnService`.
* **iOS**: `openstream-ffi`, Swift 6 Strict Concurrency, `PacketTunnelProvider`.

---

## 🔬 Исследовательские материалы и архив 1.0

[TWITCH_TRAFFIC_MAP](research/TWITCH_TRAFFIC_MAP.md) · [FORKOP_ISSUES_ANALYSIS](research/FORKOP_ISSUES_ANALYSIS.md) · [HLS](HLS_ARCHITECTURE.md) · [DASH](DASH_ARCHITECTURE.md) · [COEXISTENCE](COEXISTENCE.md) · [PACKAGING](PACKAGING.md)

