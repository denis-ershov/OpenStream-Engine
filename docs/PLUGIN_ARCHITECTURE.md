# Plugin Architecture

Оглавление: [INDEX.md](INDEX.md). SDK: [SDK.md](SDK.md). ABI: [adr/0001-plugin-abi.md](adr/0001-plugin-abi.md).

## Интерфейс

Плагины реализуют trait `Plugin` в crate `ose-plugin` (`PLUGIN_ABI_VERSION = 3`):

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn match_request(&self, req: &RequestMeta) -> bool;
    fn capabilities(&self) -> PluginCapabilities;

    fn filter_segments(
        &self,
        manifest: &mut Manifest,
        meta: &RequestMeta,
    ) -> Result<ProcessOutcome, PluginError>;

    fn rewrite_urls(
        &self,
        manifest: &mut Manifest,
        meta: &RequestMeta,
    ) -> Result<(), PluginError>;

    /// По умолчанию: filter_segments → rewrite_urls.
    async fn process_manifest(
        &self,
        manifest: Manifest,
        meta: &RequestMeta,
    ) -> Result<(Manifest, ProcessOutcome), PluginError>;

    fn stats(&self) -> PluginStats;
}
```

`RequestMeta.proxy_base` — база для rewrite master (из `proxy_public_url` или Host Edge-запроса).

## Стадии обработки

1. **Parse** — ядро (`ose-manifest`).
2. **Detect + Filter** — `filter_segments` (helper `strip_ad_segments`; Twitch — только `PlaylistKind::Media`).
3. **Rewrite** — `rewrite_urls` / `rewrite_master_variant_urls`.
4. **Prefetch policy** — ядро (`keep` / `strip_all` / `strip_when_ads_removed`).

## Подключение

- Статическая линковка в `streamproxyd`.
- Регистрация: Twitch (`ose-plugin-twitch`), Kick/Trovo/generic (`ose-plugin-hls` + `ose-rules`), DASH (`ose-plugin-dash`).
- Hot-reload: `POST /api/reload` (+ SIGHUP на Unix).

## Rule engine (`ose-rules`)

YAML: `hosts` + detector `rules` (`contains` / `date_range` / `ext_inf_not_live` / `regex`).  
Пресеты: `kick_default()`, `trovo_default()`. Пример: `rules.example.yaml`.

## Plugin Twitch

- Segment Stripping на **media** + master rewrite при `proxy_base`.
- Host match: `ttvnw.net`, `twitch.tv`, `video-weaver`, `video-edge`, `playlist.ttvnw`, …
- Без GraphQL в плагине: token/usher — слой `ose-proxy` Edge.
- Opt-in scaffold `backup_seamless` (Stage G) — по умолчанию выключен.

## Plugin HLS (Kick / Trovo / custom)

- `RulesHlsPlugin` поверх `RuleSet`.
- Включение: `kick.enabled` / `trovo.enabled` / `rules_file`.

## Plugin DASH

- Match: `.mpd` (`ManifestKind::Dash`).
- Удаление рекламных Period / AdaptationSet.
- CMAF сегменты не трогаются.

См. [DASH_ARCHITECTURE.md](DASH_ARCHITECTURE.md), [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md).
