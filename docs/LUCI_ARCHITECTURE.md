# Архитектура веб-интерфейса LuCI (OpenStream Engine)

Документ фиксирует архитектуру, структуру компонентов, поток данных и модель локализации веб-интерфейса `luci-app-openstream` для OpenWrt 24.10+.

---

## 1. Структура компонентов и зоны ответственности

Веб-интерфейс OpenStream Engine разделен на три изолированных уровня в соответствии со стандартами LuCI современного поколения:

```
[ Браузер пользователя (Клиентский LuCI) ]
      │
      ├── UI / Presentation Layer:
      │     ├── openstream.css (Дизайн-система Linear/Apple/Vercel)
      │     └── views/*.js (8 представлений на ES6 DOM API)
      │
      ├── i18n Localization Layer:
      │     ├── po/ru/openstream.po (Словарь переводов gettext)
      │     └── openstream.ru.lmo (Скомпилированный бинарный кэш SuperFastHash)
      │
      └── Dispatcher & Menu Layer:
            ├── menu.d/luci-app-openstream.json (Декларативное меню LuCI)
            └── luasrc/controller/openstream.lua (Модуль диспетчера)
      │
      ▼ [ JSON-RPC / ubus over HTTP (/ubus) ]
[ Демон rpcd & ucode-плагин на роутере ]
      │
      ├── /usr/share/rpcd/acl.d/luci-app-openstream.json (Права доступа read/write)
      └── /usr/share/rpcd/ucode/openstream.uc (RPC методы ядра openstream)
      │
      ▼ [ Системные вызовы ядра и демонов ]
[ nftables | streamproxyd | Zapret2 | sing-box | dnsmasq ]
```

### Зоны ответственности файлов

| Файл / Каталог | Роль и зона ответственности |
|---|---|
| `root/www/luci-static/resources/openstream/openstream.css` | Единая дизайн-система: CSS-токены, адаптивные сетки, темы (Dark/Light), карточки, переключатели, баджи. |
| `root/www/luci-static/resources/view/openstream/status.js` | Панель мониторинга здоровья ключевых служб (streamproxyd, zapret2, sing-box, nftables), мягкий перезапуск через `luci:setInitStatus`. |
| `root/www/luci-static/resources/view/openstream/routing.js` | Управление декларативными правилами `.osrule.yaml`, живой инспектор маршрута домена, модальный редактор. |
| `root/www/luci-static/resources/view/openstream/servers.js` | Управление исходящими серверами sing-box, замер пинга, выбор режима шлюза (URLTest / Manual), импорт подписок. |
| `root/www/luci-static/resources/view/openstream/services.js` | Сетевые сервисы: каскадный Multi-DNS, DoH Bootstrap, блокировка QUIC/DoH, исключения CIDR/MAC, экспорт/импорт бэкапа. |
| `root/www/luci-static/resources/view/openstream/twitch.js` | Оптимизатор прямых трансляций Twitch: архитектура Zero-CA, готовые сценарии (Geo-Split, Playlist Edge, Quality Unlock, Custom). |
| `root/www/luci-static/resources/view/openstream/monitor.js` | Живой мониторинг распределения потоков по подсистемам ядра в реальном времени. |
| `root/www/luci-static/resources/view/openstream/updates.js` | Управление сборками sing-box (4 редакции), автообновление cron, статус компонентов, терминал журнала. |
| `root/www/luci-static/resources/view/openstream/diagnostics.js` | Автоматизированная самодиагностика очередей, сокетов и динамических таблиц. |
| `root/usr/share/luci/menu.d/luci-app-openstream.json` | Декларативное описание структуры навигационного меню в интерфейсе OpenWrt. |
| `root/usr/share/rpcd/ucode/openstream.uc` | Серверная реализация RPC API на языке ucode, прямое взаимодействие с uci, fs и процессами. |
| `root/usr/share/rpcd/acl.d/luci-app-openstream.json` | Декларативная матрица прав доступа LuCI/rpcd с четким разделением `read` и `write`. |
| `po/ru/openstream.po` & `*.lmo` | Полная локализация интерфейса на русский язык. |

