# Changelog

## [Unreleased]

### Документация

- [ROADMAP.md](ROADMAP.md): сверка с 0.4.2; легенда `[x]` / `[~]` / `[ ]` / `[blocked]`; Stage E (поле/perf), F (production plugins), G (opt-in seamless).
- [BUILD_OPENWRT.md](BUILD_OPENWRT.md): сборка `openstream-engine` + `luci-app-openstream` для Cortex-A53 (`.ipk` / `.apk`); `scripts/build-a53.sh`; Makefile принимает `OPENSTREAM_BIN`.

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
