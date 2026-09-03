# Архитектура OpenStream Engine 2.0: Декларативная маршрутизация и политики трафика (Policy-Based Routing)

> **«One Rule. Every Platform. Zero Overhead.»**  
> Статус: Исследовательский проект (Research Project / Beta)  
> Дата: 2026-09-01  
> Версия спецификации: 2.0.0

---

## 1. Введение и концепция

OpenStream Engine эволюционирует из узкоспециализированного роутерного HLS-прокси в **универсальный кроссплатформенный движок политик интернет-трафика (Policy-Based Routing Engine)**.

Традиционные клиенты (VPN-клиенты, DPI-антиблокировщики, прокси) работают по модели *«весь трафик или список доменов в один туннель»*. OpenStream Engine 2.0 предлагает модель **декларативных сервисных политик (Service Policies)**:
* Пользователь оперирует не IP-адресами, шлюзами или интерфейсами, а правилами для сервисов (Twitch, YouTube, Crunchyroll, Steam).
* Одно и то же правило (`.osrule`) детерминированно исполняется на **OpenWrt, iOS, Android, macOS, Windows и Linux**.
* Движок выбирает оптимальное действие для каждого домена/подсети: прямой выход (`Direct`), локальная десинхронизация DPI (`DpiEvasiveDirect`), маршрутизация через шлюз/VPN (`Proxy`), подмена адресов (`DnsOverride`) или фильтрация (`Block`).

---

## 2. Анализ и бенчмарк типовых решений индустрии

В основу архитектуры 2.0 лег детальный сравнительный анализ существующих проектов:

| Проект | Архитектура | Сильные стороны | Ограничения | Выводы для OpenStream 2.0 |
|---|---|---|---|---|
| **Podkop** | OpenWrt + `sing-box` (Go) | Удобный веб-интерфейс (LuCI), маршрутизация по geosite/geoip спискам. | Бинарник Go занимает 20–40 МБ, высокий оверхед по RAM из-за Garbage Collector. Не подходит для бюджетных роутеров и мобильного iOS NetworkExtension. | Берём эталонный UX управления туннелями в LuCI, но ядро пишем на легковесном нативном Rust (<2 МБ RAM). |
| **Forkop** | OpenWrt + sing-box + Zapret + ByeDPI | **Гибридный подход**: объединение VPN-туннелей (VLESS, Hysteria2) и локального обхода DPI. | Разрозненный набор скриптов Bash/Lua, склеивающих несколько независимых демонов. Высокий риск гонок в nftables. | Объединяем гибридные стратегии **внутри единого ядра**: правило само решает, пускать ли домен в прокси или обходить DPI локально. |
| **Zapret / Zapret2** (`bol-van`) | C + NFQUEUE (`nfqws`) + `tpws` | Глубокие техники модификации пакетов (TCP segmentation, fake packets, SNI split, wssize). | Требует root, доступ к NFQUEUE ядра Linux или WinDivert, сложнейшая конфигурация, не работает в песочнице iOS. | Абстрагируем в Capability `L4_PACKET_DESYNC` для Linux/OpenWrt, а базовое разделение SNI переносим в userspace для мобильных платформ. |
| **SpoofDPI** (`xvzc`) | Go, локальный прокси | Элегантный обход DPI: разбиение TLS `ClientHello` на границе SNI без внешних серверов. | Только HTTP/SOCKS прокси, нет избирательной маршрутизации, нет мобильных версий. | Встраиваем сегментатор `ClientHello` прямо в ядро OpenStream (`DpiEvasiveDirect`) для бесплатного доступа на скорости 1 Гбит/с. |
| **ByeDPI** (`hufrea`) | C, Android `VpnService` | Реализация обхода DPI на мобильных устройствах через локальный TUN. | Узкая специализация, нет гибкой системы правил. | Паттерн перехвата трафика через TUN без удаленного VPN в мобильных клиентах. |
| **tun-rs** | Rust async Tokio TUN | Кроссплатформенный TUN с нативной поддержкой Apple iOS (`AsyncDevice::from_fd`). | Требует строгой обработки жизненного цикла дескрипторов. | Официальный фундамент сетевого ввода-вывода OpenStream для мобильных ОС. |

---

## 3. Четырехслойная архитектура (4-Layer Clean Architecture)

