# Plugin Architecture

## Интерфейс

Плагины реализуют trait `Plugin` в crate `ose-plugin`:

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

`RequestMeta` включает опциональный `proxy_base` для rewrite master variants.

## Стадии обработки

1. **Parse** — ядро (`ose-manifest`).
2. **Detect + Filter** — `filter_segments` (общий helper `strip_ad_segments`).
3. **Rewrite** — `rewrite_urls` / `rewrite_master_variant_urls`.
4. **Prefetch policy** — ядро (`apply_prefetch_policy`: keep / strip_all / strip_when_ads_removed).

## Подключение

- Статическая линковка в `streamproxyd`.
- Регистрация: Twitch (`ose-plugin-twitch`), Kick/Trovo/generic (`ose-plugin-hls` + `ose-rules`).
- Hot-reload: `POST /api/reload` (+ SIGHUP на Unix) перечитывает YAML и пересобирает список плагинов.

## Rule engine (`ose-rules`)

YAML rulesets: `hosts` + detector `rules` (`contains` / `date_range` / `ext_inf_not_live` / `regex`).
Пресеты: `kick_default()`, `trovo_default()`. Пример: `rules.example.yaml`.

## Plugin Twitch

- Segment Stripping + master rewrite при заданном `proxy_public_url`.
- Без GraphQL, token switching, embed player.
- Host: `ttvnw.net`, `video-weaver`, `video-edge`, …

## Plugin HLS (Kick / Trovo / custom)

- `RulesHlsPlugin` поверх `RuleSet`.
- Включение: `kick.enabled` / `trovo.enabled` / `rules_file` в конфиге.

## Plugin DASH

- Match: `.mpd` (`ManifestKind::Dash`).
- Удаление рекламных Period / AdaptationSet по `DashFilterRules`.
- CMAF сегменты не трогаются (streaming passthrough).

См. [DASH_ARCHITECTURE.md](DASH_ARCHITECTURE.md), [HLS_ARCHITECTURE.md](HLS_ARCHITECTURE.md).
