# Совместимость с zapret / zapret2 / podkop

OpenStream Engine **не** обходит DPI и **не** конкурирует за маршрутизацию. Zapret/nfqws и podkop обрабатывают доступ к CDN; OpenStream только переписывает HLS-манифесты.

## Рекомендуемый порядок

1. Настроить podkop / zapret для доступа к Twitch (если нужен обход блокировок).
2. Установить OpenStream в режиме **`explicit`**.
3. На клиенте указать HTTP proxy `router:18080` (или PAC) и доверить CA (для MITM).

## Правила изоляции

| Область | Запрещено | Делаем |
|---------|-----------|--------|
| nftables | Править `inet zapret`, `inet fw4`, таблицы podkop | Только `inet openstream` |
| fwmark | Полная перезапись mark | По умолчанию не маркируем; иначе OR+маска |
| Redirect | Весь `tcp dport 443` | Opt-in whitelist HLS IP set |
| DNS | dnsmasq / DoH / hosts | Не трогаем |
| Порты | 80 / 443 / 53 | `18080` |

## Режимы

- **`explicit` (default)** — нулевой конфликт.
- **`redirect_whitelist`** — только при отсутствии конфликтующего tpws-redirect на те же dst.
- **`off`** — отладка рядом с агрессивным transparent-стеком.

## Детект соседей

При старте и в `/api/status`: `nfqws`, `nfqws2`, `tpws`, `zapret`, `podkop`, `sing-box`, `xray`, `hydraroute`. LuCI показывает предупреждение, но не блокирует запуск.

## Что не делать

- Blanket-redirect всего HTTPS на OpenStream.
- `flush ruleset` / правка чужих nft-таблиц.
- Борьба с podkop за DNS.
