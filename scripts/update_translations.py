#!/usr/bin/env python3
import os
import re
import json
import glob
import subprocess
import sys

# Set utf-8 output encoding for windows console
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VIEW_DIR = os.path.join(ROOT_DIR, "luci-app-openstream", "root", "www", "luci-static", "resources", "view", "openstream")
MENU_FILE = os.path.join(ROOT_DIR, "luci-app-openstream", "root", "usr", "share", "luci", "menu.d", "luci-app-openstream.json")
PO_FILE = os.path.join(ROOT_DIR, "luci-app-openstream", "po", "ru", "openstream.po")
LMO_FILE = os.path.join(ROOT_DIR, "luci-app-openstream", "root", "usr", "lib", "lua", "luci", "i18n", "openstream.ru.lmo")
PO2LMO_SCRIPT = os.path.join(ROOT_DIR, "scripts", "po2lmo.py")

# Translation dictionary mapping english keys to russian
TRANSLATIONS = {
    # Menu items
    "OpenStream Engine": "OpenStream Engine",
    "Policy Routing": "Маршрутизация",
    "Servers & Subscriptions": "Серверы и подписки",
    "Live Monitoring": "Мониторинг потоков",
    "Status": "Состояние",
    "Twitch Optimizer": "Оптимизатор Twitch",
    "Services & DNS": "Сервисы и DNS",
    "Updates": "Обновления",
    "Diagnostics": "Диагностика",

    # Common actions & buttons
    "Save & Apply": "Сохранить и применить",
    "Save All Settings": "Сохранить все настройки",
    "Saving...": "Сохранение...",
    "Saving settings...": "Сохранение настроек...",
    "Apply and Reload": "Применить и перезагрузить",
    "Applied successfully!": "Успешно применено!",
    "Reloading…": "Перезагрузка…",
    "Failed to reload": "Ошибка перезагрузки",
    "Loading…": "Загрузка…",
    "Active": "Активен",
    "Standby": "В ожидании",
    "Yes": "Да",
    "No": "Нет",
    "Selected": "Выбран",
    "Select": "Выбрать",
    "Optional": "Опционально",
    "Check Ping": "Проверить пинг",
    "Measuring latency...": "Замер задержки...",
    "Ping": "Пинг",
    "Delete server": "Удалить сервер",
    "Delete server: ": "Удалить сервер: ",
    "Import Nodes": "Импортировать узлы",
    "Downloading & parsing...": "Загрузка и парсинг...",
    "Refresh Now": "Обновить сейчас",
    "Refreshing...": "Обновление...",
    "Clear History": "Очистить историю",
    "Inspect Route": "Проверить маршрут",
    "Restart Service": "Перезапустить службу",
    "Restarting streamproxyd…": "Перезапуск streamproxyd…",
    "Run Diagnostics": "Запустить диагностику",
    "Diagnosing...": "Диагностика...",
    "Check for Updates": "Проверить обновления",
    "Checking for Updates": "Проверка обновлений",
    "Update All (1-Click)": "Обновить всё в 1 клик",
    "Updating Components": "Обновление компонентов",
    "Updating Component": "Обновление компонента",
    "Refresh Console": "Обновить консоль",
    "Save Schedule": "Сохранить расписание",
    "Download Backup (.tar.gz)": "Скачать бэкап (.tar.gz)",
    "Creating archive...": "Создание архива...",
    "Restore from Archive": "Восстановить из архива",
    "Restoring Configuration": "Восстановление конфигурации",
    "Unpacking archive and restarting services...": "Распаковка архива и перезапуск служб...",
    "Add Custom Rule": "Добавить правило",
    "Edit Policy Rule": "Редактировать правило",
    "New Routing Policy Rule": "Новое правило маршрутизации",
    "Save Rule": "Сохранить правило",
    "Cancel": "Отмена",
    "Delete rule": "Удалить правило",
    "Delete rule ": "Удалить правило ",
    "Test Route": "Тест маршрута",
    "Testing…": "Тестирование…",

    # Status View
    "Engine Status & Overview": "Состояние и обзор OpenStream Engine",
    "Real-time operational status of streamproxyd, Zapret2 nfqws2, sing-box TPROXY, and nftables integration.": "Мониторинг состояния streamproxyd, Zapret2 nfqws2, sing-box TPROXY и nftables в реальном времени.",
    "Restart requested successfully!": "Запрос на перезапуск успешно отправлен!",
    "Restart failed: ": "Ошибка перезапуска: ",
    "Active & Healthy": "Работает и активен",
    "Service Offline": "Служба остановлена",
    "DPI Bypass Engine": "Обход DPI (Zapret2)",
    "Bypassing DPI (nfqws2)": "Активен обход блокировок",
    "Not Running": "Не запущен",
    "Proxy Core (sing-box)": "Прокси-ядро (sing-box)",
    "Transparent TPROXY Active": "Прозрачный TPROXY активен",
    "Kernel nftables Rules": "Правила ядра nftables",
    "Chain & Sets Active": "Цепочки и сеты активны",
    "Tables Missing": "Таблицы не загружены",
    "System Architecture & Network Interfaces": "Архитектура системы и сетевые интерфейсы",
    "Video Stream Proxy": "Прокси видеопотоков",
    "Online (Port 8888)": "В сети (Порт 8888)",
    "Transparent HTTP/HTTPS stream rewriter strips server-side inserted advertisements (SSAI) in HLS/DASH playlists without requiring root CA certificates on client devices.": "Прозрачный рерайтер HTTP/HTTPS потоков удаляет рекламу (SSAI) в плейлистах HLS/DASH без установки корневых CA-сертификатов на клиентские устройства.",
    "Anti-Censorship (Zapret2)": "Обход DPI (Zapret2)",
    "Kernel NFQUEUE Engine": "Движок ядра NFQUEUE",
    "L7 packet segmentation, fake SNI injection, and TCP window manipulation for seamless YouTube 4K, Discord voice, and restricted media access.": "Сегментация пакетов на L7, отправка fake SNI и манипуляции с окном TCP для работы YouTube 4K, голосовой связи Discord и заблокированных медиаресурсов.",
    "Outbound Multiplexer (sing-box)": "Мультиплексор исходящих (sing-box)",
    "TPROXY / TUN Gateway": "Шлюз TPROXY / TUN",
    "High-performance routing layer supporting VLESS Reality/xHTTP, Hysteria 2, TUIC v5, Shadowsocks 2022, and URLTest lowest-latency auto-selection.": "Высокопроизводительный уровень маршрутизации с поддержкой VLESS Reality/xHTTP, Hysteria 2, TUIC v5, Shadowsocks 2022 и автовыбором по задержке URLTest.",
    "Firewall & Kernel Acceleration": "Файрвол и ускорение ядра",
    "Zero-Copy Routing": "Маршрутизация без копирования",
    "nftables policy routing integrated directly with fw4 and dnsmasq ipsets/nftsets. DNS requests resolve dynamically without traffic redirection loops.": "Политики nftables интегрированы напрямую с fw4 и dnsmasq nftsets. Запросы разрешаются динамически без петель перенаправления трафика.",

    # Routing View
    "Policy Routing & Rules": "Политика маршрутизации и правил",
    "Declarative routing engine managing Zapret2 DPI bypass, StreamProxy ad-stripping, and sing-box outbounds.": "Декларативный движок маршрутизации: обход DPI Zapret2, очистка потоков StreamProxy и туннели sing-box.",
    "Domain Route Inspector": "Инспектор маршрута домена",
    "Domain routing rules": "Правил маршрутизации доменов",
    "Inspect which engine handles traffic for a specific domain in real time:": "Узнайте в реальном времени, какая подсистема обрабатывает трафик для домена:",
    "All Rules": "Все правила",
    "Streaming & Media": "Стриминг и видео",
    "DPI Bypass (Zapret2)": "Обход DPI (Zapret2)",
    "Proxy (sing-box TPROXY)": "Прокси (sing-box TPROXY)",
    "Privacy & AdBlock": "Приватность и антиреклама",
    "Custom Rules": "Пользовательские правила",
    "No rules matching selected filter.": "Нет правил, соответствующих выбранному фильтру.",
    "Target Domains (Wildcards allowed)": "Целевые домены (поддерживаются маски *)",
    "Target IP / CIDR Ranges (Optional)": "Целевые IP / CIDR диапазоны (Опционально)",
    "Routing Target Action": "Действие маршрутизации",
    "Rule Identifier (ID)": "Идентификатор правила (ID)",
    "Rule Name": "Название правила",
    "One domain per line, e.g.:\nexample.com\n*.cdn.example.org": "Один домен на строку, например:\nexample.com\n*.cdn.example.org",
    "One domain per line, e.g.:\\nexample.com\\n*.cdn.example.org": "Один домен на строку, например:\nexample.com\n*.cdn.example.org",
    "One IP or CIDR per line, e.g.:\n198.51.100.0/24\n203.0.113.5": "Один IP или CIDR на строку, например:\n198.51.100.0/24\n203.0.113.5",
    "One IP or CIDR per line, e.g.:\\n198.51.100.0/24\\n203.0.113.5": "Один IP или CIDR на строку, например:\n198.51.100.0/24\n203.0.113.5",
    "Routing rules applied successfully!": "Правила маршрутизации успешно применены!",
    "Short rule purpose": "Краткое описание правила",
    "Unnamed Rule": "Правило без названия",
    "rules loaded": "правил загружено",
    "domains": "доменов",
    "no domains": "нет доменов",

    # Servers View
    "Outbound Servers & Subscriptions": "Исходящие серверы и подписки",
    "Management of sing-box outbounds: VLESS Reality, Hysteria 2, TUIC, Shadowsocks 2022, Trojan and auto-failover.": "Управление узлами sing-box: VLESS Reality, Hysteria 2, TUIC, Shadowsocks 2022, Trojan и автопереключение.",
    "Outbound Gateway Selector": "Селектор исходящего шлюза",
    "Selector Mode": "Режим выбора шлюза",
    "URLTest (Auto-best latency)": "URLTest (Автовыбор наименьшей задержки)",
    "Manual Selection": "Фиксированный сервер (Вручную)",
    "URLTest automatically chooses the lowest-latency outbound; Manual locks to a specific server.": "URLTest автоматически переключается на узел с наименьшим пингом; Фиксированный режим жестко закрепляет сервер.",
    "Active Outbound Node": "Текущий активный узел",
    "Automatically determined based on lowest ping": "Определяется автоматически по наименьшему пингу",
    "Traffic routes through this selected server": "Трафик направляется через выбранный сервер",
    "No servers available": "Нет доступных серверов",
    "Auto (Lowest Latency)": "Автовыбор (Наименьший пинг)",
    "Import Subscriptions & Nodes": "Импорт подписок и узлов",
    "Subscription URL, Base64 or Node URI": "Ссылка на подписку, Base64 или URI узла",
    "Paste https:// subscription link, Base64 block, or vless://, hysteria2://, tuic://, ss://, trojan:// URI": "Вставьте https:// ссылку на подписку, блок Base64 или vless://, hysteria2://, tuic://, ss://, trojan:// ссылки",
    "User-Agent Header (Default: ClashMeta)": "Заголовок User-Agent (По умолчанию: ClashMeta)",
    "Hardware ID (X-HWID for Private Providers)": "Аппаратный ID (X-HWID для приватных подписок)",
    "Optional: e.g. 64-char hex hardware ID": "Опционально: 64-значный hex HWID",
    "Subscription or node link field is empty.": "Поле ссылки на подписку или узел пусто.",
    "Nodes imported successfully!": "Узлы успешно импортированы!",
    "Import failed: ": "Ошибка импорта: ",
    "Import error: ": "Сбой при импорте: ",
    "Configured Servers": "Настроенные серверы",
    "nodes": "узлов",
    "No servers added yet": "Серверы ещё не добавлены",
    "Paste a subscription link or node URI above to import outbounds.": "Вставьте ссылку на подписку или URI узла выше для импорта серверов.",
    "Untested": "Не проверен",
    "Server latency test completed!": "Замер задержки серверов успешно выполнен!",
    "Ping test failed: ": "Ошибка проверки пинга: ",
    "Server list and outbounds successfully applied!": "Список серверов и конфигурация sing-box успешно применены!",
    "RPC Error: ": "Ошибка RPC: ",

    # Services View
    "Network Services & Security": "Сетевые сервисы и безопасность",
    "Configure cascading DNS failover, DoH bootstrap, QUIC/DoH blocking, BitTorrent WAN bypass, and system backup.": "Настройка каскадного DNS, Bootstrap DoH, блокировки QUIC/DoH, прямого пропуска BitTorrent и резервных копий.",
    "Multi-DNS Failover & Bootstrap Resolver": "Отказоустойчивый Multi-DNS и Bootstrap",
    "Resilient DNS": "Отказоустойчивый DNS",
    "Upstream DNS Servers (DoH / DoT / UDP - one per line)": "Основные DNS-серверы (DoH / DoT / UDP - по одному на строку)",
    "Cascading upstream resolvers queried through the secure engine.": "Каскадные апстрим-резолверы, опрашиваемые безопасным движком.",
    "Bootstrap DNS (Static plain IPs for resolving DoH endpoints)": "Bootstrap DNS (Статические IP без шифрования для DoH)",
    "Plain IP resolvers used to bootstrap initial DoH server domain names.": "Прямые IP DNS-серверов для разрешения доменных имён DoH при старте.",
    "Failover Failure Threshold": "Порог сбоев для Failover (от флаппинга)",
    "Number of consecutive failed queries before switching to secondary DNS (anti-flapping).": "Количество неудачных запросов подряд перед переключением на запасной DNS.",
    "Primary Recovery Interval (seconds)": "Интервал возврата к Primary DNS (секунды)",
    "Time to wait before probing and returning to the primary DNS server.": "Время ожидания перед проверкой и возвратом к основному DNS-серверу.",
    "mTLS DoH: Client Certificate (PEM Path)": "mTLS DoH: Клиентский сертификат (PEM путь)",
    "Optional client certificate for mutual TLS DoH resolvers.": "Опциональный клиентский сертификат для DoH-серверов с взаимной TLS-аутентификацией.",
    "mTLS DoH: Client Private Key (PEM Path)": "mTLS DoH: Приватный ключ клиента (PEM путь)",
    "Optional private key matching the client certificate.": "Приватный ключ, соответствующий клиентскому сертификату.",
    "Local DNS Overrides (Hosts format: domain IP)": "Локальные DNS-переопределения (формат hosts: домен IP)",
    "Static domain mappings evaluated before querying upstream DNS.": "Статические сопоставления доменов, проверяемые до обращения к внешним DNS.",
    "Automatic DNS Failover": "Автопереключение при сбоях DNS (Failover)",
    "Prefer IPv4 for DNS Queries": "Приоритет IPv4 при DNS-запросах",
    "Network Security & Bypass Rules": "Сетевая безопасность и правила пропуска",
    "nftables Kernel Rules": "Правила ядра nftables",
    "BitTorrent WAN Bypass": "Прямой пропуск BitTorrent в WAN",
    "Ports 6881–6889 & 51413 bypass VPN directly to WAN, preventing VPN provider bans.": "Порты 6881–6889 и 51413 направляются в WAN напрямую, предотвращая блокировки хостеров.",
    "Block QUIC (UDP 443)": "Блокировать QUIC (UDP 443)",
    "Forces TLS 1.3 TCP fallback in browsers, essential for YouTube DPI bypass in Zapret2.": "Форсирует TCP TLS 1.3 в браузерах. Критически важно для работы обхода замедления YouTube в Zapret2.",
    "Block Unauthorized DoH/DoT": "Блокировать сторонний DoH/DoT",
    "Blocks rogue browser DoH to prevent DNS leaks and enforce router policy routing.": "Предотвращает утечки DNS через встроенные резолверы браузеров мимо политик роутера.",
    "Exclude NTP (UDP 123)": "Исключить синхронизацию времени NTP",
    "Direct WAN routing for time synchronization without tunnel jitter.": "Прямой пропуск пакетов времени без задержек и джиттера туннелей.",
    "Download Updates via VPN": "Скачивать обновления через VPN",
    "Use active sing-box tunnel when downloading GitHub release packages if blocked.": "Использовать туннель sing-box для скачивания релизов GitHub при региональных блокировках.",
    "Excluded Subnets and IPs (CIDR list, comma-separated)": "Исключенные подсети и IP (CIDR список через запятую)",
    "IP prefixes that are always bypassed directly to WAN without proxying.": "IP-префиксы, которые всегда идут напрямую в WAN без обработки прокси.",
    "Excluded LAN Clients (IP or MAC, comma-separated)": "Исключенные клиенты LAN (IP или MAC через запятую)",
    "Specific local devices completely excluded from all proxying.": "Устройства локальной сети, полностью исключенные из всех правил маршрутизации.",
    "Backup & Restore": "Резервное копирование и восстановление",
    "Archive .tar.gz": "Архив .tar.gz",
    "Export all custom routing rules, UCI configuration, and server credentials to a portable archive, or restore on a freshly flashed router.": "Экспорт всех правил, конфигураций UCI и серверов в переносимый архив tar.gz или восстановление при перепрошивке.",
    "Backup archive downloaded successfully!": "Архив резервной копии успешно скачан!",
    "Backup error: ": "Ошибка создания бэкапа: ",
    "Configuration restored successfully!": "Конфигурация успешно восстановлена!",
    "Restore error: ": "Ошибка восстановления: ",
    "DNS and network security settings applied successfully!": "Параметры DNS и сетевой безопасности успешно применены!",

    # Twitch View
    "Twitch Live Stream Optimizer": "Оптимизатор прямых трансляций Twitch",
    "Server-side token split routing and SSAI ad-stripping without installing client SSL root certificates (Zero-CA).": "Серверное разделение токенов и удаление рекламы SSAI без установки SSL-сертификатов на клиентские устройства (Zero-CA).",
    "Optimizer Mode & Routing Presets": "Режим оптимизатора и готовые сценарии",
    "Module Enabled": "Модуль включен",
    "Operational Preset": "Рабочий сценарий",
    "Geo-Split via Clean-Proxy (Zero-CA for SmartTV/PC) [Recommended]": "Geo-Split через Clean-Proxy (Zero-CA для SmartTV/ПК) [Рекомендуется]",
    "Playlist Edge: Local SSAI ad-stripping in streamproxyd (:18080)": "Playlist Edge: Локальное удаление SSAI-рекламы в streamproxyd (:18080)",
    "Quality Unlock: 1080p60 unthrottled via SmartDNS": "Quality Unlock: Разблокировка 1080p60 без ограничений через SmartDNS",
    "Custom: Fine-grained matrix split routing": "Пользовательский: Раздельная матричная маршрутизация",
    "Disabled": "Отключено",
    "Geo-Split requests stream playback access tokens from an ad-free region (e.g. Ukraine, Kazakhstan, Albania) while delivering raw 1080p video chunks directly from your local ISP CDN.": "Geo-Split запрашивает авторизационный токен в стране без рекламы (Украина, Казахстан, Албания), а 1080p-видеопоток транслирует напрямую с прямого CDN вашего провайдера.",
    "Matrix Routing Destinations": "Направления матричной маршрутизации",
    "Access Token Route (gql.twitch.tv)": "Маршрут токена доступа (gql.twitch.tv)",
    "Controls region-based SSAI ad-injection rules.": "Управляет региональными правилами внедрения рекламы SSAI.",
    "Master Playlist Route (usher.ttvnw.net)": "Маршрут мастер-плейлиста (usher.ttvnw.net)",
    "Controls resolution availability and transcoder endpoints.": "Управляет доступностью разрешений (1080p60) и эндоинтами транскодеров.",
    "Video Stream CDN Route (live-video.net)": "Маршрут CDN видеопотока (live-video.net)",
    "Direct WAN (Max ISP Bandwidth)": "Прямой WAN (Максимальная скорость провайдера)",
    "Transports multi-gigabyte video segments.": "Передаёт многогигабайтные видеофрагменты.",
    "Zero-CA Architectural Blueprint": "Архитектурная концепция Zero-CA",
    "Client Certificate Free": "Без сертификатов на клиентах",
    "SmartTV, Consoles & Mobile": "SmartTV, игровые консоли и смартфоны",
    "Works out of the box on Apple TV, Android TV, LG webOS, Samsung Tizen, PlayStation, Xbox, iOS and Android without modifying device certificates or running mitmproxy.": "Работает из коробки на Apple TV, Android TV, LG webOS, Samsung Tizen, PlayStation, Xbox, iOS и Android без установки сертификатов и без mitmproxy.",
    "Bandwidth Efficiency": "Экономия пропускной способности",
    "Heavy video traffic (~8-12 Mbps per stream) never overloads your VPN tunnel. Only lightweight JSON token handshakes (~2 KB) are routed through geo-split outbounds.": "Тяжелый видеопоток (~8–12 Мбит/с на стрим) не нагружает ваш VPN. Через удаленный туннель проходят только крошечные JSON-запросы токенов (~2 КБ).",
    "Twitch Optimizer configuration saved and applied!": "Конфигурация оптимизатора Twitch сохранена и применена!",

    # Monitor View
    "Live Traffic & Routing Monitor": "Живой мониторинг трафика и маршрутов",
    "Real-time inspection of active network flows, core engine classification, and client traffic mapping.": "Отслеживание активных сетевых соединений, секций ядра и сопоставления клиентов в реальном времени.",
    "Flow Distribution Metrics": "Метрики распределения потоков",
    "Engine Active": "Движок активен",
    "Live Domain Routing Inspector": "Живой инспектор маршрутов доменов",
    "Query the active routing table in real-time to inspect which core subsystem will handle traffic for a specific domain:": "Мгновенная проверка таблицы маршрутизации: через какую подсистему ядра пойдёт трафик для домена:",
    "Active Flow Classifications": "Классификация активных потоков",
    "flows": "потоков",
    "All Subsystems": "Все подсистемы",
    "Zapret2 (nfqws2)": "Zapret2 (nfqws2)",
    "StreamProxy (:18080)": "StreamProxy (:18080)",
    "sing-box / VPN": "sing-box / VPN",
    "Sinkhole (Block)": "DNS Sinkhole (Блокировка)",
    "Total Tracked Flows": "Всего отслежено потоков",
    "sing-box Outbound": "Исходящий sing-box",
    "DNS Sinkhole Block": "DNS Sinkhole Блокировка",
    "No flows matched the current filter": "Потоков по выбранному фильтру не обнаружено",
    "Traffic in this category currently passes through standard default routes.": "Трафик для этой категории сейчас направляется через стандартные маршруты по умолчанию.",
    "Matched routing rule": "Сработало правило маршрутизации",
    "Applied to all LAN clients": "Применяется ко всем клиентам LAN",
    "Flow monitor history cleared.": "История мониторинга потоков очищена.",

    # Updates View
    "Component & Update Center": "Центр компонентов и обновлений",
    "Manage OSE core services, sing-box flavor variants, Zapret2 DPI rules, and automated cron syncing.": "Управление службами OSE, вариантами sing-box, правилами Zapret2 и автоматической синхронизацией по cron.",
    "sing-box Binary Flavor Selector": "Выбор сборки sing-box (Flavors)",
    "Installed: ": "Установлена: ",
    "Not Installed": "Не установлен",
    "Choose the sing-box build tailored to your router hardware constraints (RAM, flash storage, and protocol features):": "Выберите сборку sing-box под аппаратные возможности вашего роутера (RAM, флеш-память и протоколы):",
    "Official standard feed build. Rock-solid Shadowsocks, VLESS, WireGuard and DNS routing.": "Официальная стабильная сборка из фидов OpenWrt. Надежные протоколы Shadowsocks, VLESS, WireGuard и DNS-маршрутизация.",
    "Full protocol suite: VLESS Reality, xHTTP transport, TUIC v5, Shadowsocks 2022 and state-of-the-art anti-censorship.": "Полный набор современных протоколов: VLESS Reality, транспорт xHTTP, TUIC v5, Shadowsocks 2022 и новейшие методы обхода цензуры.",
    "Stripped minimal build (<15 MB RAM usage). Optimized for entry-level routers with 64–128 MB RAM.": "Максимально облегченная сборка (потребление <15 МБ RAM). Оптимально для роутеров начального уровня с 64–128 МБ RAM.",
    "UPX-compressed Extended build. Provides all modern protocol features while saving 65% flash storage.": "UPX-сжатый вариант Extended. Все современные протоколы с экономией 65% дискового пространства.",
    "Switch to Stable": "Переключить на Stable",
    "Switch to Extended": "Переключить на Extended",
    "Switch to Tiny": "Переключить на Tiny",
    "Switch to Compressed": "Переключить на Compressed",
    "Switched to Stable variant.": "Выбран вариант Stable.",
    "Switched to Extended variant.": "Выбран вариант Extended (xHTTP).",
    "Switched to Tiny variant.": "Выбран вариант Tiny.",
    "Switched to Compressed variant.": "Выбран вариант Extended Compress.",
    "Automated Sync & Cron Schedule": "Автоматическая синхронизация по расписанию (Cron)",
    "Enable Automated Cron": "Включить автообновление cron",
    "Executes periodic syncing in the background (/etc/crontabs/root).": "Фоновое выполнение обновлений через системный cron (/etc/crontabs/root).",
    "Sync Interval": "Интервал синхронизации",
    "Daily (at 04:00 AM)": "Ежедневно (в 04:00 утра)",
    "Every 3 Days": "Каждые 3 дня",
    "Weekly (Sunday at 04:00 AM)": "Еженедельно (Воскресенье, 04:00 утра)",
    "Low-traffic hours recommended for transparent updates.": "Рекомендуются часы минимальной нагрузки сети для бесшовного обновления.",
    "Synchronized Modules": "Синхронизируемые модули",
    "Community Domain & GeoIP Lists": "Списки доменов сообщества и GeoIP",
    "sing-box Core Binary": "Бинарник sing-box Core",
    "Zapret2 (nfqws2) Binary & Strategy": "Бинарник и стратегии Zapret2 (nfqws2)",
    "Automated update schedule saved successfully!": "Расписание автообновления успешно сохранено!",
    "Failed to save schedule: ": "Не удалось сохранить расписание: ",
    "Installed Subsystem Status": "Статус установленных подсистем",
    "packages": "пакетов",
    "Update: ": "Доступно: ",
    "Up to Date": "Актуально",
    "Installed Version: ": "Установленная версия: ",
    "Upgrade to ": "Обновить до ",
    "Recheck": "Проверить",
    "Update completed.": "Обновление выполнено.",
    "Update failed": "Сбой обновления",
    "Update failed: ": "Ошибка обновления: ",
    "Automated updates are disabled in this binary release. Run \"opkg update && opkg upgrade\" in terminal.": "Автоматическое обновление отключено в данном релизе. Выполните 'opkg update && opkg upgrade' в терминале.",
    "Automated updates disabled. Upgrade via terminal: opkg update && opkg upgrade ": "Автообновление отключено. Обновите вручную в консоли: opkg update && opkg upgrade ",
    "All components updated successfully.": "Все компоненты успешно обновлены.",
    "Update Execution Console": "Консоль процесса обновления",
    "No update activity logged.": "Журнал обновлений пуст.",
    "Update check complete. Versions refreshed.": "Проверка завершена. Данные о версиях обновлены.",

    # Diagnostics View
    "Self-Diagnostics & Health Hub": "Самодиагностика и проверка здоровья системы",
    "Comprehensive automated verification of nftables chains, sing-box sockets, Zapret2 DPI queues, and DNS integrity.": "Комплексная автоматическая проверка цепочек nftables, сокетов sing-box, очередей Zapret2 и целостности DNS.",
    "Diagnostic Checks": "Диагностические проверки",
    "checks": "проверок",
    "No diagnostic results yet": "Диагностика ещё не проводилась",
    "Click \"Run Diagnostics\" to inspect system components.": "Нажмите «Запустить диагностику» для проверки компонентов роутера.",
    "HEALTHY": "В НОРМЕ",
    "WARNING": "ВНИМАНИЕ",
    "FAIL": "ОШИБКА",
    "Self-diagnostics completed!": "Самодиагностика системы успешно завершена!",
    "Diagnostics error: ": "Ошибка выполнения диагностики: ",

    # Additional UI strings
    "Kernel-level fwmark packet dispatching via Linux nftables & IP rule sets routes targeted traffic across Direct, VPN, Anti-DPI, and Proxy egress interfaces.": "Диспетчеризация пакетов на уровне ядра через fwmark в Linux nftables и таблицы IP route направляет трафик между WAN, VPN, Anti-DPI и Proxy.",
    "Orchestration Layer": "Уровень оркестрации",
    "Stopped": "Остановлена",
    "Submitting update request...": "Отправка запроса на обновление...",
    "● Service Running": "● Служба работает",
    "Active Declarative Rules": "Активные декларативные правила",
    "Failed to apply rules: ": "Ошибка применения правил: ",
    "Inactive": "Неактивен",
    "Failed to restart service: ": "Не удалось перезапустить службу: ",
    "Multi-DNS & DoH Shield": "Multi-DNS и защита DoH",
    "Apply Changes": "Применить изменения",
    "Declarative Policy Routing": "Декларативная маршрутизация",
    "Submitting update request for ": "Запрос на обновление для ",
    "Unknown error": "Неизвестная ошибка",
    "Idempotent table management preserving dynamic sets across reloads with mutual conflict isolation alongside Forkop, Podkop, and Passwall.": "Идемпотентное управление таблицами с сохранением сетов при перезагрузках и изоляцией конфликтов с Forkop, Podkop и Passwall.",
    "Display Name": "Отображаемое имя",
    "Installed": "Установлен",
    "Anti-DPI (Zapret2 NFQUEUE)": "Обход DPI (Zapret2 NFQUEUE)",
    "Automatic domain population into nftables dynamic sets with failover resolvers and automated DNS leak mitigation.": "Автоматическое добавление доменов в динамические сеты nftables через dnsmasq с защитой от утечек DNS.",
    "Ecosystem Coexistence": "Совместимость в экосистеме",
    "Declarative routing policy": "Декларативная политика маршрутизации",
    "Anti-DPI (Zapret2 nfqws2)": "Обход DPI (Zapret2 nfqws2)",
    "○ Service Stopped": "○ Служба остановлена",
    "Custom": "Пользовательский",
    "e.g. My Custom Service": "например, Мой сервис",
    "Direct WAN (Provider Default)": "Прямой WAN (По умолчанию)",
    "HLS/DASH Stream Proxy (Port 8888)": "StreamProxy HLS/DASH (Порт 8888)",
    "Querying remote repositories and package index...": "Опрос удаленных репозиториев и индекса пакетов...",
    "Create Routing Rule": "Создать правило маршрутизации",
    "Ready": "Готов",
    "Description": "Описание",
    "Real-time Domain Route Inspector": "Инспектор маршрута домена в реальном времени",
    "Edit": "Редактировать",
    "Core Daemon (Rust)": "Основной демон (Rust)",
    "Block (Reject/Drop)": "Блокировка (Reject/Drop)",
    "Default WAN": "Основной WAN",
    "Manage declarative traffic rules (.osrule.yaml), test domain resolution, and configure client policy routing.": "Управление правилами маршрутизации (.osrule.yaml), проверка разрешения доменов и настройка клиентских политик.",
    "Update error: ": "Ошибка обновления: ",
    "Applying routing rules…": "Применение правил маршрутизации…",
    "Primary VPN (Europe)": "Основной VPN (Европа)",
    "All": "Все",
    "Direct WAN (Bypass)": "Прямой WAN (Прямой интернет)",
    "Domains will be automatically populated into nftables dynamic target sets via dnsmasq.": "Домены будут автоматически добавлены в динамические сеты nftables через dnsmasq.",
    "Save error: ": "Ошибка сохранения: ",
    "NFQUEUE 1088": "Очередь NFQUEUE 1088",
    "Are you sure you want to delete this rule?": "Вы уверены, что хотите удалить это правило?",
    "Error: ": "Ошибка: ",
    "Edit Routing Rule": "Редактировать правило маршрутизации",
    "Universal Cross-Platform Traffic Orchestrator & Declarative Policy Routing Engine": "Универсальный оркестратор сетевого трафика и декларативный движок маршрутизации",
    "SmartDNS Comss.one (1080p60 Unlock)": "SmartDNS Comss.one (1080p60)"
}

