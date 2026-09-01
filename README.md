# OpenStream Engine 🧪 (Beta / Research Project)

[Read in English](README_EN.md)

> [!NOTE]
> **Исследовательский статус (Experimental Beta / Research Project):**  
> OpenStream Engine — это открытая исследовательская лаборатория и движок декларативной маршрутизации сетевого трафика. Проект исследует методы выборочной маршрутизации сервисов (Twitch, YouTube, Crunchyroll и др.), обход серверной рекламы (SSAI), разблокировку качества (1080p60 / 1440p / Source) и локальную десинхронизацию DPI (Anti-DPI) **без необходимости расшифровки TLS (No MITM) и без установки сторонних CA-сертификатов на клиентские устройства**.

---

## 🌟 Видение OpenStream 2.0: «One Rule. Every Platform. Zero Overhead.»

Большинство существующих сетевых утилит делятся на две крайности: либо тяжелые VPN-клиенты, перенаправляющие весь трафик в один туннель, либо узкоспециализированные скрипты под конкретную ОС.

**OpenStream Engine 2.0** — это универсальный кроссплатформенный движок **декларативных сервисных политик (Service-Based Policy Routing)**:
* **Единый формат правил (`.osrule.yaml`)**: одно и то же правило сервиса детерминированно исполняется на **OpenWrt, iOS, Android, macOS, Windows и Linux**.
* **Сверхлегкое Rust-ядро (`openstream-core`)**: прямое сопоставление FQDN через Zero-Allocation Reverse Suffix Trie ($O(k)$) и подсетей CIDR. Потребление памяти $< 2$ МБ RAM без сборщика мусора (Garbage Collector), что делает ядро идеальным как для бюджетных роутеров с 128 МБ RAM, так и для жесткой песочницы Apple iOS NetworkExtension (лимит 15–50 МБ).
* **Гибридные стратегии выхода**: в рамках одного правила один домен может идти в туннель (`Proxy`), второй — напрямую с локальным разделением ClientHello (`DpiEvasiveDirect`), третий — на полной скорости провайдера (`Direct`), а трекеры — в `Block` (0.0.0.0).

---

## 🔬 Анализ типовых решений индустрии

При проектировании архитектуры OpenStream 2.0 был проведен детальный аудит существующих открытых проектов:

| Проект | Стек | Сильные стороны | Архитектурные ограничения | Ответ в OpenStream 2.0 |
|---|---|---|---|---|
| **Podkop** | OpenWrt + `sing-box` (Go) | Удобный веб-интерфейс LuCI, поддержка geosite/geoip списков. | Бинарник Go 25–40 МБ, высокий оверхед RAM из-за сборщика мусора. Не запускается на роутерах с малым overlay и в iOS sandbox. | Берем удобный UX управления правилами, но переносим ядро на компактный нативный Rust (<2 МБ). |
| **Forkop** | OpenWrt + sing-box + Zapret + ByeDPI | **Гибридность**: сочетание VPN-туннелей и локальных Anti-DPI утилит. | Множество независимых процессов и bash-скриптов, риск конфликтов в nftables/iptables. | **Встроенная гибридность**: выбор между прокси, локальным Anti-DPI и прямым выходом управляется внутри единого ядра. |
| **Zapret / Zapret2** (`bol-van`) | C + NFQUEUE (`nfqws`) + `tpws` | Глубокие техники модификации пакетов (TCP segmentation, fake SNI, desync). | Требует root, NFQUEUE ядра Linux или WinDivert, сложнейшая конфигурация, не работает на iOS. | Выделение L4-десинхронизации в платформенную Capability для роутеров и базовая адаптация в userspace. |
| **SpoofDPI** (`xvzc`) / **ByeDPI** | Go / C (Android VpnService) | Локальное разделение `ClientHello` на границе SNI восстанавливает доступ к сервисам без зарубежных серверов. | Узкая специализация, отсутствие декларативной экосистемы правил и маршрутизации. | Встраивание алгоритма `DpiEvasiveDirect` прямо в ядро OpenStream для максимальной скорости (до 1 Гбит/с) без затрат на VPS. |
| **tun-rs** | Rust async Tokio TUN | Кроссплатформенный TUN с поддержкой Apple iOS (`AsyncDevice::from_fd`). | Требует строгой обработки дескрипторов. | Официальный фундамент для сетевого ввода-вывода мобильных адаптеров OpenStream 2.0. |