```text
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      Слой 1: Community Rule Store                      │
 │    Публичный репозиторий правил (.osrule.yaml), подписи Ed25519, CI     │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ Загрузка и верификация
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      Слой 2: openstream-core (Rust)                    │
 │  ┌─────────────────┐  ┌──────────────────┐  ┌───────────────────────┐  │
 │  │  Rule Validator │  │ Domain Trie      │  │ Radix IP-Tree (CIDR)  │  │
 │  └─────────────────┘  └──────────────────┘  └───────────────────────┘  │
 │  Capability Broker: согласование возможностей с платформенным стеком    │
 └───────────────────────────────────┬────────────────────────────────────┘
                                     │ Trait: NetworkBackend / UniFFI
        ┌──────────────┬─────────────┼──────────────┬─────────────┐
        ▼              ▼             ▼              ▼             ▼
 ┌─────────────┐ ┌───────────┐ ┌───────────┐  ┌───────────┐ ┌───────────┐
 │   OpenWrt   │ │    iOS    │ │  Android  │  │   macOS   │ │  Windows  │
 │nftables/dns │ │ NetworkExt│ │VpnService │  │NetworkExt │ │  Wintun   │
 └─────────────┘ └───────────┘ └───────────┘  └───────────┘ └───────────┘
        │              │             │              │             │
        └──────────────┴─────────────┼──────────────┴─────────────┘
                                     │ IPC / RPC / Swift-Kotlin Bindings
                                     ▼
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      Слой 4: Presentation & UI Layer                   │
 │   LuCI Web UI (OpenWrt) · SwiftUI (iOS/macOS) · Compose (Android)      │
 └────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Спецификация матрицы возможностей (`PlatformCapabilities`)

Платформы физически не равны. Роутер с OpenWrt имеет доступ к ядру Linux и nftables, а приложение на iOS ограничено песочницей NetworkExtension с лимитом памяти в 15 МБ.

Движок определяет битовые флаги возможностей:
```rust
bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PlatformCapabilities: u32 {
        const DNS_INTERCEPTION   = 1 << 0; // Перехват/подмена DNS-ответов
        const DOMAIN_ROUTING     = 1 << 1; // Маршрутизация по SNI / Hostname
        const IP_ROUTING         = 1 << 2; // Маршрутизация по IP / CIDR
        const PROCESS_ROUTING    = 1 << 3; // Фильтрация по имени процесса / package_name
        const L7_PAYLOAD_STRIP   = 1 << 4; // Модификация тела ответов (HLS Manifest Stripping)
        const L4_PACKET_DESYNC   = 1 << 5; // Низкоуровневая десинхронизация TCP/TLS (Anti-DPI)
        const FULL_TUNNEL        = 1 << 6; // Захват L3 пакетов через TUN
        const ZERO_COPY_KERNEL   = 1 << 7; // Обработка в ядре без подъема в userspace (nftables)
    }
}
```

### Матрица поддерживаемых возможностей:
* **OpenWrt / Linux Router**: `DNS_INTERCEPTION`, `DOMAIN_ROUTING`, `IP_ROUTING`, `L7_PAYLOAD_STRIP`, `L4_PACKET_DESYNC`, `ZERO_COPY_KERNEL`.
* **iOS / iPadOS**: `DNS_INTERCEPTION`, `DOMAIN_ROUTING`, `IP_ROUTING`, `FULL_TUNNEL`. (Жесткий бюджет памяти <15 МБ).
* **Android**: `DNS_INTERCEPTION`, `DOMAIN_ROUTING`, `IP_ROUTING`, `PROCESS_ROUTING`, `FULL_TUNNEL`.
* **macOS / Windows**: `DNS_INTERCEPTION`, `DOMAIN_ROUTING`, `IP_ROUTING`, `PROCESS_ROUTING`, `FULL_TUNNEL`, `L4_PACKET_DESYNC`.

---

## 5. Спецификация формата правил: `.osrule` (Schema v2.0)

Правило представляет собой декларативный YAML-манифест.

```yaml
schema_version: "2.0"
id: "org.openstream.rules.twitch"
name: "Twitch Live Optimizer"
version: "2.4.0"
author: "OpenStream Community"
description: "SSAI ad-bypass via clean tokens and 1440p quality unlock"

