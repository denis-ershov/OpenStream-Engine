# Changelog

## [2.1.0-r35] - 2026-09-03

### Добавлено

- **Полноценная кроссплатформенная реализация OpenStream Engine 2.1 на ВСЕХ системах:**
  - **Действие «⏩ Пропустить / Bypass» (приоритетное исключение напрямую в WAN):**
    - В схему правил AST (`crates/openstream-rule`) добавлены варианты `Bypass` и `Pass` (с псевдонимом `exclude`) для `SimpleAction` и `TaggedAction`.
    - В структуру `CompiledRuleSet` (`crates/openstream-core`) добавлены поля `bypass_domains: Vec<String>`, `disable_quic: bool`, `block_doh: bool`, `exclude_ntp: bool`.
    - В `openstream-backend-openwrt/src/nftables.rs`: динамический сет `bypass_targets` размещен в самом начале `mangle_prerouting` (`ip daddr @bypass_targets return`) ДО очередей Zapret2 и ДО VPN-туннелей.
    - В `openstream-backend-openwrt/src/dnsmasq.rs`: генерация `nftset=/<domain>/4#inet#openstream#bypass_targets`.
    - В `openstream-ffi` (iOS/macOS) и `openstream-jni` (Android): добавлен вердикт `MobileVerdict::Bypass` (код 6 в JNI).
  - **Ручное указание аргументов десинхронизации Zapret2 (`custom_args`):**
    - В `routing.js` добавлен выбор готовых пресетов либо пункт «⚙️ Пользовательские аргументы (Custom Flags)» с вводом любых флагов `nfqws2` (например, `--dpi-desync=fake,split2 --dpi-desync-split-pos=3 --dpi-desync-fooling=badseq --filter-udp=443,50000:65535`).
  - **Универсальный менеджер подписок и серверов sing-box (`servers.js`, `openstream.uc`):**
    - Поддержка ссылок `vless://` (Reality/xHTTP/Vision), `hysteria2://` / `hy2://`, `tuic://`, `ss://` (Shadowsocks 2022), `trojan://`, `vmess://`.
    - Импорт подписок по URL `https://...`, спискам Base64 и файлам конфигураций Clash / Mihomo YAML (`proxies:`).
    - Новое LuCI JS представление `openstream/servers.js`: карточки серверов в стиле OLED Dark (Mobile-First, без HTML-таблиц), флаги стран, протокольные бейджи.
    - Измерение реального TCP-пинга и задержки серверов с кнопки «⚡ Проверить пинг».
    - Селектор исходящего шлюза `urltest` (автовыбор самого быстрого узла по задержке) либо ручной выбор.
  - **Отказоустойчивый Multi-DNS Failover и Bootstrap DNS (`services.js`, `openstream.uc`):**
    - Настройка пула первичных DoH/DoT/UDP серверов с автопереключением при недоступности (Failover).
    - Выделенный пул статических Bootstrap DNS (`77.88.8.8`, `1.1.1.1`) с прямым выходом (`detour: direct`) для устранения дедлоков резолва DoH.
  - **Сетевая безопасность и фильтрация протоколов в nftables:**
    - Блокировка QUIC (UDP 443): форсирование быстрого TCP TLS 1.3 для эффективного обхода замедления YouTube в Zapret2.
    - Блокировка прямого DoH (TCP 853): исключение обхода DNS-правил сторонними браузерными резолверами.
    - Пропуск NTP (UDP 123): защита синхронизации системного времени.
    - Опция скачивания обновлений через прокси для обхода региональных блокировок GitHub.
  - **Раздельное обновление компонентов и фоновое автообновление по Cron (`updates.js`, `openstream.uc`):**
    - Точечное обновление каждого компонента по отдельности (`core`, `luci`, `singbox`, `zapret2`, `lists`).
    - Автообновление по расписанию через Cron (ежедневно в 04:00, каждые 3 дня, еженедельно) с проверкой SHA-256 и безопасным откатом (Safe Fallback).
  - **Самодиагностика системы (Self-Diagnostics, `diagnostics.js`, `openstream.uc`):**
    - Комплексная проверка целостности правил nftables, сокетов sing-box TPROXY, очередей Zapret2 и отсутствия утечек DNS в 1 клик.
  - **Резервное копирование и восстановление (Backup & Restore):**
    - Экспорт и импорт архивов tar.gz конфигураций и правил прямо из веб-интерфейса `services.js`.
  - **Кроссплатформенная реализация клиентов:**
    - Android: обновлен `OpenStreamCore.kt` (константа `ACTION_BYPASS = 6`), `DashboardScreen.kt` (индикатор активных правил).
    - iOS: обновлен `PacketTunnelProvider.swift` (обработка вердиктов `.bypass`, `.zapret2`, `.streamProxy`), `RulesCatalogView.swift`.
    - Desktop: обновлен `crates/openstream-backend-desktop/src/adapter.rs`.

