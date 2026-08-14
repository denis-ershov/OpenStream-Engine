#!/usr/bin/env python3
"""Тестирование Гипотезы 2: Clean Proxy (Запрос токена / плейлиста через Geo-прокси стран без монетизации)."""

from __future__ import annotations

import argparse
import json
import sys
import unittest
from typing import Any
from urllib.parse import quote, urljoin

import httpx

if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"
GQL_QUERY = """query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) { value signature }
}"""

AD_MARKERS = ("twitch-stitched-ad", "X-TV-TWITCH-AD", "ad_break", "midroll", "Amazon|")

# Известные публичные эндпоинты Clean Proxy (TTV LOL PRO / Luminous)
CLEAN_PROXY_ENDPOINTS = [
    "https://api.ttv.lol/playlist/{channel}.m3u8",
    "https://eu.luminous.dev/live/{channel}",
    "https://us.luminous.dev/live/{channel}",
]


def check_token_clean(token_value_str: str) -> dict[str, Any]:
    """Проверяет флаги рекламы в PlaybackAccessToken."""
    try:
        data = json.loads(token_value_str)
        return {
            "show_ads": data.get("show_ads", True),
            "server_ads": data.get("server_ads", True),
            "hide_ads": data.get("hide_ads", False),
            "user_ip": data.get("user_ip", "unknown"),
            "is_clean": not data.get("show_ads", True) and not data.get("server_ads", True)
        }
    except Exception as e:
        return {"error": str(e), "is_clean": False}


def check_playlist_ads(playlist_text: str) -> dict[str, Any]:
    """Проверяет наличие SSAI рекламных маркеров в HLS-плейлисте."""
    found_markers = [m for m in AD_MARKERS if m.lower() in playlist_text.lower()]
    return {
        "ads_found": len(found_markers) > 0,
        "markers": found_markers,
        "is_valid_hls": playlist_text.startswith("#EXTM3U"),
        "line_count": len(playlist_text.splitlines())
    }


def fetch_via_clean_proxy(channel: str, proxy_url: str | None = None) -> dict[str, Any]:
    """Запрашивает мастер/медиа плейлист через Clean Proxy или Custom Proxy."""
    client_kwargs: dict[str, Any] = {"timeout": 15.0}
    if proxy_url:
        client_kwargs["proxy"] = proxy_url

    results: dict[str, Any] = {"channel": channel, "proxy": proxy_url}

    with httpx.Client(**client_kwargs) as client:
        # Проверяем доступность Clean Proxy эндпоинтов
        for ep in CLEAN_PROXY_ENDPOINTS:
            target_url = ep.format(channel=channel.lower())
            try:
                r = client.get(target_url, headers={"X-Donate-To": "https://ttv.lol/donate"})
                if r.status_code == 200 and "#EXTM3U" in r.text:
                    ad_check = check_playlist_ads(r.text)
                    results["endpoint_used"] = target_url
                    results["status"] = 200
                    results["ads_found"] = ad_check["ads_found"]
                    results["markers"] = ad_check["markers"]
                    results["clean_playlist_sample"] = "\n".join(r.text.splitlines()[:10])
                    results["pass"] = not ad_check["ads_found"]
                    return results
            except Exception as e:
                results[f"err_{ep}"] = str(e)

        # Fallback: прямой запрос GQL + Usher через прокси
        try:
            token_resp = client.post(
                "https://gql.twitch.tv/gql",
                json={"operationName": "PlaybackAccessToken_Template", "query": GQL_QUERY, "variables": {"isLive": True, "login": channel.lower(), "playerType": "site"}},
                headers={"Client-ID": TWITCH_CLIENT_ID}
            )
            if token_resp.status_code == 200:
                tok_data = token_resp.json().get("data", {}).get("streamPlaybackAccessToken", {})
                val, sig = tok_data.get("value"), tok_data.get("signature")
                if val and sig:
                    results["token_analysis"] = check_token_clean(val)
                    master_url = f"https://usher.ttvnw.net/api/channel/hls/{channel.lower()}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
                    master_resp = client.get(master_url)
                    if master_resp.status_code == 200 and "#EXTM3U" in master_resp.text:
                        media_uris = [line.strip() for line in master_resp.text.splitlines() if line.startswith("http")]
                        if media_uris:
                            media_resp = client.get(media_uris[0])
                            ad_check = check_playlist_ads(media_resp.text)
                            results["status"] = 200
                            results["ads_found"] = ad_check["ads_found"]
                            results["markers"] = ad_check["markers"]
                            results["pass"] = not ad_check["ads_found"]
                            return results
        except Exception as e:
            results["direct_proxy_err"] = str(e)

    results["pass"] = False
    return results


class TestCleanProxy(unittest.TestCase):
    """Модульные тесты Clean Proxy."""

    def test_clean_token_validation(self):
        clean_token = json.dumps({"show_ads": False, "server_ads": False, "hide_ads": True, "user_ip": "185.10.10.1"})
        res = check_token_clean(clean_token)
        self.assertTrue(res["is_clean"])
        self.assertFalse(res["show_ads"])

    def test_ad_token_validation(self):
        ad_token = json.dumps({"show_ads": True, "server_ads": True, "hide_ads": False, "user_ip": "80.90.10.1"})
        res = check_token_clean(ad_token)
        self.assertFalse(res["is_clean"])
        self.assertTrue(res["show_ads"])

    def test_playlist_without_ads(self):
        clean_pl = """#EXTM3U
#EXT-X-VERSION:3
#EXTINF:2.000,live
https://cdn.example.com/segment1.ts
#EXTINF:2.000,live
https://cdn.example.com/segment2.ts
"""
        res = check_playlist_ads(clean_pl)
        self.assertFalse(res["ads_found"])
        self.assertTrue(res["is_valid_hls"])


def run_live_test(channel: str, proxy: str | None = None) -> int:
    """Запускает проверку Clean Proxy на реальном канале."""
    print(f"=== Проверка Гипотезы 2 (Clean Proxy) на канале: {channel} ===")
    if proxy:
        print(f"Используемый прокси: {proxy}")

    res = fetch_via_clean_proxy(channel, proxy)
    print("\n--- Результаты запроса через Clean Proxy ---")
    print(json.dumps(res, indent=2, ensure_ascii=False))

    if res.get("pass"):
        print("\n[OK] Гипотеза подтверждена: Clean Proxy вернул чистый поток без SSAI рекламы!")
        return 0
    else:
        print("\n[FAIL] Реклама обнаружена или прокси-эндпоинт недоступен")
        return 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Тестирование Clean Proxy гипотезы")
    parser.add_argument("--channel", default="ewc_plus_en")
    parser.add_argument("--proxy", default=None, help="Опциональный HTTP/SOCKS5 прокси")
    parser.add_argument("--unit", action="store_true", help="Запуск только unit-тестов")
    args = parser.parse_args()

    if args.unit or len(sys.argv) == 1:
        unittest.main(argv=[sys.argv[0]])
    else:
        sys.exit(run_live_test(args.channel, args.proxy))