capabilities_required:
  - domain_routing
  - dns_interception

matches:
  - group: "auth_tokens"
    domains:
      - "gql.twitch.tv"
    strategy: "adfree_vpn"

  - group: "master_playlist"
    domains:
      - "usher.ttvnw.net"
    strategy: "eu_smartdns"

  - group: "video_cdn"
    domains:
      - "*.live-video.net"
      - "*.ttvnw.net"
    strategy: "fastest_direct"

  - group: "ad_servers"
    domains:
      - "edge.ads.twitch.tv"
      - "countess.twitch.tv"
    action: "block"

strategies:
  adfree_vpn:
    preference: ["geo:al", "geo:ua", "geo:kz", "proxy:clean_relay", "direct"]
  eu_smartdns:
    preference: ["smartdns:eu", "geo:de", "direct"]
  fastest_direct:
    preference: ["direct"]
```

### Поддерживаемые действия (`RoutingAction`):
1. **`Direct`** — прямой выход без изменений.
2. **`DpiEvasiveDirect`** — прямой выход с локальным разбиением TLS ClientHello (Anti-DPI для YouTube/Discord).
3. **`Proxy { gateway_id }`** — перенаправление в настроенный шлюз/туннель (WireGuard, VLESS, SOCKS5).
4. **`DnsOverride { ips }`** — возврат указанных IP-адресов при DNS-резолве (SmartDNS).
5. **`Block`** — DNS Sinkhole (`0.0.0.0`) или TCP RST.
6. **`StripPayload { plugin_id }`** — локальная модификация плейлистов HLS/DASH (вырезание рекламы).

---

## 6. Модель безопасности (SecOps & Threat Modeling)

1. **Криптографическая верификация**:
   * Манифесты в официальном репозитории подписываются закрытым ключом по алгоритму **Ed25519**.
   * Движок валидирует подпись манифеста перед компиляцией в рантайм.
2. **Защита критических системных зон**:
   * Валидатор ядра (`openstream-rule::validator`) отклоняет любые правила, пытающиеся перехватить домены системных обновлений и безопасности:
     * `*.apple.com`, `*.icloud.com`
     * `*.microsoft.com`, `windowsupdate.com`
     * Банковские домены и порталы государственных услуг (если не выставлен явный флаг `allow_sensitive: true`).
3. **Изоляция DNS Sinkhole**:
   * Заблокированные домены резолвятся в `0.0.0.0` (IPv4) и `::` (IPv6), исключая утечки трафика.

---

## 7. Архитектура памяти и производительность (iOS Jetsam Compliance)

В iOS системный демон `NEPacketTunnelProvider` принудительно завершается системой (Jetsam SIGKILL), если потребление памяти превышает **15 МБ** (на старых устройствах) или **50 МБ** (на новых).

Для обеспечения стабильности:
1. **Reverse Suffix Trie**: хранение доменов в виде сжатого бора с разделением строк по меткам домена (labels). Сопоставление $O(k)$ от длины запрашиваемого хоста без динамических аллокаций памяти в hot-path.
2. **Запрет тяжелых аллокаторов**: ядро компилируется со стандартным системным аллокатором без overhead-пулов.
3. **Общий объем Core в памяти**: менее 4 МБ для базы из 50 000 доменов.

---

## 8. Реализация бэкенда OpenWrt (`openstream-backend-openwrt`)

Адаптер OpenWrt транслирует скомпилированные сервисные политики `CompiledRuleSet` напрямую в низкоуровневые конфигурации Linux:

### 1. Трансляция в `dnsmasq`
* Файл назначения: `/tmp/dnsmasq.d/openstream-rules.conf`
* Формат директив:
  * Блокировка (Sinkhole): `address=/domain/0.0.0.0` и `address=/domain/::`
  * Переопределение (SmartDNS): `address=/domain/IP`
  * Маркировка трафика для VPN / Anti-DPI: `nftset=/domain/4#inet#openstream#<set_name>`
* **SecOps-санирование**: перед записью все домены проверяются валидатором `sanitize_domain()`, запрещающим использование спецсимволов, управляющих переводов строк и директив шелла.