---

## 2. Поток данных (Data Flow)

1. **Загрузка страницы:**
   - LuCI диспетчер (`menu.d/*.json`) сопоставляет маршрут URL (например, `admin/services/openstream/routing`) с путем к модулю `openstream/routing`.
   - Клиентский движок LuCI асинхронно запрашивает JavaScript-модуль `root/www/luci-static/resources/view/openstream/routing.js`.
   - Метод `load()` выполняет параллельные запросы к `ubus` через `rpc.declare` (серверный объект `openstream` в `openstream.uc`).
   - Метод `render()` создает DOM-элементы с использованием дизайн-системы `openstream.css`.
   - Функция `_('...')` перехватывает английские строки и подставляет локализованный перевод из `openstream.ru.lmo`.

2. **Сохранение конфигурации:**
   - Пользователь нажимает «Сохранить и применить».
   - Фронтенд собирает значения полей, валидирует форматы (IP, CIDR, домены, URL).
   - Вызывается соответствующий RPC-метод записи (`save_routing`, `save_servers`, `save_dns_config` и т.д.).
   - `openstream.uc` атомарно записывает параметры в UCI или файлы правил `/etc/openstream/rules/*.osrule.yaml`.
   - Скрипт-рендер `/usr/libexec/openstream-render.uc` компилирует правила в таблицы nftables без сброса накопленных динамических DNS-сетов.
   - Пользователь получает всплывающее уведомление `ui.addNotification` об успешном применении.

---

## 3. Ключевые архитектурные решения

1. **Отказ от серверного рендеринга шаблонов `.htm`:**
   Все устаревшие шаблоны `.htm` из `luasrc/view/` полностью удалены. Все экраны работают как клиентские одностраничные приложения (SPA), не нагружая процессор роутера рендерингом HTML в Lua.
2. **Изоляция диспетчеризации меню:**
   В `menu.d/luci-app-openstream.json` исключены условия `depends`, способные скрыть пункт меню при пустом UCI. В контроллере `openstream.lua` исключены повторные вызовы `entry(...)`, что исключает конфликты диспетчера в LuCI 24.10.
3. **Безопасный reload rpcd при установке:**
   В скрипте `luci_postinst` жесткая остановка процесса `killall -HUP rpcd` заменена на безопасный reload `ubus call rpcd reload`, что предотвращает обрыв сессии авторизации и не выбрасывает администратора на страницу логина при установке пакета.
4. **Соблюдение правил Mobile First:**
   Во всех 8 экранах запрещены HTML-таблицы. Для мобильных телефонов, планшетов и экранов MacBook/десктопов используются отзывчивые CSS-сетки (`.os-grid`, `.os-grid-2`), карточки (`.os-card`, `.os-entity-row`) и адаптивные переключатели (`.os-switch`).
5. **Честная двухуровневая локализация:**
   В исходном коде JS используется исключительно английский язык в вызовах `_('...')`. Все переводы собраны в `openstream.po` и скомпилированы в бинарный индекс `openstream.ru.lmo` с алгоритмом хэширования `SuperFastHash`. При выборе английского языка интерфейс остается аккуратным английским, при выборе русского — на 100% переведен на профессиональный технический русский язык.

---

## 4. Внешние зависимости и границы системы

- **Внешние зависимости фронтенда:**
  - LuCI core (`view.js`, `dom.js`, `rpc.js`, `ui.js`, `poll.js`, `uci.js`).
  - ubus HTTP endpoint (`/ubus`).
- **Внешние зависимости бэкенда (`openstream.uc`):**
  - `ucode`, `ucode-mod-fs`, `ucode-mod-uci`.
  - `rpcd` с настроенным ACL.
- **Ограничения:**
  - Работает на OpenWrt 22.03, 23.05, 24.10+.
  - Требует наличия установленного пакета `openstream-engine` (демон `streamproxyd` и скрипты ядра).