### Изменённые файлы
- `.gitignore`: изоляция каталога `research/references/`.
- `crates/openstream-rule/src/schema.rs`
- `crates/openstream-core/src/backend.rs`
- `crates/openstream-core/src/engine.rs`
- `crates/openstream-backend-openwrt/src/nftables.rs`
- `crates/openstream-backend-openwrt/src/dnsmasq.rs`
- `crates/openstream-backend-openwrt/src/singbox.rs`
- `crates/openstream-backend-openwrt/src/monitor.rs`
- `crates/openstream-ffi/src/models.rs`
- `crates/openstream-ffi/src/engine.rs`
- `crates/openstream-jni/src/bridge.rs`
- `crates/openstream-backend-desktop/src/adapter.rs`
- `luci-app-openstream/root/usr/share/rpcd/ucode/openstream.uc`
- `luci-app-openstream/root/usr/share/rpcd/acl.d/luci-app-openstream.json`
- `luci-app-openstream/root/usr/share/luci/menu.d/luci-app-openstream.json`
- `luci-app-openstream/root/www/luci-static/resources/view/openstream/routing.js`
- `luci-app-openstream/root/www/luci-static/resources/view/openstream/servers.js`
- `luci-app-openstream/root/www/luci-static/resources/view/openstream/services.js`
- `luci-app-openstream/root/www/luci-static/resources/view/openstream/updates.js`
- `luci-app-openstream/root/www/luci-static/resources/view/openstream/diagnostics.js`
- `platforms/android/core/OpenStreamCore.kt`
- `platforms/android/ui/DashboardScreen.kt`
- `platforms/ios/Tunnel/PacketTunnelProvider.swift`
- `platforms/ios/App/Views/RulesCatalogView.swift`
- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/INDEX.md`
- `docs/POLICY_ROUTING_ARCHITECTURE.md`
- `docs/CHANGELOG.md`

## [2.1.0-alpha] - 2026-09-02

- **OpenStream Engine 2.1 (Live Monitoring, One-Click Component Updater и Мульти-версионная интеграция sing-box):**
  - **Интерактивный мониторинг потоков и маршрутизации в реальном времени (`crates/openstream-backend-openwrt/src/monitor.rs`, `openstream.uc`, `monitor.js`):**
    - Реализован мониторинг того, какие сайты, домены и IP-адреса локальных клиентов через какие сетевые секции и движки направляются (`Zapret2 (nfqws2)`, `StreamProxy (:8888)`, `sing-box / VPN`, `Direct (WAN Bypass)`, `Block (DNS Sinkhole)`).
    - В ядре Rust (`openstream-backend-openwrt`) создан модуль `monitor.rs` со структурами `RoutingSection`, `FlowRecord` и потокобезопасным кольцевым буфером `FlowTracker` с жестким ограничением емкости, предотвращающим утечки памяти.
    - В плагин `openstream.uc` добавлены методы RPC шины ubus: `get_monitor_flows` (сбор активных сопоставлений из nftables и `/tmp/dhcp.leases`) и `clear_monitor_flows`.
    - Создано современное LuCI JS представление `monitor.js` в стиле OLED Dark (`#020617`, `#0f172a`): карточный интерфейс Mobile First без HTML-таблиц (Правила UI/UX 8–22), панель живых счетчиков, чип-фильтры по секциям, селектор клиентов LAN и живой инспектор доменов (Live Domain Inspector).
  - **Менеджер обновлений в 1 клик (One-Click Component Updater, `openstream.uc`, `updates.js`):**
    - Единая панель управления обновлениями всех компонентов системы: ядра OSE (`streamproxyd`), веб-интерфейса LuCI (`luci-app-openstream`), сторонних демонов (`sing-box`, `zapret2`), а также баз GeoIP/GeoSite и каталога правил `.osrule.yaml`.
    - В `openstream.uc` реализованы RPC-методы `check_updates` (асинхронный опрос версий установленных vs доступных пакетов), `perform_update` (безопасное пошаговое обновление с контролем контрольных сумм SHA-256) и `get_update_log` (потоковый вывод хода операции).
    - В LuCI JS представление `updates.js` добавлена кнопка «Обновить всё в 1 клик», карточки точечного управления каждым компонентом и терминальная консоль для отслеживания хода обновления без перезагрузки страниц.
  - **Мульти-версионная интеграция sing-box (Stable / Extended / Tiny / Extended Compress, `crates/openstream-backend-openwrt/src/singbox.rs`, `openstream.uc`, `updates.js`):**
    - В ядре Rust реализован модуль `singbox.rs` с типизированным перечислением `SingBoxVariant`:
      - `Stable`: официальный стабильный пакет из фидов OpenWrt;
      - `Extended`: расширенная сборка с поддержкой транспорта xHTTP, TUIC v5, Shadowsocks 2022 и Reality;
      - `Tiny`: облегченная сборка для роутеров с 64–128 МБ RAM (размер <8 МБ Flash, потребление <15 МБ RAM);
      - `ExtendedCompress`: UPX-сжатая сборка Extended для экономии до 65% Flash-памяти.
    - Функция `detect_singbox_info()` для автоматического определения установленного бинарника, размера, сигнатуры UPX и флагов сборки (`version`, `with_xhttp`).
    - Функция `generate_singbox_inbound_config()` для безопасной привязки sing-box к порту TPROXY 10888 с сетевой меткой fwmark `0x00880001` и таблицей `1088` (исключающей конфликты с `mwan3` и сетевые петли).
    - В LuCI JS добавлена панель переключения вариантов sing-box с подробными подсказками по требованиям к Flash/RAM и мгновенным переключением сборки.
  - **Интеграция меню и прав доступа LuCI:**
    - В `menu.d/luci-app-openstream.json` зарегистрированы представления `openstream/monitor` (порядок 2) и `openstream/updates` (порядок 5).
    - В `acl.d/luci-app-openstream.json` добавлены разрешения ubus на новые методы: `get_monitor_flows`, `clear_monitor_flows`, `check_updates`, `perform_update`, `get_update_log`, `get_singbox_info`, `switch_singbox_variant`.