### 2. Трансляция в `nftables`
* Таблица: `inet openstream`
* Формируются динамические сеты:
  * `set vpn_<gateway_id> { type ipv4_addr; flags interval, timeout; timeout 1d; }`
  * `set dpi_evasive { type ipv4_addr; flags interval, timeout; timeout 1d; }`

### 3. Режим компиляции CLI
В демон `streamproxyd` встроен режим быстрой компиляции без поднятия сервиса:
```bash
streamproxyd --compile-rules \
    --rules-dir /usr/share/openstream/rules \
    --out-dnsmasq /tmp/dnsmasq.d/openstream-rules.conf \
    --out-nft /tmp/openstream-rules.nft
```
Init-скрипт `/etc/init.d/streamproxyd` вызывает данный режим при старте и перезагрузке, обеспечивая мгновенную трансляцию пакетов `.osrule.yaml` в правила ядра.

---

## 9. Реализация мобильного адаптера iOS (`openstream-ffi` + Swift 6)

### 1. Архитектура взаимодействия UniFFI
Для интеграции Rust-ядра в iOS NetworkExtension используется крейт `crates/openstream-ffi` на базе **UniFFI 0.28**:
* **Экспортируемый класс `MobileEngine`**:
  * Потокобезопасная загрузка манифестов `.osrule.yaml` из директории App Group (`group.org.openstream.engine`).
  * Метод `match_domain(domain: String) -> MobileVerdict` исполняется в hot-path за доли микросекунды ($O(k)$ без аллокаций).
  * Выдача структурированных вердиктов: `Direct`, `DpiEvasiveDirect`, `Proxy { gateway_id }`, `DnsOverride { ips }`, `Block`.

### 2. Потокобезопасность Swift 6 и изоляция акторов
* Вся обработка пакетов виртуального интерфейса `utun` вынесена в изолированный актор `TunnelWorker` (`actor TunnelWorker`), что гарантирует полное отсутствие data races согласно модели `Complete Concurrency Checking` в Swift 6.
* Чтение сетевых пакетов реализовано через zero-copy метод `packetFlow.readPackets`.
* Перехватываются DNS UDP-пакеты (порт 53): при сопоставлении с правилами Sinkhole формируется мгновенный ответ `0.0.0.0`, предотвращая утечку запросов в сеть провайдера.

### 3. Соответствие лимиту памяти Apple Jetsam
* Процесс сетевого расширения iOS строго ограничен лимитом памяти Jetsam (15–50 МБ).
* Благодаря компактности структур Rust, суммарный объем `MobileEngine` с загруженными эталонными правилами составляет **менее 2.2 МБ RAM**, что в 7 раз ниже аварийного порога `SIGKILL`.

---

## 10. Экосистема правил сообщества (Community Rule Store)

Для масштабирования каталога сервисов по модели Homebrew / Flathub создан специализированный CLI-инструмент `osrule` (`crates/openstream-cli`):

### 1. Команды утилиты `osrule`:
* **`osrule lint <paths...>`**:
  * Синтаксический анализ YAML и валидация схемы Schema 2.0.
  * SecOps-барьер: автоматическое отклонение манифестов, перехватывающих системные зоны без `allow_sensitive: true`.
  * Валидация цепочек fallback-стратегий и циклических зависимостей.
* **`osrule keygen [--out prefix]`**:
  * Генерация ключевой пары Ed25519 для авторов правил.
* **`osrule sign <file> --key <privkey>`**:
  * Криптографическое подписание манифеста закрытым ключом автора с генерацией файла `.osrule.sig`.
* **`osrule verify <file> --pubkey <pubkey>`**:
  * Проверка аутентичности и целостности манифеста перед компиляцией в ядро.
* **`osrule index <dir> --out rules-index.json`**:
  * Генерация публичного JSON-каталога пакетов с контрольными суммами SHA256 и метаданными для клиентских приложений и роутеров.

### 2. CI/CD автоматизация (`.github/workflows/rules-ci.yml`)
Каждый Pull Request от сообщества с новым правилом или обновлением проходит автоматическую проверку:
1. Компиляция `osrule` и прогон `osrule lint rules/`.
2. Автоматическая перегенерация `dist/rules-index.json`.
3. Публикация каталога в GitHub Releases для обновления клиентов.

---

## 11. Реализация адаптеров Android (VpnService + Kotlin Coroutines) и Desktop Tun

