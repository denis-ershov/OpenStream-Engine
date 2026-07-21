# OpenStream Engine SDK

Руководство для авторов плагинов (v3 / ABI `PLUGIN_ABI_VERSION = 3`).

## Модель

См. [ADR 0001](adr/0001-plugin-abi.md): **статическая линковка**, не WASM и не `.so`.

1. Создайте crate по шаблону [`templates/ose-plugin-skeleton`](../templates/ose-plugin-skeleton/).
2. Реализуйте `ose_plugin::Plugin` (+ опционально `ose_media::MediaFilter`).
3. Зарегистрируйте `Arc<dyn Plugin>` в `streamproxyd` `build_plugins`.
4. Пересоберите пакет OpenWrt / release binary.

## Минимальный контракт

```rust
use async_trait::async_trait;
use ose_plugin::{Plugin, PluginStats, ProcessOutcome, RequestMeta, PluginError};

pub struct HelloPlugin;

#[async_trait]
impl Plugin for HelloPlugin {
    fn name(&self) -> &str { "hello" }
    fn match_request(&self, req: &RequestMeta) -> bool {
        req.is_manifest && req.host.contains("example.com")
    }
    fn filter_segments(
        &self,
        _manifest: &mut ose_manifest::Manifest,
        _meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError> {
        Ok(ProcessOutcome::default())
    }
    fn stats(&self) -> PluginStats { PluginStats::default() }
}
```

Проверьте `ose_plugin::PLUGIN_ABI_VERSION` при обновлении зависимости.

## HLS vs DASH

| Kind | Методы |
|------|--------|
| HLS (`.m3u8`) | `filter_segments`, `rewrite_urls`, `process_manifest` |
| DASH (`.mpd`) | `filter_dash`, `process_mpd` |

Helpers: `strip_ad_segments`, `rewrite_master_variant_urls`, `ose_dash::filter_ad_nodes`.

## Rules без кода

Для простых CDN достаточно YAML (`ose-rules` + `ose-plugin-hls`), без нового crate.

## Observability

Плагин не пишет в ring напрямую — ядро публикует `EngineEvent` после strip.
Метрики: `GET /metrics` (OpenMetrics), события: `GET /api/events`.

## Opt-in advanced

Twitch seamless backup / token switching — **только** при `twitch.backup_seamless: true` (scaffold; default off). Strip остаётся default UX.
