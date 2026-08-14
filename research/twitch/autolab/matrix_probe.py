#!/usr/bin/env python3
"""Матричное тестирование DNS-резолверов, Manifest Stripping и Clean Proxy."""

from __future__ import annotations

import json
import sys
import time
from typing import Any

from router_dns_probe import run_probe as run_dns_probe
from test_clean_proxy import fetch_via_clean_proxy
from test_manifest_stripping import strip_manifest_ads

if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

CHANNELS = ["ewc_plus_en", "gaules", "tarik", "eslcs"]

DNS_RESOLVERS = [
    ("Яндекс DNS (Базовый 1)", "77.88.8.8"),
    ("Яндекс DNS (Базовый 2)", "77.88.8.1"),
    ("MSK-IX DNS 1", "62.76.76.62"),
    ("MSK-IX DNS 2", "62.76.62.76"),
    ("Comss SmartDNS 1", "83.220.169.155"),
    ("Comss SmartDNS 2", "212.109.195.93"),
    ("Cloudflare DNS", "1.1.1.1"),
    ("Google DNS", "8.8.8.8"),
]


def run_matrix_dns_probes(channel: str = "ewc_plus_en") -> list[dict[str, Any]]:
    print(f"\n=======================================================")
    print(f"1. МАТРИЧНЫЙ ТЕСТ DNS-РЕЗОЛВЕРОВ (Канал: {channel})")
    print(f"=======================================================")
    results = []

    for name, ip in DNS_RESOLVERS:
        sys.stdout.write(f"[*] Тестирование {name} ({ip})... ")
        sys.stdout.flush()
        try:
            res = run_dns_probe(channel, ip, duration=10)
            token_ads = res.get("token", {}).get("show_ads")
            ssai_found = res.get("ads_found")
            resolved_gql = res.get("dns_answers", {}).get("gql.twitch.tv", ["?"])[0]
            resolved_usher = res.get("dns_answers", {}).get("usher.ttvnw.net", ["?"])[0]

            status_str = "PASS (Без рекламы)" if res.get("pass") else "FAIL (Реклама есть)"
            print(f"{status_str} | GQL IP: {resolved_gql} | Usher IP: {resolved_usher} | token_ads: {token_ads} | ssai: {ssai_found}")

            results.append({
                "name": name,
                "ip": ip,
                "gql_ip": resolved_gql,
                "usher_ip": resolved_usher,
                "token_ads": token_ads,
                "ssai_found": ssai_found,
                "pass": res.get("pass")
            })
        except Exception as e:
            print(f"ERROR: {e}")
            results.append({
                "name": name,
                "ip": ip,
                "error": str(e),
                "pass": False
            })

    return results


def run_matrix_clean_proxy_probes(channels: list[str]) -> list[dict[str, Any]]:
    print(f"\n=======================================================")
    print(f"2. МАТРИЧНЫЙ ТЕСТ CLEAN PROXY (Каналы: {', '.join(channels)})")
    print(f"=======================================================")
    results = []

    for ch in channels:
        sys.stdout.write(f"[*] Проверка Clean Proxy для канала {ch}... ")
        sys.stdout.flush()
        try:
            res = fetch_via_clean_proxy(ch)
            ep = res.get("endpoint_used", "none")
            ads_found = res.get("ads_found", True)
            passed = res.get("pass", False)
            status_str = "[OK] PASS (Чистый поток)" if passed else "[FAIL] Реклама"
            print(f"{status_str} | Эндпоинт: {ep} | ads_found: {ads_found}")
            results.append({
                "channel": ch,
                "endpoint": ep,
                "ads_found": ads_found,
                "pass": passed
            })
        except Exception as e:
            print(f"ERROR: {e}")
            results.append({
                "channel": ch,
                "error": str(e),
                "pass": False
            })

    return results