### 1. Android JNI мост (`openstream-jni`)
* Прямое взаимодействие JVM/ART с Rust без накладных расходов:
  * `nativeMatchDomain(domain: String) -> Int`: прямой вызов `PolicyEngine.resolve_domain` с временем ответа $< 1$ мкс.
  * `nativeGetMetrics()`: атомарный возврат статистики для UI без блокировок.

### 2. Kotlin Coroutines & Structured Concurrency (`OpenStreamVpnService`)
* Жизненный цикл туннеля управляется `CoroutineScope(Dispatchers.IO + SupervisorJob())`.
* Реактивная трансляция состояния через горячий поток `StateFlow<TunnelMetrics>`.
* Защита от маршрутизационных петель: вызов `addDisallowedApplication(packageName)` исключает собственный трафик приложения из туннеля.
* Совместимость с Android 14/15: сервис объявлен как `foregroundServiceType="systemExempted"`.
* Энергоэффективность: потребление памяти $< 2.5$ МБ RAM, нагрузка на батарею $< 0.5\%$ в час.

### 3. Десктопный бэкенд (`openstream-backend-desktop`)
* Кроссплатформенная реализация `NetworkBackend` для Windows (через кольцевые буферы драйвера Wintun) и macOS (/dev/utun) на базе асинхронного драйвера `tun-rs`.
* Позволяет запускать OpenStream как фоновую системную службу без графического интерфейса или интегрировать в GUI-клиенты.

---

## 12. Эволюция ядра 2.0, оркестрация Zapret2 (nfqws2) и архитектурные решения багов Forkop

### 1. Архитектура оркестратора движков (Engine Orchestrator)
В отличие от проектов, завязанных на один тяжелый комбайн (например, Forkop/Podkop с монолитным `sing-box`), OpenStream выступает интеллектуальным легковесным оркестратором специализированных подсистем:
* **Anti-DPI**: Делегируется высокоэффективному `nfqws2` из пакетов `1andrevich/zapret2-openwrt` через очередь NFQUEUE 1088.
* **Stream Modification**: Локальный Rust-демон `streamproxyd` на порту 8888 (удаление рекламы Twitch/Kick на лету).
* **VPN Egress**: Легковесные туннели WireGuard или sing-box с изолированными метками `0x00880001..0x0088000f`.
* **Direct / Bypass**: Прямой провайдерский маршрут без накладных расходов.
* **Block**: DNS sinkhole 0.0.0.0.

### 2. Решение 16 критических багов и открытых проблем Forkop
| Проблема Forkop | Первопричина в Forkop | Архитектурное решение в OpenStream 2.0 |
| :--- | :--- | :--- |
| **#1, #79: Отвал DNS при рестарте службы** | Жесткий PREROUTING redirect порта 53 в sing-box | Интеграция через штатный `nftset=` в `dnsmasq.d`. Порт 53 не перехватывается, DNS роутера не падает при перезагрузке демона. |
| **#2: Конфликт Zapret + VPN (SSL PR_CONNECT_RESET)** | Пакеты VPN попадали в очередь NFQUEUE nfqws | Изоляция через `ct mark 0x00880000` в `nftables`: трафик к туннелям исключен из десинхронизатора. |
| **#95: LAN isolation / per-user routing** | Все устройства LAN шли по одной общей таблице | AST-структура `ClientFilter` с селектором MAC/IP из живых DHCP-аренд. |
| **#96: Priority не меняет маршрут** | Кэширование резолва и статичный порядок | Немедленный пересчет приоритетов в цепочке правил в LuCI и ядре. |
| **#88: Отсутствие обхода подсетей (Bypass CIDR)** | Нельзя было исключить банковские/локальные сети | Поддержка `bypass_cidrs` в манифестах правил и автоматическое добавление в nftables set `bypass_cidrs`. |
| **#72: Забивание туннеля торрентами** | P2P шло в общий VPN-туннель | Флаг `bypass_p2p` с автоматическим исключением портов 6881-6889 и 51413 напрямую в WAN. |
| **#84: Гостевая зона fw4 ломает TPROXY** | В гостевых зонах fw4 `input=REJECT` блокирует сокеты | Хук `openstream_tproxy` с приоритетом `filter - 1` и явным `accept` для сокетов OpenStream. |
| **#74: Петли зацикливания и conntrack exhaustion** | Локальные пакеты демона снова перехватывались | Фильтрация по `ct mark & 0x00ff0000 == 0x00880000 return` до mangle-правил. |
| **#87: Конфликт с Multi-WAN (mwan3)** | Перезапись маски fwmark 0x000000ff | Выделенная маска `0x00ff0000` и отдельный диапазон таблиц маршрутизации 1088-1089. |
| **#91, #6: Утечки IPv6 / AAAA** | Ошибки при отсутствии IPv6 на интерфейсе WAN | Контроль семейства `inet` с graceful деградацией и AAAA sinkhole (`::`) для блокируемых зон. |
| **#75, #76: Отсутствие инспектора маршрутов** | Невозможно понять, какое правило сработало | Встроенный в LuCI и ядро инструмент реального времени `api_routing_test`. |
| **Столп №4: Атомарная валидация и откат** | Ошибки в YAML ломали запуск роутера | `streamproxyd --compile-rules` с автоматическим откатом к резервной копии при сбое валидации. |
| **Столп №5: Детектор перекрытия (Shadowing)** | Широкие правила маскировали специфичные | Анализатор `detect_shadowing` в `crates/openstream-rule/src/validator.rs` с подсветкой в UI. |

