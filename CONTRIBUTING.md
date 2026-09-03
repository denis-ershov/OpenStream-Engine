# Руководство по участию в разработке (Contributing Guide)

Спасибо за интерес к проекту **OpenStream Engine**! Мы рады любому вкладу: исправлению ошибок, добавлению новых сервисных правил (`.osrule.yaml`), улучшению веб-интерфейса LuCI и оптимизации ядра.

[English summary below](#english-summary)

---

## 1. Структура проекта

* `crates/openstream-core` — высокопроизводительное ядро маршрутизации (Reverse Suffix Trie, Longest Prefix Match).
* `crates/openstream-rule` — парсер, AST и валидатор схемы декларативных правил `.osrule.yaml`.
* `crates/openstream-backend-openwrt` — генератор конфигураций nftables, dnsmasq, sing-box и Zapret2.
* `crates/openstream-backend-desktop` — TUN сетевой адаптер для Linux, macOS и Windows.
* `crates/openstream-ffi` / `crates/openstream-jni` — биндинги для мобильных клиентов (iOS NetworkExtension и Android VpnService).
* `luci-app-openstream` — веб-интерфейс LuCI на JavaScript (OLED Dark, Mobile First) и RPC демон на `ucode`.
* `rules/` — официальный каталог декларативных правил сервисов (Twitch, YouTube, Crunchyroll и др.).

---

## 2. Разработка и тестирование

### Требования к окружению
- **Rust**: 1.80+ (Stable).
- **Cargo-инструменты**: `clippy`, `rustfmt`.
- **Python**: 3.9+ (для скрипта сборки IPK пакетов).

### Сборка и запуск тестов
Перед созданием Pull Request обязательно убедитесь, что все тесты и линтеры проходят успешно:

```bash
# Запуск полного набора модульных и интеграционных тестов
cargo test --workspace

# Проверка качества кода с флагом -D warnings (требование CI)
cargo clippy --workspace -- -D warnings

# Проверка форматирования
cargo fmt --all -- --check
```

---

## 3. Добавление или обновление сервисных правил (`.osrule.yaml`)

Каждое правило описывает стратегию маршрутизации конкретного сервиса:
1. Создайте или отредактируйте файл `rules/<service_name>.osrule.yaml`.
2. Правило должно соответствовать схеме `schema_version: "2.1"`.
3. Запустите встроенный валидатор:
   ```bash
   cargo run -p osrule -- lint rules/<service_name>.osrule.yaml
   ```
4. Убедитесь в отсутствии затенения (shadowing) специфических доменов общими wildcard-шаблонами.

---

## 4. Стандарты UI/UX для LuCI Web UI (Правила 8–22)

Если вы работаете над веб-интерфейсом `luci-app-openstream`:
* **Mobile First**: Интерфейс обязан идеально работать на мобильных устройствах, смартфонах и планшетах.
* **Категорический запрет HTML-таблиц**: Классические таблицы `<table>` на мобильных экранах запрещены. Вместо них используются адаптивные карточки (`Card`), списки описаний и accordion.
* **Цветовая палитра OLED Dark**: Используйте согласованные токены `#020617`, `#0b1329`, акцентные цвета (`#3b82f6` синий, `#10b981` изумрудный, `#06b6d4` циан для Bypass, `#a855f7` фиолетовый).
* **Сенсорные цели**: Высота кнопок и интерактивных полей не менее 44 px.

---

## 5. Процесс создания Pull Request

1. Создайте форк репозитория и тематическую ветку (`git checkout -b feature/my-feature`).
2. Внесите изменения с понятными и информативными коммитами.
3. Проверьте прохождение тестов: `cargo test --workspace && cargo clippy --workspace -- -D warnings`.
4. Зафиксируйте изменения в `docs/CHANGELOG.md` согласно правилу ведения истории версий.
5. Отправьте Pull Request и заполните форму описания.

---

<a name="english-summary"></a>
## English Summary

We warmly welcome contributions!
1. **Requirements**: Rust 1.80+ (Stable), Clippy, Python 3.9+.
2. **Quality Gates**: Every PR must pass `cargo test --workspace` and `cargo clippy --workspace -- -D warnings`.
3. **Rule Validation**: Run `cargo run -p osrule -- lint rules/<file>.osrule.yaml` before submitting rules.
4. **UI Design**: Strictly adhere to Mobile-First principles (no HTML tables on small screens, OLED Dark palette).
5. **Changelog**: Please document significant changes in `docs/CHANGELOG.md`.