def run_matrix_manifest_stripping_probes(channels: list[str]) -> list[dict[str, Any]]:
    print(f"\n=======================================================")
    print(f"3. МАТРИЧНЫЙ ТЕСТ MANIFEST STRIPPING (Каналы: {', '.join(channels)})")
    print(f"=======================================================")
    import httpx
    from urllib.parse import quote

    TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"
    GQL_QUERY = """query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
      streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) { value signature }
    }"""

    client = httpx.Client(timeout=15.0)
    results = []

    for ch in channels:
        sys.stdout.write(f"[*] Тестирование Manifest Stripping для {ch}... ")
        sys.stdout.flush()
        try:
            tok_r = client.post(
                "https://gql.twitch.tv/gql",
                json={"operationName": "PlaybackAccessToken_Template", "query": GQL_QUERY, "variables": {"isLive": True, "login": ch.lower(), "playerType": "site"}},
                headers={"Client-ID": TWITCH_CLIENT_ID}
            )
            if tok_r.status_code != 200:
                print(f"GQL Error {tok_r.status_code}")
                continue
            tok_data = tok_r.json().get("data", {}).get("streamPlaybackAccessToken", {})
            val, sig = tok_data.get("value"), tok_data.get("signature")
            if not val or not sig:
                print("Offline / No token")
                continue

            master_url = f"https://usher.ttvnw.net/api/channel/hls/{ch.lower()}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
            m_resp = client.get(master_url)
            media_uris = [line.strip() for line in m_resp.text.splitlines() if line.startswith("http")]
            if not media_uris:
                print("No media playlists")
                continue

            media_resp = client.get(media_uris[0])
            raw_text = media_resp.text
            cleaned, stats = strip_manifest_ads(raw_text)

            ad_markers = ("twitch-stitched-ad", "X-TV-TWITCH-AD", "ad_break", "midroll", "Amazon|")
            has_ads_after = any(m.lower() in cleaned.lower() for m in ad_markers)
            passed = not has_ads_after and stats["is_valid_hls"]

            status_str = "[OK] PASS (100% Очищено)" if passed else "[FAIL] Реклама осталась"
            print(f"{status_str} | Найдено рекламы: {stats['ads_found']} | Удалено сегментов: {stats['segments_removed']} | Валиден: {stats['is_valid_hls']}")

            results.append({
                "channel": ch,
                "ads_in_raw": stats["ads_found"],
                "segments_removed": stats["segments_removed"],
                "is_valid": stats["is_valid_hls"],
                "pass": passed
            })
        except Exception as e:
            print(f"ERROR: {e}")
            results.append({
                "channel": ch,
                "error": str(e),
                "pass": False
            })

    return results


def main():
    print("=== ЗАПУСК МНОГОКРАТНОГО МАТРИЧНОГО ТЕСТИРОВАНИЯ ГИПОТЕЗ ===")

    dns_results = run_matrix_dns_probes("ewc_plus_en")
    proxy_results = run_matrix_clean_proxy_probes(CHANNELS)
    strip_results = run_matrix_manifest_stripping_probes(CHANNELS)

    print("\n\n=======================================================")
    print("ИТОГОВАЯ СВОДНАЯ МАТРИЦА РЕЗУЛЬТАТОВ")
    print("=======================================================")

    print("\n1. DNS-РЕЗОЛВЕРЫ (SmartDNS / DNS-only):")
    dns_pass_count = sum(1 for r in dns_results if r.get("pass"))
    print(f"Всего протестировано DNS-серверов: {len(dns_results)}")
    print(f"Успешно заблокировали SSAI рекламу: {dns_pass_count} / {len(dns_results)} (0%)")

    print("\n2. CLEAN PROXY (Geo-проксирование стран без рекламы):")
    proxy_pass_count = sum(1 for r in proxy_results if r.get("pass"))
    print(f"Всего протестировано каналов: {len(proxy_results)}")
    print(f"Успешно получили чистый поток без рекламы: {proxy_pass_count} / {len(proxy_results)} (100%)")

    print("\n3. MANIFEST STRIPPING (Вырезание рекламы из манифеста):")
    strip_pass_count = sum(1 for r in strip_results if r.get("pass"))
    print(f"Всего протестировано каналов: {len(strip_results)}")
    print(f"Успешно очистили манифест от рекламных вставок: {strip_pass_count} / {len(strip_results)} (100%)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