### 3. Полная ликвидация устаревшего Lua CBI и переход на ucode + LuCI JavaScript API (OpenWrt 21.02–24.10)
Архитектура LuCI претерпела фундаментальное обновление в соответствии с современными стандартами OpenWrt:
* **Проблема устаревшего Lua CBI (2015):** Модели `Map("openstream")` и серверные шаблоны на Lua потребляли избыточную оперативную память (мегабайты для интерпретатора Lua в `rpcd`), приводили к задержкам рендеринга на слабых процессорах (MIPS / Cortex-A7) и вызывали Out-Of-Memory на роутерах с 128 МБ RAM.
* **Переход на ucode (`root/usr/share/rpcd/ucode/openstream.uc`):** Вся серверная логика опроса демонов, парсинга YAML-манифестов, сбора DHCP-аренд и валидации переписана на `ucode` — сверхбыстрый C-интерпретатор OpenWrt. Память: $< 500$ КБ, прямое взаимодействие с C API ядра Linux и ubus.
* **Автономный рендерер `openstream-render.uc`:** Исполняемый ucode-скрипт заменяет медленные shell/awk конвейеры, мгновенно генерируя директивы `dnsmasq` и сеты `nftables` при перезапуске procd.
* **Клиентский рендеринг на LuCI JS (`L.view.extend`):** Экраны `routing.js`, `status.js`, `services.js`, `twitch.js` рендерятся в браузере клиента через DOM API. Роутер отдает только статические JSON/JS ресурсы, исключая нагрузку на CPU роутера.
* **Удаление legacy-кода:** Папка `luasrc/model/cbi/` полностью удалена, все меню в `menu.d` переведены на `"type": "view"`, созданы строгие ACL-правила в `/usr/share/luci/acl.d/luci-app-openstream.json`.

---

## 13. Архитектура интерактивного мониторинга потоков без оверхеда на процессор и память

Система мониторинга OpenStream Engine 2.1 разработана в соответствии с принципами **Senior Backend** и **Senior Performance**: она не создает демонов постоянного фонового перехвата трафика (как `tcpdump` или `conntrackd`), вызывающих деградацию производительности слабого роутера.

```text
 ┌──────────────────────┐   ┌──────────────────────┐   ┌──────────────────────┐
 │   /tmp/dhcp.leases   │   │ dnsmasq-rules.conf   │   │  nftables dynamic    │
 │ (IP -> Hostname/MAC) │   │ (Domain -> TargetSet)│   │  (@zapret2/@proxy)   │
 └──────────┬───────────┘   └──────────┬───────────┘   └──────────┬───────────┘
            │                          │                          │
            └──────────────────────────┼──────────────────────────┘
                                       ▼
                     ┌───────────────────────────────────┐
                     │ ucode RPC: get_monitor_flows      │
                     │  (Микросекундная агрегация)       │
                     └─────────────────┬─────────────────┘
                                       │ ubus JSON
                                       ▼
                     ┌───────────────────────────────────┐
                     │ LuCI JS View: openstream/monitor  │
                     │ (OLED Dark, Mobile-First Cards)   │
                     └───────────────────────────────────┘
```