---

## 📋 Пример декларативного правила (`.osrule.yaml`)

```yaml
schema_version: "2.0"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.0.0"

matches:
  - group: "auth_token"
    domains: ["gql.twitch.tv"]
    strategy: "adfree_egress" # В регион без рекламы (UA/AL/KZ)

  - group: "master_playlist"
    domains: ["usher.ttvnw.net"]
    strategy: "quality_unlock" # В ЕС/SmartDNS для 1080p/1440p

  - group: "video_cdn"
    domains: ["*.live-video.net", "*.ttvnw.net"]
    action: "direct" # Прямой канал максимальной скорости от провайдера

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

## 🚀 Текущий статус и готовые сборки для OpenWrt (Релиз 0.4.2-35)

Для роутеров на базе OpenWrt 24.10 (архитектура **Cortex-A53 / aarch64**) доступен стабильный релиз с поддержкой LuCI Web UI.

### Актуальная матрица сценариев в LuCI

| Пресет в LuCI | Token (GQL) | Master (Usher) | Media & Segments (CDN) | Баннеры (Ads) |
|---|---|---|---|---|
| 🛡️ **«Geo-Split через Clean-Proxy»** *(Рекомендуется)* | **Ad-Free VPN (UA/AL/KZ)** | **SmartDNS / EU VPN** | **Direct WAN (Провайдер)** | **DNS Block (0.0.0.0)** |
| ⚡ **«Playlist Edge (Manifest Strip)»** | **Через streamproxyd** | **Через streamproxyd** | **Прямой CDN (Через Edge)** | **Вырезаются из HLS** |
| 🌍 **«Quality Unlock (1440p/Source)»** | **Direct WAN** | **SmartDNS / EU VPN** | **Direct WAN (Провайдер)** | **DNS Block (0.0.0.0)** |
| ⚙️ **«Пользовательский (Custom Matrix)»** | *Выбор* | *Выбор* | *Выбор* | *Выбор* |

### Установка на OpenWrt 24.10

Пакеты доступны в каталоге [`dist/openwrt-24.10-a53/ipk/`](dist/openwrt-24.10-a53/ipk/):
```bash
opkg update
opkg install openstream-engine_0.4.2-35_aarch64_cortex-a53.ipk
opkg install luci-app-openstream_0.4.2-35_all.ipk
opkg install luci-i18n-openstream-ru_0.4.2-35_all.ipk
```

После установки перейдите в веб-интерфейс: **Службы → OpenStream Engine**.

---

## 🛣️ Дорожная карта (Roadmap)

```text
[x] Фаза 1: Спецификация 2.0, крейты openstream-rule и openstream-core, Trie-матчер и эталонные правила.
[x] Фаза 2: Рефакторинг сетевого адаптера OpenWrt под трейт NetworkBackend (dnsmasq, nftables, LuCI).
[x] Фаза 3: iOS Proof-of-Concept (Swift 6 + NetworkExtension PacketTunnelProvider через UniFFI).
[x] Фаза 4: Публичный GitHub-каталог Community Rules с CI-валидатором (osrule lint) и Ed25519-подписями.
[x] Фаза 5: Адаптеры для Android (VpnService + Kotlin Coroutines + NDK), macOS и Windows (tun-rs).
```

---

## 🛡️ Безопасность (Security by Design)

* **Без MITM и без CA**: устройства не требуют установки сторонних сертификатов и доверяют только подлинным SSL-сертификатам сервисов.
* **SecOps-защита системных доменов**: движок на уровне ядра запрещает перехват критических зон (`*.apple.com`, `windowsupdate.com`, банковские ресурсы) без явного подтверждения пользователем.
* **Подписи Ed25519**: пакеты правил защищены криптографической подписью для предотвращения внедрения вредоносных маршрутов.

---

## Лицензия

MIT © 2026 Denis Ershov
