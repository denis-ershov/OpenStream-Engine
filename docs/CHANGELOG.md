# Changelog

## [Unreleased]

### Исправлено

- **CI Cross-Compilation & Rust Toolchain Fix:** В `.github/workflows/ci.yml` добавлен экшн `goto-bus-stop/setup-zig@v2` и переведена сборка `aarch64-unknown-linux-musl` на `cargo-zigbuild`. Это устраняет ошибки отсутствия `aarch64-linux-musl-gcc` (для сборки C-кода `ring`) и отсутствие `zig` для armv7. В `rust-toolchain.toml` параметр `channel` установлен в `stable`.
- **Autolab GQL & Classification:** Исправлен GQL-запрос токена Twitch (удалены неиспользуемые переменные `$vodID` и `$isVod`, вызывавшие ошибку валидации схемы на стороне Twitch). Исправлена классификация трафика: сегменты видео (`.ts` на хостах `live-video.net`) теперь корректно детектируются как `segment`, а не `playlist`.
- **r14 — Edge без strip:** master без `proxy_public_url` не rewrite'ился → player брал media с CDN; strip только на media. Auto `proxy_base` из Host + warn.
- **r13 — Edge TLS panic:** rustls 0.23 без CryptoProvider → panic на GQL/usher (`panic=abort` валит демон). Фикс: `ring::install_default()` в `streamproxyd` + `ose-proxy`.
- **Permission denied (exit 126):** бинарь без `+x` после pack с Windows — postinst `chmod 0755`, `fix-ipk-exec-bits.py`; старые `.ipk` удаляются при сборке.

### Добавлено

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