def load_po():
    entries = {}
    if os.path.exists(PO_FILE):
        with open(PO_FILE, "r", encoding="utf-8") as f:
            content = f.read()
        blocks = re.split(r'\n\s*\n', content)
        for b in blocks:
            msgid_match = re.search(r'msgid\s+(".*?"(?:\s*\n\s*".*?")*)', b)
            msgstr_match = re.search(r'msgstr\s+(".*?"(?:\s*\n\s*".*?")*)', b)
            if msgid_match and msgstr_match:
                def clean(s):
                    parts = re.findall(r'"(.*?)"', s)
                    res = "".join(parts)
                    res = res.replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"').replace('\\\\', '\\')
                    return res
                k = clean(msgid_match.group(1))
                v = clean(msgstr_match.group(1))
                if k:
                    entries[k] = v
    return entries

def save_po(entries):
    with open(PO_FILE, "w", encoding="utf-8", newline="\n") as f:
        f.write('msgid ""\nmsgstr "Content-Type: text/plain; charset=UTF-8\\n"\n\n')
        for k in sorted(entries.keys()):
            if not k:
                continue
            v = entries[k]
            # Escape strings
            k_esc = k.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n')
            v_esc = v.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n')
            f.write(f'msgid "{k_esc}"\n')
            f.write(f'msgstr "{v_esc}"\n\n')