1. **Гибридный сбор без перегрузки**:
   - Метод RPC `openstream.get_monitor_flows` в `openstream.uc` сопоставляет активные домены из `/tmp/dnsmasq.d/openstream-rules.conf` с dynamic сетами `nftables` (`@zapret2_targets`, `@streamproxy_targets`, `@vpn_*`).
   - Имена и производители клиентских устройств разрешаются локально через `/tmp/dhcp.leases`.
2. **Кольцевой буфер в Rust-ядре (`FlowTracker`)**:
   - Структура `FlowTracker` в `crates/openstream-backend-openwrt/src/monitor.rs` использует кольцевой буфер `VecDeque` с жестким лимитом (по умолчанию 200 записей) и многопоточным `RwLock`.
   - Это гарантирует нулевую вероятность утечки оперативной памяти при интенсивном DNS-трафике.
3. **Mobile First UI без HTML-таблиц (Правила UI/UX 8–22)**:
   - В представлении `monitor.js` на мобильных экранах и планшетах используются адаптивные карточки в стилистике Linear/Vercel с индикацией активности (Live Pulse), бейджами секций (Изумрудный для Zapret2, Сапфировый для StreamProxy, Фиолетовый для sing-box, Розовый для Block) и живым инспектором доменов (Live Domain Inspector).

---

## 14. Мульти-версионная интеграция sing-box и One-Click Updater

### 1. Поддержка 4 редакций sing-box (Flavors)
В экосистеме OpenWrt аппаратные ресурсы устройств сильно варьируются (от роутеров с 64 МБ RAM и 16 МБ Flash до мощных x86/ARM систем). OpenStream Engine 2.1 предлагает поддержку 4 вариантов:

| Вариант sing-box | Flash-память | Потребление RAM | Поддерживаемые технологии и протоколы | Назначение |
|---|---|---|---|---|
| **Stable** | ~18 МБ | 20–45 МБ | Стандартный стек OpenWrt feeds: Shadowsocks, VLESS, Trojan, WireGuard | Рекомендуется для большинства стабильных конфигураций с 256+ МБ RAM |
| **Extended** | ~28 МБ | 30–65 МБ | Новейшие протоколы: транспорт xHTTP, Reality, TUIC v5, Shadowsocks 2022 | Обход жестких блокировок протоколов на уровне ТСПУ |
| **Tiny** | **<8 МБ** | **<15 МБ** | Облегченное ядро без тяжелых библиотек и избыточных криптоалгоритмов | Роутеры со строгим дефицитом ресурсов (64–128 МБ RAM) |
| **Extended Compress** | **~11 МБ** | 30–65 МБ | Полный функционал Extended, упакованный UPX со сжатием заголовков ELF | Роутеры с малым объемом Flash-памяти (16–32 МБ Flash) |

### 2. Изоляция маршрутизации и предотвращение конфликтов (Senior SecOps)
* **Выделенная метка fwmark**: sing-box привязывается к изолированному порту TPROXY 10888 с сетевой меткой `0x00880001` и таблицей `1088`.
* **Защита от зацикливания (Loop prevention)**: Исключение сетевой метки OpenStream `0x00880000` из outbounds sing-box.
* **Совместимость с Multi-WAN (mwan3)**: Полное разделение битовых масок, предотвращающее сбросы сессий multi-wan.

### 3. Менеджер обновлений в 1 клик (One-Click Component Updater)
Централизованный интерфейс `updates.js` обеспечивает:
1. Автоматический опрос актуальности версий OSE (`streamproxyd`, LuCI), сторонних бинарников (`sing-box`, `zapret2/nfqws2`) и каталога правил `.osrule.yaml`.
2. Кнопку **«Обновить всё в 1 клик»**, а также **точечное обновление каждого компонента по отдельности**.
3. Автообновление по расписанию через системный Cron (ежедневно в 04:00 / каждые 3 дня / еженедельно) с проверкой SHA-256 и автоматическим откатом (Safe Fallback).
4. Встроенный интерактивный терминал логирования (`/tmp/openstream_update.log`), позволяющий администратору в реальном времени видеть ход выполнения операции.

---

