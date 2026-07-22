# ADR 0001: Plugin distribution model (v3 ABI)

- **Status:** Accepted
- **Date:** 2026-07-21
- **Deciders:** OpenStream Engine maintainers

## Context

Нужен SDK для сторонних и встроенных плагинов (Twitch, Kick, Trovo, YouTube, DASH). Варианты:

1. Динамические `.so` / `cdylib` на устройстве
2. WASM plugins (wasmtime / wasmi)
3. Compile-time статическая линковка + versioned Rust trait ABI

Ограничения OpenWrt: musl, RAM ≤ ~30 МБ рядом с zapret/podkop, отсутствие удобного rustc на роутере, размер бинаря.

## Decision

**Выбираем (3): статическая линковка + versioned Plugin ABI (`PLUGIN_ABI_VERSION`).**

- Плагины — Rust crates в workspace / out-of-tree crate, линкуются в `streamproxyd` при сборке.
- Hot-reload = перечитывание YAML + rebuild in-process plugin list (уже есть `/api/reload`), не смена `.so`.
- WASM / cdylib — **отложены** (ADR follow-up), пока не доказана польза без роста RSS/атак-поверхности.

## Consequences

- Плюсы: предсказуемый размер, нет JIT/loader, простой audit, работает на musl.
- Минусы: новый плагин = пересборка пакета (или static feature flags в feed).
- ABI: константа `ose_plugin::PLUGIN_ABI_VERSION`; breaking change trait → bump major ABI + docs.
- Документация плагинов: [SDK.md](../SDK.md), [PLUGIN_ARCHITECTURE.md](../PLUGIN_ARCHITECTURE.md), [INDEX.md](../INDEX.md).

## Alternatives considered

| Вариант | Почему нет сейчас |
|---------|-------------------|
| cdylib | Нестабильно на musl/OpenWrt; ABI hell; сложнее подпись/обновление |
| WASM | +runtime MB; host imports сложны; избыточно для strip-фильтров |