def main():
    print("Collecting translatable strings...")
    collected = set()

    # JS files
    pattern = re.compile(r"_\(\s*['\"]([^'\"]+)['\"]\s*\)")
    for jf in glob.glob(os.path.join(VIEW_DIR, "*.js")):
        with open(jf, "r", encoding="utf-8") as f:
            c = f.read()
        for m in pattern.findall(c):
            collected.add(m)

    # Menu file
    if os.path.exists(MENU_FILE):
        with open(MENU_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)
        for _, v in data.items():
            if "title" in v:
                collected.add(v["title"])

    print(f"Total collected unique keys: {len(collected)}")

    entries = load_po()
    print(f"Existing PO entries: {len(entries)}")

    # Update with manual translations dictionary
    for k, v in TRANSLATIONS.items():
        entries[k] = v

    # Check for missing
    missing = []
    for k in collected:
        if k not in entries or not entries[k]:
            missing.append(k)

    if missing:
        print(f"WARNING: {len(missing)} strings still missing translation:")
        for m in missing:
            print(f"  - {repr(m)}")
            # Default to key if untranslated
            entries[m] = m
    else:
        print("ALL collected strings have Russian translations!")

    save_po(entries)
    print(f"Updated {PO_FILE} with {len(entries)} entries.")

    # Compile LMO
    print(f"Compiling {LMO_FILE}...")
    os.makedirs(os.path.dirname(LMO_FILE), exist_ok=True)
    subprocess.run([sys.executable, PO2LMO_SCRIPT, PO_FILE, LMO_FILE], check=True)
    print("✓ Successfully updated and compiled translations!")

if __name__ == "__main__":
    main()