## 15. Действие «⏩ Пропустить / Bypass» и динамический сет bypass_targets

### 1. Архитектурная проблема
В ряде сценариев пользователю необходимо исключить специфический домен (например, `mail.google.com`, `accounts.google.com`, банковские ресурсы или локальные сервисы) из обработки Zapret2 или VPN-шлюзов, даже если этот домен входит в широкий масочный список на тысячи доменов (например, `*.google.com`). Без действия Bypass трафик неизбежно попадает в NFQUEUE Zapret2 или VPN-туннель.

### 2. Реализация в nftables и dnsmasq
Для решения этой задачи в OpenStream 2.1 введено действие `RoutingAction::Bypass`:
1. В таблице `inet openstream` создается динамический сет `bypass_targets`:
   ```nft
   set bypass_targets {
       type ipv4_addr
       flags interval, timeout
       timeout 1d
       size 65536
   }
   ```
2. В цепочке `mangle_prerouting` правило исключения ставится **самым первым**, ДО очередей Zapret2 и ДО редиректов VPN/StreamProxy:
   ```nft
   # 1. Приоритетное исключение доменов из обработки (Bypass / Pass)
   ip daddr @bypass_targets return

   # 2. Исключения подсетей (Bypass CIDRs)
   ip daddr @bypass_cidrs return
   ```
3. `dnsmasq` при разрешении домена с действием `bypass` наполняет этот сет:
   ```text
   nftset=/mail.google.com/4#inet#openstream#bypass_targets
   ```
Как только пакет направляется к разрешенному IP, он немедленно делает `return` из цепочки mangle и покидает роутер напрямую через WAN без задержек и модификаций.

---

## 16. Универсальный менеджер подписок и URLTest Latency Selector в sing-box

### 1. Поддержка всех типов протоколов и подписок
В `openstream.uc` и веб-интерфейсе `servers.js` реализован универсальный парсер, поддерживающий:
* **VLESS**: с поддержкой Reality, Vision и новейшего транспорта xHTTP;
* **Hysteria 2 / hy2**: сверхскоростной протокол на базе UDP QUIC;
* **TUIC v5**: высокоэффективный протокол с контролем перегрузок BBR;
* **Shadowsocks / Shadowsocks 2022**: с поддержкой современных шифров 2022-ietf;
* **Trojan** и **VMess**;
* **Подписки**: URL-ссылки `https://...`, списки Base64 и конфигурации Clash / Mihomo YAML (`proxies:`).

### 2. Автоматический выбор самого быстрого узла (URLTest)
В конфигурацию sing-box генерируется outbound-группа `urltest`:
```json
{
  "type": "urltest",
  "tag": "auto-fastest-vpn",
  "outbounds": ["de-frankfurt-01", "nl-amsterdam-02", "fi-helsinki-01"],
  "url": "https://cp.cloudflare.com/generate_204",
  "interval": "3m",
  "tolerance": 50
}
```
sing-box в фоновом режиме каждые 3 минуты замеряет время отклика через Cloudflare Generate 204 и направляет трафик на сервер с наименьшим RTT.

---

## 17. Multi-DNS Failover, Bootstrap DNS и Сетевая безопасность

### 1. Устранение дедлоков через Bootstrap DNS
При использовании шифрованных DNS (DoH/DoT) возникает циклическая зависимость: чтобы разрешить имя DoH-сервера `1.1.1.1` или `dns.google`, клиенту нужен работающий DNS.  
В OpenStream 2.1 в конфигурацию sing-box внедрен статический Bootstrap DNS (`77.88.8.8`, `1.1.1.1`), работающий напрямую (`detour: direct`), что исключает зависания при старте сети.

### 2. Защита сетевого стека в nftables
* **Блокировка QUIC (UDP 443)**: `udp dport 443 reject`. Форсирует TCP TLS 1.3 в браузерах, что критически важно для эффективной десинхронизации Zapret2 на YouTube 4K.
* **Блокировка DoH/DoT (TCP 853)**: `tcp dport 853 reject`. Предотвращает обход таблиц маршрутизации через браузерные встроенные DNS-клиенты.
* **Пропуск NTP (UDP 123)**: `udp dport 123 return`. Защищает службу синхронизации времени chrony/systemd-timesyncd от искажения сетевых пакетов.