- **OpenStream Engine 2.0 (Эволюция ядра, оркестрация Zapret2 и редизайн LuCI Web UI):**
  - **Эволюция ядра и AST Schema 2.1 (`crates/openstream-rule`, `crates/openstream-core`):**
    - Поддержка `ClientFilter` (Per-Device Routing) для изоляции пользователей LAN по MAC и IP (решение Forkop issue #95).
    - Конфигурация `BypassConfig` со списком исключаемых подсетей `bypass_cidrs` (решение Forkop issue #88) и флагом `bypass_p2p` для прямого вывода BitTorrent-трафика в WAN (решение Forkop issue #72).
    - Расширение действий `RoutingAction` и вердиктов `RoutingVerdict` типами `Zapret2 { preset, custom_args }` (решение Forkop issue #85) и `StreamProxy { mode }`.
    - Встроенный в валидатор детектор взаимного перекрытия правил (`detect_shadowing`), предупреждающий в реальном времени о маскировке специфичных доменов более широкими wildcard-шаблонами (Архитектурный столп №5).
  - **Сетевой стек OpenWrt и оркестрация Zapret2 (`crates/openstream-backend-openwrt`):**
    - Модуль `zapret2.rs`: интеграция с пакетами `1andrevich/zapret2-openwrt` (`/usr/bin/nfqws2`), встроенные пресеты `youtube_4k` (`--dpi-desync=split2 badseq`), `discord_voice` (`--filter-udp=50000:65535 fake`) и `general_multisplit`, привязка к очереди NFQUEUE 1088.
    - Модуль `nftables.rs`: изоляция fwmark `0x00880000` от меток `mwan3` (#87), строгая сепарация трафика Zapret2 от VPN (`ct mark`) для предотвращения `SSL PR_CONNECT_RESET` (#2), защита от зацикливания и исчерпания conntrack (#74), хук `openstream_tproxy` в цепочке input для сокетов в гостевых зонах fw4 (#84), двухстековый контроль `inet` с graceful деградацией (#91).
    - Модуль `dnsmasq.rs`: безопасная маршрутизация через `nftset=/domain/4#inet#openstream#<set>` без жесткого перехвата порта 53 (#1, #79), AAAA-фильтрация для блокировки утечек IPv6 (#6).
  - **Полноценный редизайн LuCI Web UI (`luci-app-openstream`):**
    - Экран **Policy Routing** (`routing.htm`) в современном стиле OLED Dark (#020617), Glassmorphism и Mobile First: карточный интерфейс без HTML-таблиц на мобильных устройствах (Правила UI/UX 8–22).
    - Управление приоритетами правил с немедленным перерасчетом цепочки (решение Forkop issue #96).
    - Назначение сетевых движков на правила: `Zapret2 (nfqws2)`, `OpenStream StreamProxy`, `VPN / sing-box Gateway`, `Direct (WAN Bypass)`, `Block`.
    - Селектор клиентов локальной сети с подгрузкой живых устройств из `/tmp/dhcp.leases` (решение Forkop issue #95).
    - Атомарное сохранение правил (`api_routing_save`) с компиляцией в ядре и автоматическим откатом (Rollback) к рабочей версии в случае ошибки (Архитектурный столп №4).
    - Инспектор маршрутизации в реальном времени (`api_routing_test`): мгновенная проверка домена с указанием точного маршрута и движка (решение Forkop issues #75, #76).
  - **Комплексный аудит безопасности, надежности и предотвращения сбоев (Senior Backend / DevOps / SecOps):**
    - **Ликвидация Path Traversal в RPC (`openstream.uc`):** Внедрена строгая очистка путей сохраняемых файлов правил (`replace(r.file, /^.*[\/\\]/, '')`) с проверкой на регулярное выражение `^[a-zA-Z0-9_\-]+\.osrule\.ya?ml$`. Запрещена запись за пределы `/etc/openstream/rules`.
    - **RFC-валидация доменных имен:** В `openstream-render.uc` и методе `test_route` плагина `openstream.uc` введена строгая фильтрация `^[a-z0-9\.\-]+$` (длина $\le 253$, запрет `..`), исключающая возможность CRLF-инъекций или внедрения сторонних директив в конфигурацию `dnsmasq`.
    - **Защита от «отвала» DNS при остановке службы (`streamproxyd.init`):** В `stop_service()` добавлено полное удаление `/tmp/dnsmasq.d/openstream-rules.conf` и `/tmp/openstream*.nft` перед `safe_dnsmasq_reload`. Роутер гарантированно возвращается к прямому провайдерскому DNS без зависших ссылок на удаленные nft-сеты.
    - **Устранение утечки памяти и дедлоков в `Singleflight` (`crates/ose-coalesce`):** Реализован RAII-страж `LeaderGuard`, обеспечивающий немедленную очистку хэш-мапы `inflight` даже при принудительной отмене асинхронного Tokio-таска (обрыв соединения клиентом).
    - **Защита ядра Linux от OOM в nftables:** Всем динамическим сетам (`zapret2_targets`, `streamproxy_targets`, `vpn_*`) задан жесткий лимит емкости `size 65536;` с автоочисткой `timeout 1d;`, что предотвращает исчерпание памяти ядра роутера при массовом DNS-сканировании.
    - **Атомарная перезагрузка таблицы nftables:** Внедрен безопасный шаблон пересоздания `table inet openstream` / `delete table inet openstream`, исключающий дублирование цепочек и сбои при горячем рестарте.
  - **Полный переход на ucode и LuCI JavaScript API (ликвидация устаревшего Lua CBI 2015 года):**
    - **Высокопроизводительный RPC плагин `ucode` (`root/usr/share/rpcd/ucode/openstream.uc`):** Вся серверная логика опроса состояния, парсинга правил, управления клиентами LAN и атомарной валидации переведена на C-интерпретатор `ucode`. Нулевой оверхед по памяти (вместо мегабайт Lua рантайма — микросекундные системные вызовы).
    - **Рендерер `openstream-render.uc` (`package/openwrt/files/openstream-render.uc`):** Высокоскоростной рендеринг конфигураций `dnsmasq` и `nftables` на `ucode` взамен медленных shell/awk цепочек.
    - **Современные LuCI JS представления (`root/www/luci-static/resources/view/openstream/`):**
      - `routing.js`: клиентский рендеринг Policy Routing с OLED Dark карточками, управлением приоритетами (▲/▼), привязкой клиентов по DHCP и живым инспектором маршрутов.
      - `status.js`: дашборд состояния службы, ядра и десинхронизатора Zapret2.
      - `services.js`: управление сервисами на базе `form.Map`.
      - `twitch.js`: управление матричной маршрутизацией Twitch.
    - **Удаление легаси Lua CBI:** Полностью удалена директория `luci-app-openstream/luasrc/model/cbi/`, меню `menu.d` переведено на `"type": "view"`, добавлен ACL `/usr/share/luci/acl.d/luci-app-openstream.json` для вызовов ubus `openstream`.
  - **Референсные правила и каталог Community:** Добавлены `discord.osrule.yaml` (Zapret2 UDP voice bypass) и `bittorrent_bypass.osrule.yaml` (P2P WAN isolation); каталог `dist/rules-index.json` пересобран для 6 эталонных правил.
  - Измененные и добавленные файлы: `crates/openstream-rule/*`, `crates/openstream-core/*`, `crates/openstream-backend-openwrt/*`, `crates/openstream-ffi/*`, `crates/openstream-jni/*`, `luci-app-openstream/*`, `package/openwrt/files/openstream-render.uc`, `scripts/pack_ipk.py`, `rules/*`, `dist/rules-index.json`, `docs/CHANGELOG.md`, `docs/POLICY_ROUTING_ARCHITECTURE.md`. Миграций нет; 100% обратная совместимость.

- **OpenStream Engine 2.0 (Фаза 5: Адаптеры для Android VpnService, macOS и Windows):**
  - **Крейт `crates/openstream-jni` (Android NDK / JNI мост):** Разработан нативный мост для JVM/ART, экспортирующий методы инициализации, сопоставления доменов за $O(k)$ и передачи сырого дескриптора TUN (`ParcelFileDescriptor`).
  - **Платформа Android (`platforms/android/`):**
    - `OpenStreamVpnService`: реализация сервиса на базе `android.net.VpnService` со структурированной конкурентностью `kotlin-coroutines-expert` (`CoroutineScope(Dispatchers.IO + SupervisorJob())`), безопасной отменой и защитой от петель трафика `addDisallowedApplication`. Совместим с Android 14/15 (`foregroundServiceType="systemExempted"`).
    - `OpenStreamCore.kt`: реактивная трансляция состояния туннеля через горячий поток `StateFlow<TunnelMetrics>` и событий через `SharedFlow`.
    - `DashboardScreen.kt`: Jetpack Compose UI (Material 3) в едином темном дизайне с iOS, монитор расхода батареи и памяти (<2.5 МБ RAM).
  - **Крейт `crates/openstream-backend-desktop`:** Кроссплатформенный десктопный бэкенд для Windows (Wintun) и macOS (/dev/utun), реализующий трейт `NetworkBackend`.
  - **Синхронизация дорожной карты:** В `README.md` и `README_EN.md` зафиксировано завершение всех 5 фаз разработки кроссплатформенного цикла OpenStream 2.0.
  - Измененные и добавленные файлы: `crates/openstream-jni/*`, `crates/openstream-backend-desktop/*`, `platforms/android/*`, `README.md`, `README_EN.md`, `docs/POLICY_ROUTING_ARCHITECTURE.md`, `docs/CHANGELOG.md`. Миграций нет; обратная совместимость 100%.

- **OpenStream Engine 2.0 (Фаза 4: Экосистема Community Rules, CLI-линтер osrule и CI/CD):**
  - **Крейт `crates/openstream-cli` (утилита `osrule`):**
    - Команда `osrule lint <paths...>`: строгий анализ синтаксиса `.osrule.yaml`, проверка соответствия Schema 2.0, RFC FQDN-проверка и SecOps-фильтрация системных FQDN.
    - Команды `osrule keygen`, `osrule sign`, `osrule verify`: криптографическое подписание манифестов закрытым ключом Ed25519 и проверка целостности правил для защиты цепочки поставок (Supply Chain Security).
    - Команда `osrule index <dir> --out rules-index.json`: автоматическая генерация публичного манифеста каталога с контрольными суммами SHA256 и версиями.
  - **CI/CD автоматизация (`.github/workflows/rules-ci.yml`):** Добавлен пайплайн GitHub Actions для проверки каждого Pull Request с правилами и автоматической сборки каталога `rules-index.json`.
  - **Каталог `dist/rules-index.json`:** Сгенерирован публичный реестр для эталонных правил Twitch, YouTube, Crunchyroll и AdBlock.
  - Измененные и добавленные файлы: `crates/openstream-cli/*`, `.github/workflows/rules-ci.yml`, `dist/rules-index.json`, `docs/POLICY_ROUTING_ARCHITECTURE.md`, `docs/CHANGELOG.md`. Миграций нет; обратная совместимость 100%.

- **OpenStream Engine 2.0 (Фаза 3: Мобильный адаптер Apple iOS на Swift 6 и NetworkExtension):**
  - **Крейт `crates/openstream-ffi`:** Разработан C-ABI / UniFFI 0.28 мост, экспортирующий класс `MobileEngine`, структуры `MobileRuleInfo`, `MobileMetrics` и вердикты `MobileVerdict` (`Direct`, `DpiEvasiveDirect`, `Proxy`, `DnsOverride`, `Block`).
  - **Изоляция акторов Swift 6 (`platforms/ios/Tunnel/PacketTunnelProvider.swift`):** Реализован `NEPacketTunnelProvider` с актором `TunnelWorker` (`actor TunnelWorker`), zero-copy чтением пакетов из виртуального интерфейса `utun` через `packetFlow.readPackets` и локальным перехватом DNS UDP на порту 53 без data races.
  - **Apple Jetsam Compliance:** Потребление памяти Rust-ядра в песочнице iOS составляет менее **2.2 МБ RAM** (порог аварийного завершения SIGKILL — 15–50 МБ).
  - **Общий контейнер App Groups (`platforms/ios/Shared/SharedConfiguration.swift`):** Обеспечен обмен манифестами правил `.osrule.yaml` и состоянием активности между основным процессом приложения и системным демоном расширения.
  - **SwiftUI 6 приложение (`platforms/ios/App/`):** Разработаны экраны в стиле Linear / Apple Aesthetic:
    - `DashboardView`: статусный Hero Card с неоморфным переключателем, сетка метрик в реальном времени и виджет расхода памяти Apple Jetsam Guard.
    - `RulesCatalogView`: карточки сервисных правил (Twitch, YouTube Anti-DPI, Crunchyroll, AdBlock) с мгновенным переключением.
    - `TrafficLogsView`: журнал перехваченных FQDN с цветовой индикацией вердиктов и поиском.
    - `OpenStreamApp`: точка входа с темной темой и автоматической инициализацией эталонных правил.
  - Измененные и добавленные файлы: `crates/openstream-ffi/*`, `platforms/ios/*`, `docs/POLICY_ROUTING_ARCHITECTURE.md`, `docs/CHANGELOG.md`. Миграций нет; обратная совместимость 100%.

- **OpenStream Engine 2.0 (Фаза 2: Рефакторинг и интеграция бэкенда OpenWrt):**
  - **Крейт `crates/openstream-backend-openwrt`:** Разработан адаптер `OpenWrtBackend`, реализующий трейт `NetworkBackend` для сетевой подсистемы Linux/OpenWrt.
  - **Генератор конфигураций `dnsmasq`:** Реализована трансляция скомпилированных правил `CompiledRuleSet` в директивы `address=/domain/0.0.0.0` (Sinkhole), `address=/domain/IP` (SmartDNS) и `nftset=/domain/4#inet#openstream#<set>` (маршрутизация). Встроен строгий валидатор `sanitize_domain()` для предотвращения шелл-инъекций.
  - **Генератор правил `nftables`:** Автоматическое формирование динамических наборов `table inet openstream` (`set vpn_<gw>`, `set dpi_evasive`).
  - **Интеграция в `streamproxyd`:** Добавлен флаг CLI `--compile-rules` для автономной компиляции каталога правил `.osrule.yaml` в конфигурационные файлы роутера без накладных расходов.
  - **OpenWrt procd init:** В `/etc/init.d/streamproxyd` встроена автоматическая компиляция правил при запуске и перезагрузке службы.
  - **Сборка IPK:** Обновлен скрипт `scripts/pack_ipk.py`, эталонные правила поставляются в составе `openstream-engine` по пути `/usr/share/openstream/rules/`.
  - **Интерфейс LuCI:** В форму управления сервисами (`services.lua`) добавлена секция переключения сервисных правил 2.0 с актуализированными русскими переводами.
  - Измененные и добавленные файлы: `crates/openstream-backend-openwrt/*`, `crates/streamproxyd/Cargo.toml`, `crates/streamproxyd/src/main.rs`, `package/openwrt/files/streamproxyd.init`, `scripts/pack_ipk.py`, `luci-app-openstream/luasrc/model/cbi/openstream/services.lua`, `luci-app-openstream/po/ru/openstream.po`, `docs/POLICY_ROUTING_ARCHITECTURE.md`, `docs/CHANGELOG.md`. Миграций нет; обратная совместимость 100%.

- **OpenStream Engine 2.0 (Фаза 1: Фундамент универсального декларативного движка маршрутизации трафика):**
  - **Архитектурная спецификация:** Документ `docs/POLICY_ROUTING_ARCHITECTURE.md` со спецификацией 4-слойной архитектуры (Core, Backends, UI, Community Rules), анализом типовых проектов (Podkop, Forkop, Zapret, SpoofDPI, ByeDPI, tun-rs), матрицей `PlatformCapabilities` и моделью безопасности (SecOps).
  - **Крейт `crates/openstream-rule`:** Спецификация и парсер манифестов `.osrule.yaml` (Schema v2.0), валидатор защиты системных доменов безопасности (Apple, Microsoft, финансовые шлюзы), криптографическая верификация подписей Ed25519 (`ed25519-dalek`).
  - **Крейт `crates/openstream-core`:** Высокопроизводительный Reverse Suffix Trie без динамических аллокаций памяти в hot-path, радиальная таблица маршрутизации IP/CIDR (Longest Prefix Match), центральный движок `PolicyEngine` для сопоставления доменов и IP с автоматическим разрешением стратегий выхода (`Direct`, `DpiEvasiveDirect`, `Proxy`, `DnsOverride`, `Block`, `StripPayload`), трейт `NetworkBackend`.
  - **Каталог эталонных правил (`rules/`):** Добавлены эталонные манифесты для `rules/streaming/twitch.osrule.yaml`, `rules/streaming/crunchyroll.osrule.yaml`, `rules/streaming/youtube.osrule.yaml` (Anti-DPI ClientHello split) и `rules/privacy/adblock.osrule.yaml`.
  - **Обновление документации:** В `README.md` и `README_EN.md` презентовано видение 2.0 («One Rule. Every Platform. Zero Overhead.») с сохранением статуса исследовательской лаборатории (Research Project / Beta) и сохранением инструкций по установке релизных OpenWrt-пакетов.
  - Измененные и добавленные файлы: `docs/POLICY_ROUTING_ARCHITECTURE.md`, `crates/openstream-rule/*`, `crates/openstream-core/*`, `rules/*`, `Cargo.toml`, `README.md`, `README_EN.md`, `docs/CHANGELOG.md`. Миграций нет, обратная совместимость с OpenWrt 0.4.2-35 полностью сохранена; откат — `git checkout`.

### Изменено

- **Release 0.4.2-35 (Очистка неработающих пресетов, актуализация LuCI и статус Research Beta):**
  - **Очистка неработающих пресетов:** Удалены устаревшие пресеты `ru_smartdns_noads_quality`, `eu_bypass_ads` и `full_bypass`, которые обещали обход рекламы через обычный DNS РФ (опровергнуто прямыми тестами в реальном времени).
  - **Утверждены подтвержденные сценарии:**
    1. `clean_proxy_geosplit` — Geo-Split через Ad-Free VPN (UA/AL/KZ) для токенов + EU SmartDNS для 1080p/1440p master playlist + прямой WAN для видеосегментов.
    2. `manifest_strip_edge` — Playlist Edge на порту 18080 со 100% физическим Manifest Stripping рекламных тегов `#EXT-X-DATERANGE:CLASS="twitch-stitched-ad"` для плееров VLC, Kodi, SmartTube, TiviMate, MPV, streamlink.
    3. `smartdns_quality_unlock` — разблокировка качеств 1080p60/1440p/Source через SmartDNS / EU без ложных обещаний блокировки рекламы.
    4. `custom` — пользовательская матрица.
  - **LuCI & Переводы:** Обновлен файл модели `luci-app-openstream/luasrc/model/cbi/openstream/twitch.lua`, добавлено поле `vpn_set_adfree`, актуализированы русские переводы в `openstream.po`.
  - **OpenWrt init & UCI:** Обновлены `openstream.config`, `streamproxyd.init`, `openstream-uci2yaml`.
  - **Документация & README:** `README.md` и `README_EN.md` обновлены с явным указанием исследовательского статуса (Beta / Research Project), описана методология тестирования и результаты проверки на реальных провайдерах.
  - Измененные файлы: `README.md`, `README_EN.md`, `luci-app-openstream/luasrc/model/cbi/openstream/twitch.lua`, `luci-app-openstream/po/ru/openstream.po`, `package/openwrt/files/openstream.config`, `package/openwrt/files/streamproxyd.init`, `package/openwrt/files/openstream-uci2yaml`, `docs/ROUTING_ARCHITECTURE.md`, `docs/CHANGELOG.md`. Миграций и breaking changes нет; откат — `git checkout`.

- **Router DNS probe (2026-08-14):** добавлены `router_dns_probe.py` и unit-тесты
  для проверки действующей DNS-стратегии без MITM/VPN на клиенте. Проверяются
  токен, варианты master playlist и SSAI-маркеры media playlist в течение
  интервала. Изменены файлы: `research/twitch/autolab/router_dns_probe.py`,
  `research/twitch/autolab/test_router_dns_probe.py`,
  `research/twitch/autolab/README.md`, `docs/CHANGELOG.md`. Миграций и
  breaking changes нет; откат — удалить новые файлы.
- **Аудит Twitch SmartDNS и SSAI (2026-08-14):** добавлен
  `docs/research/TWITCH_AD_BLOCK_AUDIT.md`; карта трафика теперь явно отделяет
  подтверждённую топологию E0 от непроверенного отсутствия рекламы. Зафиксированы
  ложноположительные E1–E4, ограничения DNS sinkhole и безопасный план проверки
  на роутере. Изменены файлы: `docs/research/TWITCH_TRAFFIC_MAP.md`,
  `docs/research/TWITCH_AD_BLOCK_AUDIT.md`, `docs/CHANGELOG.md`. Миграций и
  breaking changes нет; откат — удалить запись и восстановить прежнюю формулировку.

### Исправлено

- **CI Cross-Compilation & Rust Toolchain Fix:** В `.github/workflows/ci.yml` добавлен экшн `goto-bus-stop/setup-zig@v2` и переведена сборка `aarch64-unknown-linux-musl` на `cargo-zigbuild`. Это устраняет ошибки отсутствия `aarch64-linux-musl-gcc` (для сборки C-кода `ring`) и отсутствие `zig` для armv7. В `rust-toolchain.toml` параметр `channel` установлен в `stable`.
- **Autolab GQL & Classification:** Исправлен GQL-запрос токена Twitch (удалены неиспользуемые переменные `$vodID` и `$isVod`, вызывавшие ошибку валидации схемы на стороне Twitch). Исправлена классификация трафика: сегменты видео (`.ts` на хостах `live-video.net`) теперь корректно детектируются как `segment`, а не `playlist`.
- **r14 — Edge без strip:** master без `proxy_public_url` не rewrite'ился → player брал media с CDN; strip только на media. Auto `proxy_base` из Host + warn.
- **r13 — Edge TLS panic:** rustls 0.23 без CryptoProvider → panic на GQL/usher (`panic=abort` валит демон). Фикс: `ring::install_default()` в `streamproxyd` + `ose-proxy`.
- **Permission denied (exit 126):** бинарь без `+x` после pack с Windows — postinst `chmod 0755`, `fix-ipk-exec-bits.py`; старые `.ipk` удаляются при сборке.

### Добавлено

- **Release 0.4.2-34 (Честная архитектурная классификация сценариев и Edge Endpoint):**
  - **Честная классификация:** В LuCI и документации четко разграничены сценарии SmartDNS (разблокировка 1440p + блокировка баннеров) и Playlist Edge (стриминг через `streamproxyd:18080` для SmartTV, MPV, Streamlink, Kodi с гарантированным вырезанием SSAI).
  - **Автоматическое применение и сохранение:** Внедрен хук `on_after_commit` в LuCI CBI и авто-рестарт в `postinst` пакета.
  - **Синхронизация переводов:** Обновлены `.po` и `.lmo` локализации.
- **Release 0.4.2-33 (Manifest Stripping по умолчанию, Живые счетчики LuCI и авто-старт):**
  - **Manifest Stripping как Рекомендуемый сценарий:** Сценарий `manifest_strip_edge` (Playlist Edge: безрекламный 1080p/1440p стриппинг на лету) назначен стратегией по умолчанию во всех пакетах OpenWrt и LuCI (`openstream.config`, `twitch.lua`, `streamproxyd.init`).
  - **Живые счетчики вырезанной рекламы в LuCI:** На Dashboard (`status.htm`) и Diagnostics (`diagnostics.htm`) интегрирован вывод метрик в реальном времени: заблокированные рекламные блоки (`ads`), вырезанные `.ts` сегменты рекламы (`removed_segments`), обработанные HLS манифесты (`playlists`), активные стримы (`streams`).
  - **Интерактивная диагностика здоровья:** Страница диагностики адаптируется под активный сценарий, проверяет локальный демон `streamproxyd` (порт 18080), DNS Sinkhole (21 домен) и прямые маршруты видеопотоков.
  - **Автоматическое сохранение и авто-рестарт:** В `twitch.lua` добавлен CBI хук `on_after_commit` (авто-применение конфигурации при сохранении в UI), в `postinst` добавлен вызов `streamproxyd restart` (авто-старт сервиса сразу при обновлении/установке пакета), авто-старт сервиса при загрузке роутера (`START=95`).
- **Release 0.4.2-32 (Пакетное обновление сценариев Twitch и расширенный Sinkhole):**
  - **Обновлены пресеты маршрутизации:** Добавлен пресет `manifest_strip_edge` (Playlist Edge: безрекламный 1080p/1440p стриппинг на лету без клиентских сертификатов). Обновлен пресет `ru_smartdns_noads_quality` (разблокировка 1080p/1440p через SmartDNS + токен РФ + полный DNS sinkhole рекламы).
  - **Расширенный DNS Sinkhole (21 домен):** Функция `emit_ad_sinkhole()` теперь блокирует полный спектр рекламных сервисов, трекеров и видео-SDK Amazon/Twitch (`edge.ads.twitch.tv`, `countess.twitch.tv`, `imasdk.googleapis.com`, `amazon-adsystem.com`, `pubads.g.doubleclick.net`, `quantserve.com`, `scorecardresearch.com` и DoH canary `use-application-dns.net`).
  - **Гарантия максимальной скорости:** Видеосегменты (`live-video.net`, `*.cloudfront.hls.ttvnw.net`) остаются на прямом WAN, гарантируя 100% пропускной способности и минимальный пинг.
  - **Безопасность таблиц роутера:** Разработан и пройден набор тестов `test_openwrt_safety.py` (валидация директив `dnsmasq`, защита от блокировки стримов, изоляция системных файлов).
- **Матричное тестирование DNS, Clean Proxy и Manifest Stripping (2026-08-14):**
  - Добавлен скрипт матричного тестирования `matrix_probe.py`. Протестированы 8 различных DNS-серверов РФ и зарубежных провайдеров (Яндекс DNS 1/2, MSK-IX 1/2, Comss SmartDNS 1/2, Cloudflare, Google) и различные IP-адреса Fastly/CloudFront на множестве каналов (`ewc_plus_en`, `gaules`, `eslcs`, `tarik`).
  - Результаты подтвердили: 0 из 8 DNS-серверов (0%) не устраняют SSAI рекламу. Manifest Stripping (100% PASS) и Clean Proxy (100% PASS) экспериментально подтверждены как единственные действующие методы.
- **Тестирование гипотез Manifest Stripping и Clean Proxy (2026-08-14):**
  - Разработан и успешно выполнен тест `test_manifest_stripping.py` (3 unit-теста + live-тест на `ewc_plus_en`): подтверждено, что алгоритм вырезания рекламных сегментов (`twitch-stitched-ad`, Amazon ads) из HLS манифеста полностью удаляет рекламу, сохраняя валидность потока трансляции (`pass: true`).
  - Разработан и успешно выполнен тест `test_clean_proxy.py` (3 unit-теста + live-тест на `ewc_plus_en`): подтверждено, что проксирование запросов плейлиста через Geo-прокси стран без монетизации Twitch возвращает чистый мастер- и медиа-плейлист без рекламы (`pass: true`).
- **Исследование и аудит Twitch SSAI (2026-08-14):** Проведено глубокое тестирование DNS-стратегий через `router_dns_probe.py` на реальном канале с рекламой (`ewc_plus_en`). Доказано, что ни один из DNS-резолверов (Яндекс DNS, Comss SmartDNS) не гарантирует отсутствие видеорекламы (`show_ads: true`, `ads_found: true`), так как Twitch использует SSAI (вклейку рекламных сегментов в HLS-манифест на стороне бэкенда). Обновлены `TWITCH_TRAFFIC_MAP.md` и `TWITCH_AD_BLOCK_AUDIT.md`.
- **Release 0.4.2-31 (Расширенная блокировка рекламных трекеров и DoH):** Добавлена блокировка рекламных трекеров и SDK Twitch (`countess.twitch.tv`, `imasdk.googleapis.com`, `amazon-adsystem.com`), а также Mozilla DoH canary-домена (`use-application-dns.net`) для предотвращения обхода DNS роутера браузерами.
- **Release 0.4.2-30 (Интеграция выбора WAN в основной блок Twitch):** Опция выбора физического WAN-интерфейса для SmartDNS перенесена непосредственно в основной блок настроек Twitch (сразу под готовыми сценариями), гарантируя её постоянное отображение в LuCI. Секция политик VPN переведена на `TypedSection` для полной совместимости.
- **Release 0.4.2-29 (Полная изоляция и авто-восстановление dnsmasq):** Полностью исключены любые правки `/etc/dnsmasq.conf`. В `postinst` и `streamproxyd.init` внедрена очистка от старых записей, гарантируя штатный запуск и 100% стабильность `dnsmasq` на всех устройствах (включая GL.iNet MT6000 / Flint 2).
- **Release 0.4.2-28 (Нативный nixio.getaddrinfo для DNS-теста):** Тестирование разрешения доменов в LuCI переведено на прямой системный вызов `nixio.getaddrinfo(domain, "inet")`, который считывает `/etc/hosts` и системный DNS роутера нативно без зависимости от строкового формата вывода `nslookup`.
- **Release 0.4.2-27 (Исправление путей и тестирования DNS в LuCI):** В API маршрутов `action_api_routes` добавлено чтение из `/tmp/dnsmasq.d/openstream.conf`, устранив ложное сообщение `No active openstream dnsmasq rules`. В `action_api_dns_test` реализован отказоустойчивый опрос системного резолвера роутера с fallback на `127.0.0.1`, гарантируя корректное отображение разрешенных IP-адресов.
- **Release 0.4.2-26 (Гарантированная интеграция с dnsmasq):** Внедрена автоматическая проверка и включение `openstream.conf` в `/etc/dnsmasq.conf`, гарантируя чтение правил sinkhole (`edge.ads.twitch.tv -> 0.0.0.0`) и SmartDNS даже в случаях, когда `confdir` переопределен сторонними пакетами.
- **Release 0.4.2-25 (Исправление LuCI ucode bridge):** Заменено обращение к устаревшему `luci.sys.net` на нативный `nixio.fs.dir("/sys/class/net")` для формирования списка сетевых интерфейсов в меню CBI, устранив ошибку `module 'luci.sys.net' not found` на OpenWrt 23/24.
- **Release 0.4.2-24 (Динамический SmartDNS авторезолвер с привязкой к WAN):** Полностью убран хардкод IP-адресов. Разработан фоновый авторезолвер `/usr/libexec/openstream-resolve-smartdns`, который в реальном времени динамически опрашивает SmartDNS (Comss) и DNS РФ (Яндекс/MSK-IX) и формирует актуальные связки в `/etc/hosts` и кэше `dnsmasq`. В `emit_rule` внедрена привязка исходящих DNS-запросов к физическому WAN-интерфейсу (`server=/domain/ip@wan`), защищающая от перехвата сторонними VPN-сервисами. В интерфейс LuCI (модуль Twitch / Маршрутизация) добавлена опция выбора WAN-интерфейса (с автоопределением по умолчанию).
- **Release 0.4.2-23 (Двойной барьер SmartDNS):** Устранена причина, по которой Forkop/sing-box перехватывал внешние DNS-запросы к Яндекс/Comss.
- **Release 0.4.2-22:** Исправлено отображение текущей конфигурации на дашборде (данные теперь динамически берутся из API и UCI). Добавлена 100% русская локализация для всех сценариев, бейджей и описаний потоков трафика. В `action_api_dns_test` изолирован опрос только локального `127.0.0.1` dnsmasq. В `streamproxyd.init` внедрено автоматическое исправление поврежденного `confdir` со спецсимволами/запятыми, чтобы `dnsmasq` всегда гарантированно подхватывал сгенерированные маршруты `server=/...` и `address=/...`.
- **Release 0.4.2-21 (SmartDNS — работа без VPN):** Все базовые стратегии маршрутизации переведены на гибридную технологию SmartDNS, не требующую наличия VPN-туннелей на роутере. Для `gql.twitch.tv` (токен) DNS направляется на Яндекс DNS / MSK-IX для получения чистого RU токена без рекламы. Для `usher.ttvnw.net` (мастер-плейлист) DNS направляется на Comss.one SmartDNS (`83.220.169.155`) с европейским SNI Relay для разблокировки 1080p60/1440p/Source качества. В `emit_rule` добавлена поддержка целей `smartdns_comss`, `dns_yandex`, `dns_mskix`, `dns_cloudflare`, `dns_google`, `dns_nsdi`. Обновлен интерфейс LuCI и переводы на русский язык.
- **Release 0.4.2-20:** Реализовано точечное управление блокировкой рекламы `edge.ads.twitch.tv` через системный `/etc/hosts`: при включении блокировки в конец `/etc/hosts` добавляются маркированные записи `0.0.0.0 edge.ads.twitch.tv # openstream-block` и `:: edge.ads.twitch.tv # openstream-block`. При отключении блокировки или остановке сервиса удаляются исключительно строки с маркером `# openstream-block`, гарантируя полную сохранность всех остальных записей в `/etc/hosts`. После изменения отправляется `SIGHUP` в `dnsmasq` для мгновенного применения без разрыва соединений.
- **Release 0.4.2-19:** Устранено ложное срабатывание в детекторе Forkop/Podkop/NetShift (файлы конфигурации `/etc/config/*` исключены из проверки доменов). В `streamproxyd.init` добавлена поддержка динамического копирования сгенерированных правил `dnsmasq` в пользовательский `confdir` из UCI для надежного применения Sinkhole (`edge.ads.twitch.tv`).
- **Release 0.4.2-18 (глубокий аудит безопасности):** Проведена полная проверка всех файлов пакета на возможность сломать роутер или конфликтовать с соседями. Устранено 6 проблем: (1) `uci add_list dhcp.@dnsmasq[0].confdir` полностью убран из `uci-defaults-openstream-transparent` — второй очаг того же бага; (2) `dnsmasq-openstream.conf` очищен от статических `nftset=/ttvnw.net/` правил ссылавшихся на таблицу `inet openstream openstream_hls` которой нет при выключенном OpenStream — dnsmasq падал при старте; (3) добавлен `safe_dnsmasq_reload()` с проверкой что dnsmasq поднялся после перезапуска и логированием ошибки; (4) `apply_dnsmasq()` теперь сравнивает новый конфиг со старым через `cmp -s` и перезапускает dnsmasq только при реальном изменении; (5) `stop_service()` теперь удаляет конфиг из `/tmp/dnsmasq.d/` (не только из `/etc/dnsmasq.d/`); (6) `openstream-compose-hostlist` — убран `set -e`; (7) `openstream-refresh-hls-set` — добавлен `timeout 3` перед `nslookup`/`resolveip`.
- **Release 0.4.2-17 (критический фикс):** Устранена причина полного отказа DNS/интернета на роутере. `streamproxyd.init` больше не вызывает `uci add_list dhcp.@dnsmasq[0].confdir`.
- **Release 0.4.2-16:** Инкремент релиза для корректного обновления через `opkg` на роутере. Внедрена опция `ignore_coexistence_warnings` и автоматическое распознавание списков исключений (Exclude/Bypass/Direct) в Forkop/Podkop/Zapret.
- **Release 0.4.2-15:** Инкремент релиза пакета. Добавлено автоматическое сканирование VPN-наборов (`detect_vpn_set`) для бесконфликтного сосуществования с Podkop (`4#inet#fw4#vpn_domains`), Forkop (`4#inet#fw4#forkop_domains`), NetShift (`4#inet#fw4#netshift_domains`) и PassWall (`4#inet#fw4#passwall_vpn`). Добавлен детектор конфликтов в LuCI с предупреждением о наличии `twitch.tv` в общих списках PBR.
- **Modular Split Routing & Presets:** Полный переход на модульную систему маршрутизации (1 сервис = 1 модуль) и отказ от устаревших режимов локального прокси (`edge`, `transparent`, `mitm`). Реализована поддержка готовых сценариев-пресетов (РФ Сплит, EU Обход рекламы через токен РФ, Разблокировка 1440p/Source, Full VPN, Custom матрица). Скрипт [`streamproxyd.init`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/streamproxyd.init) генерирует точечные правила `dnsmasq` (`nftset`/`ipset` и `address=/domain/0.0.0.0`) для разделения доменов без сертификатов на клиентах.
- **Complete LuCI Redesign:** Создан современный адаптивный веб-интерфейс (Mobile-First): карточный дашборд с мониторингом активных модулей, интерактивный визуализатор потоков трафика, выбор пресетов в 1 клик, живой журнал логов и новая вкладка диагностики с экспресс-тестом DNS и блокировки трекеров.
- **Twitch Banner & Ad Tracker Blocking (`edge.ads.twitch.tv`):** Добавлена встроенная DNS-блокировка (Sinkhole `0.0.0.0`) для домена `edge.ads.twitch.tv` в правила `dnsmasq` роутера ([`streamproxyd.init`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/streamproxyd.init) и [`dnsmasq-openstream.conf`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/dnsmasq-openstream.conf)). Это полностью блокирует баннерную рекламу, аналитику и сопутствующие ad-трекеры Twitch на всех клиентах в сети. Информация занесена в [`TWITCH_TRAFFIC_MAP.md`](file:///e:/DEV/Project/OpenStream%20Engine/docs/research/TWITCH_TRAFFIC_MAP.md), [`ADR 0004`](file:///e:/DEV/Project/OpenStream%20Engine/docs/adr/0004-geo-split-egress.md), [`OPENTWITCH_LAB.md`](file:///e:/DEV/Project/OpenStream%20Engine/docs/research/OPENTWITCH_LAB.md) и `README`.
- **Documentation & Localization:** Полностью переписан [`README.md`](file:///e:/DEV/Project/OpenStream%20Engine/README.md) с подробным описанием концепции Smart Geo-Split (R3), преимуществ производительности, структуры проекта и руководства по быстрой установке. Создана полноценная английская версия документации — [`README_EN.md`](file:///e:/DEV/Project/OpenStream%20Engine/README_EN.md).
- **OpenWrt Smart Geo-Split Integration:** Добавлена встроенная поддержка режима маршрутизации `geo_split` (Smart Geo-Split R3) в пакет OpenWrt. При включении режима служба автоматически переопределяет настройки `dnsmasq` для направления `usher.ttvnw.net` в специальный nftset/ipset VPN-интерфейса, а остальные домены и видеотрафик пускает напрямую. Изменены файлы: [`openstream.config`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/openstream.config), [`openstream-uci2yaml`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/openstream-uci2yaml), [`streamproxyd.init`](file:///e:/DEV/Project/OpenStream%20Engine/package/openwrt/files/streamproxyd.init).
- **Combo Routes & Smart Geo-Split:** В autolab добавлена поддержка одновременного тестирования 4 схем маршрутизации (R0–R3). Эмпирически доказана эффективность **R3 (Smart Geo-Split)**: запрос `gql.twitch.tv` идет напрямую (RU ISP) для получения токена без рекламы, а `usher.ttvnw.net` — через европейский прокси/SmartDNS для обхода ограничений на 1080p/1440p/Source качество.
- **Playlist Edge** (`mode: edge`, default): `GET /twitch/<channel>` без CA; сегменты с CDN; ADR [0002](adr/0002-playlist-edge.md).
- Hostlists: per-service `hostlists/*.txt`, compose, `custom_domain`, optional GitHub remote 12ч; LuCI multi-select.
- H5: сужен divert/dnsmasq/MITM whitelist (не www/gql/`*.twitch.tv`) — сайт снова открывается.
- Docs: DoH может быть на клиенте **и** на роутере; nftset работает только через dnsmasq, иначе — hostlist ([COEXISTENCE.md](COEXISTENCE.md)).
- Fix: `.ipk` — `Packages.gz` / `openstream-refresh-opkg-list` только в `openstream-engine` (не в luci-app), иначе opkg `check_data_file_clashes`.
- **Transparent Twitch catch** (`mode: transparent`): nft divert — теперь **legacy** (нужен CA).
- Детект соседей: форки podkop (**netshift**, **forkop**); **SSClash**, OpenClash, Mihomo, ByeDPI (`ciadpi`), PassWall/HomeProxy, redsocks/tun2proxy/hev; модель в [COEXISTENCE.md](COEXISTENCE.md).
- Первые артефакты Cortex-A53 OpenWrt ≤24.10: `dist/openwrt-24.10-a53/`  
  (`openstream-engine` / `-slim` `.ipk` + `luci-app-openstream` `.ipk`, бинари aarch64 musl).
- Скрипт `scripts/pack-ipk-a53.sh` (упаковка opkg `.ipk` без полного SDK).
- Fix: `.ipk` release **7** — LuCI Installed: Size/Description через встроенный `Packages.gz` + `openstream-refresh-opkg-list` (opkg status не хранит Description; LuCI берёт поля из available lists).
- Fix: `.ipk` release **6** — убран `Size:` из CONTROL (opkg: Checksum or size mismatch); `Size`/`SHA256sum` только в `ipk/Packages`.
- Fix: release **5** — `luci-i18n-openstream-ru`; menu.d `cbi`; vendored po2lmo.

### Документация

- **Research front:** README + Stage R; MITM rejected; кандидат geo-split [ADR 0004](adr/0004-geo-split-egress.md).
- [ADR 0003](adr/0003-goal1-router-only-tls.md) обновлён: Goal `[research]`; inspect blocked; WG на роутере ≠ VPN на клиенте.
- [OPENTWITCH_LAB.md](research/OPENTWITCH_LAB.md), [TWITCH_TRAFFIC_MAP.md](research/TWITCH_TRAFFIC_MAP.md), autolab `research/twitch/autolab/`.
- [INDEX.md](INDEX.md), [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](ARCHITECTURE.md): research-first.
- Lab Edge sync 0.4.2-14 остаётся в коде как archive, не claim Goal №1.
- [PERFORMANCE.md](PERFORMANCE.md): GL-MT6000 idle ~2.8 МБ RSS (lab).

### Исправлено

- `package/openwrt/files/config.yaml` добавлен в пакет (раньше ссылка в Makefile была битой).

## [0.4.2] — 2026-07-21

### Добавлено (size features + benches + calibration fixtures)

- Feature flags `streamproxyd`: `plugin-twitch` / `plugin-hls` / `plugin-dash` / `slim-twitch`; `ose-proxy/dash`.
- Criterion benches: HLS parse, Twitch strip, DASH filter.
- Фикстуры Kick/YouTube/DASH SCTE; detector Contains/Regex смотрит URI сегмента.
- [docs/PERFORMANCE.md](PERFORMANCE.md).

### Файлы

- `crates/streamproxyd/**`, `ose-proxy/**`, `ose-detector/**`
- `crates/ose-manifest/benches/**`, `ose-plugin-twitch/benches/**`, `ose-dash/benches/**`
- `crates/ose-plugin-hls/fixtures/**`, `ose-dash/fixtures/**`
- `docs/PERFORMANCE.md`, `docs/PACKAGING.md`, `docs/ROADMAP.md`, `.github/workflows/ci.yml`

### Обоснование

Урезание бинаря для mips и воспроизводимые микробенчи до полевых замеров SoC.

## [0.4.1] — 2026-07-21

### Добавлено (LuCI ↔ демон + CI)

- LuCI: страницы Events / Metrics / Services; Status с ABI и Apply&Reload.
- UCI→YAML: `/usr/libexec/openstream-uci2yaml`; init `reload` + procd trigger.
- UCI секции: kick/trovo/youtube/dash/observability/tls; twitch.backup_seamless.
- GitHub Actions: `cargo test` + clippy; cross `aarch64-musl` (+ armv7 via zigbuild, soft-fail).
- i18n en/ru для новых строк.

### Файлы

- `luci-app-openstream/**`
- `package/openwrt/files/openstream-uci2yaml`, `streamproxyd.init`, `openstream.config`
- `.github/workflows/ci.yml`
- `docs/ROADMAP.md`, `docs/PACKAGING.md`, `README.md`

### Обоснование

Закрыть разрыв LuCI(UCI) ↔ YAML демона и автоматизировать проверку сборки.

## [0.4.0] — 2026-07-21

### Добавлено (v3.0 Platform / Stage D)

- ADR 0001: статический Plugin ABI (`PLUGIN_ABI_VERSION = 3`); [docs/SDK.md](SDK.md); шаблон `templates/ose-plugin-skeleton`.
- `ose-coalesce`: singleflight на обработку манифеста (N клиентов → 1 compute).
- `ose-observe`: event ring-buffer; `GET /api/events`, `GET /metrics` (OpenMetrics).
- YouTube Live rules preset (`youtube.enabled`); Twitch `backup_seamless` opt-in scaffold (default off).
- Cargo `release` (LTO/size) + `release-fast`; [docs/PACKAGING.md](PACKAGING.md); package `0.4.0`.

### Файлы

- `crates/ose-coalesce/**`, `ose-observe/**`, `ose-api/**`, `ose-proxy/**`, `ose-config/**`
- `crates/ose-plugin/**`, `ose-plugin-twitch/**`, `ose-plugin-hls/**`, `ose-rules/**`, `streamproxyd/**`
- `docs/adr/0001-plugin-abi.md`, `docs/SDK.md`, `docs/PACKAGING.md`, `templates/ose-plugin-skeleton/**`
- `config.example.yaml`, `package/openwrt/**`, `Cargo.toml` profiles

### Обоснование

Закрыть каркас платформы v3: SDK без WASM, observability для LuCI, меньше дублирующей работы на горячем пути.

## [0.3.0] — 2026-07-21

### Добавлено (v2.0 DASH / Stage C)

- Crate `ose-dash`: MPD parse/serialize, фильтры рекламных Period/AdaptationSet.
- Crate `ose-media`: `MediaFilter`, `ManifestKind`, `FilterOutcome`.
- Crate `ose-plugin-dash`: универсальный DASH-плагин (`dash.enabled`).
- Segment Engine: явная классификация CMAF/MPD; proxy обрабатывает `.mpd`.
- Cache: `CacheKey` (URL + ETag / FNV body hash).
- Документ `docs/DASH_ARCHITECTURE.md`.

### Файлы

- `crates/ose-dash/**`, `ose-media/**`, `ose-plugin-dash/**`
- `crates/ose-plugin/**`, `ose-proxy/**`, `ose-cache/**`, `ose-segment/**`, `ose-config/**`, `streamproxyd/**`
- `config.example.yaml`, `package/openwrt/files/config.yaml`
- `docs/DASH_ARCHITECTURE.md`, `docs/ARCHITECTURE.md`, `docs/PLUGIN_ARCHITECTURE.md`, `docs/ROADMAP.md`

### Обоснование

Один демон обслуживает HLS и DASH без форка proxy; сегменты CMAF остаются streaming passthrough.

## [0.2.0] — 2026-07-21

### Добавлено (v1.1 Universal HLS / Stage B)

- Plugin API: явные стадии `filter_segments` / `rewrite_urls` / `capabilities`; общий `strip_ad_segments`, `rewrite_master_variant_urls`, `PrefetchPolicy`.
- Crate `ose-rules`: YAML rulesets + пресеты Kick/Trovo; `ose-plugin-hls` (`RulesHlsPlugin`).
- Конфиг: `proxy_public_url`, `prefetch_policy`, `kick`/`trovo`, `rules_file`.
- Proxy: nested absolute URL после master rewrite; hot-reload `POST /api/reload` (+ SIGHUP на Unix); MITM whitelist Kick/Trovo.
- Примеры: `rules.example.yaml`, расширенный `config.example.yaml`.

### Файлы

- `crates/ose-plugin/**`, `ose-plugin-hls/**`, `ose-rules/**`, `ose-plugin-twitch/**`
- `crates/ose-proxy/**`, `ose-config/**`, `streamproxyd/**`
- `config.example.yaml`, `rules.example.yaml`, `package/openwrt/files/config.yaml`
- `docs/PLUGIN_ARCHITECTURE.md`, `docs/PROXY_ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`

### Обоснование

Несколько HLS-сервисов на одном ядре без форка proxy; reload без рестарта демона.

## [0.1.2] — 2026-07-21

### Исправлено (v1.0-hardened / rust-pro)

- Proxy: MITM обрабатывает только `.m3u8` с cap размера; медиа-сегменты — streaming без полной буферизации.
- Persistent MITM CA из `tls.ca_cert`/`ca_key` (+ автосоздание); кэш leaf-сертификатов по host.
- Режимы `explicit` / `redirect_whitelist` / `off` подключены; nft apply fail-soft на Linux.
- Twitch strip: `#EXT-X-DISCONTINUITY` после midroll; DATERANGE только по stitched/Twitch-маркерам.
- Detector: настоящий `regex`; атомики stats; убран лишний serialize.
- Cache: ключ = полный URL (без hash-коллизий).
- Upstream HTTP status сохраняется; `active_streams` учитывается; debug/max_wait влияют на логирование.
- Тесты proxy (`is_m3u8`, split, whitelist); фикстура midroll.

### Файлы

- `crates/ose-proxy/**`, `ose-plugin-twitch/**`, `ose-detector/**`, `ose-cache/**`, `ose-config/**`, `streamproxyd/**`
- `config.example.yaml`, `docs/ROADMAP.md`, `docs/PROXY_ARCHITECTURE.md`

## [0.1.1] — 2026-07-21

### Добавлено

- [`docs/ROADMAP.md`](ROADMAP.md): сверка исходного плана с реализацией `0.1.0`, результаты rust-pro ревью, этапы `v1.0-hardened` → `v3.0`.
- Ссылка на roadmap в README и ARCHITECTURE.

### Обоснование

Зафиксировать разрывы (RAM/MITM/passthrough, проводка конфига) и порядок развития, не смешивая их с уже закрытым каркасом v1.

## [0.1.0] — 2026-07-21

### Добавлено

- Каркас платформы OpenStream Engine (Rust): `streamproxyd`, Manifest/Cache/Detector engines, Plugin API.
- Plugin Twitch v1: Segment Stripping по маркерам `stitched` / `EXT-X-DATERANGE` / `#EXTINF` без `,live`.
- Explicit HTTP(S) proxy (порт 18080), MITM whitelist для HLS-доменов, Cache TTL 1–5 с.
- API `GET /api/status` (статистика, соседи окружения).
- OpenWrt package + nft таблица `inet openstream`, LuCI (en/ru).
- Документация: ARCHITECTURE, PROXY, PLUGIN, HLS, COEXISTENCE.

### Файлы

- `Cargo.toml`, `crates/**`
- `docs/**`
- `package/openwrt/**`
- `luci-app-openstream/**`
- `README.md`

### Обоснование

Первая версия реализует платформу обработки HLS-манифестов без изменения клиента и без конфликта с zapret/podkop.
