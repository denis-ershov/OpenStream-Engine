# ose-plugin-skeleton

Шаблон out-of-tree плагина OpenStream Engine.

1. Скопируйте каталог.
2. В `Cargo.toml` укажите path/git на `ose-plugin` / `ose-manifest`.
3. Раскомментируйте impl в `src/lib.rs`.
4. Зарегистрируйте в `streamproxyd` `build_plugins` и пересоберите.

См. [docs/SDK.md](../../docs/SDK.md) и [ADR 0001](../../docs/adr/0001-plugin-abi.md).
