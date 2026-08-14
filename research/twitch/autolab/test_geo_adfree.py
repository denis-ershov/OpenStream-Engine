#!/usr/bin/env python3
"""Исследование Geo-эгрессов и Clean Relay для Twitch без рекламы."""

from __future__ import annotations

import json
import sys
from urllib.parse import quote

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


def test_channel_playlist(channel: str = "ewc_plus_en"):
    print(f"=== АНАЛИЗ ЗАПРОСА ПЛЕЙЛИСТА ДЛЯ КАНАЛА: {channel} ===")

    # 1. Прямой запрос к GQL
    with httpx.Client(timeout=10.0) as client:
        r = client.post(
            "https://gql.twitch.tv/gql",
            json={"operationName": "PlaybackAccessToken_Template", "query": GQL_QUERY, "variables": {"isLive": True, "login": channel, "playerType": "site"}},
            headers={"Client-ID": TWITCH_CLIENT_ID}
        )
        data = r.json()
        tok = data.get("data", {}).get("streamPlaybackAccessToken", {})
        val_str = tok.get("value", "{}")
        sig = tok.get("signature", "")
        tok_json = json.loads(val_str)

        print("\n--- Токен PlaybackAccessToken (напрямую) ---")
        print(f"User IP, который видит Twitch: {tok_json.get('user_ip')}")
        print(f"show_ads: {tok_json.get('show_ads')}")
        print(f"server_ads: {tok_json.get('server_ads')}")

        # 2. Запрос плейлиста через Clean API (Luminous / TTV LOL)
        print("\n--- Запрос через Clean API (Luminous Dev) ---")
        try:
            clean_r = client.get(f"https://eu.luminous.dev/live/{channel}")
            print(f"Status: {clean_r.status_code}")
            has_stitched = "stitched" in clean_r.text
            print(f"Содержит рекламные теги 'stitched': {has_stitched}")
            if clean_r.status_code == 200:
                print("Первые 10 строк чистого плейлиста:")
                print("\n".join(clean_r.text.splitlines()[:10]))
        except Exception as e:
            print(f"Ошибка Clean API: {e}")


if __name__ == "__main__":
    test_channel_playlist("ewc_plus_en")
