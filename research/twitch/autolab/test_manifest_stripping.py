#!/usr/bin/env python3
"""Тестирование Гипотезы 1: HLS Manifest Stripping (Вырезание рекламных сегментов из манифеста)."""

from __future__ import annotations

import argparse
import json
import re
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


def strip_manifest_ads(manifest_text: str) -> tuple[str, dict[str, Any]]:
    """
    Вырезает SSAI рекламные вставки из HLS медиа-плейлиста (эквивалент ose-plugin-twitch).
    Возвращает очищенный манифест и статистику стриппинга.
    """
    lines = manifest_text.splitlines()
    cleaned_lines: list[str] = []
    ads_found = False
    segments_removed = 0
    in_ad_block = False

    for i, line in enumerate(lines):
        line_clean = line.strip()

        # 1. Детектирование начала рекламы
        if "#EXT-X-DATERANGE:" in line_clean and any(m.lower() in line_clean.lower() for m in AD_MARKERS):
            ads_found = True
            in_ad_block = True
            continue

        if any(m.lower() in line_clean.lower() for m in AD_MARKERS):
            ads_found = True
            in_ad_block = True
            if not line_clean.startswith("#"):
                segments_removed += 1
            continue

        # 2. Удаление prefetch рекламы
        if line_clean.startswith("#EXT-X-TWITCH-PREFETCH:") and in_ad_block:
            continue

        # 3. Сегменты внутри рекламного блока
        if in_ad_block:
            if line_clean.startswith("#EXTINF:"):
                if any(m.lower() in line_clean.lower() for m in AD_MARKERS):
                    continue
                if "live" in line_clean.lower() or "video" in line_clean.lower():
                    in_ad_block = False
                    cleaned_lines.append("#EXT-X-DISCONTINUITY")
                    cleaned_lines.append(line)
                    continue
            elif not line_clean.startswith("#") and line_clean:
                segments_removed += 1
                continue
            elif line_clean.startswith("#EXT-X-DISCONTINUITY"):
                in_ad_block = False
                continue

        cleaned_lines.append(line)

    cleaned_manifest = "\n".join(cleaned_lines)
    stats = {
        "ads_found": ads_found,
        "segments_removed": segments_removed,
        "original_len": len(manifest_text),
        "cleaned_len": len(cleaned_manifest),
        "is_valid_hls": cleaned_manifest.startswith("#EXTM3U")
    }
    return cleaned_manifest, stats


class TestManifestStripping(unittest.TestCase):
    """Модульные тесты Manifest Stripping."""

    def test_strip_preroll_ad(self):
        sample_preroll = """#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:0
#EXT-X-DATERANGE:ID="stitched-ad-1",CLASS="twitch-stitched-ad",X-TV-TWITCH-AD-ROLL-TYPE="PREROLL"
#EXTINF:2.000,Amazon|12345
https://cdn.example.com/ad_segment_1.ts
#EXTINF:2.000,Amazon|12345
https://cdn.example.com/ad_segment_2.ts
#EXT-X-DISCONTINUITY
#EXTINF:2.000,live
https://cdn.example.com/live_segment_101.ts
#EXTINF:2.000,live
https://cdn.example.com/live_segment_102.ts
"""
        cleaned, stats = strip_manifest_ads(sample_preroll)
        self.assertTrue(stats["ads_found"])
        self.assertEqual(stats["segments_removed"], 2)
        self.assertTrue(stats["is_valid_hls"])
        self.assertNotIn("twitch-stitched-ad", cleaned)
        self.assertNotIn("ad_segment_1.ts", cleaned)
        self.assertIn("live_segment_101.ts", cleaned)
        self.assertIn("live_segment_102.ts", cleaned)

    def test_strip_midroll_ad(self):
        sample_midroll = """#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:100
#EXTINF:2.000,live
https://cdn.example.com/live_100.ts
#EXT-X-DATERANGE:ID="midroll-1",CLASS="twitch-stitched-ad",X-TV-TWITCH-AD-ROLL-TYPE="MIDROLL"
#EXTINF:2.000,ad
https://cdn.example.com/ad_midroll.ts
#EXT-X-TWITCH-PREFETCH:https://cdn.example.com/ad_prefetch.ts
#EXT-X-DISCONTINUITY
#EXTINF:2.000,live
https://cdn.example.com/live_101.ts
"""
        cleaned, stats = strip_manifest_ads(sample_midroll)
        self.assertTrue(stats["ads_found"])
        self.assertEqual(stats["segments_removed"], 1)
        self.assertNotIn("midroll", cleaned)
        self.assertNotIn("ad_midroll.ts", cleaned)
        self.assertNotIn("ad_prefetch.ts", cleaned)
        self.assertIn("live_100.ts", cleaned)
        self.assertIn("live_101.ts", cleaned)

    def test_clean_stream_untouched(self):
        sample_clean = """#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:50
#EXTINF:2.000,live
https://cdn.example.com/live_50.ts
#EXTINF:2.000,live
https://cdn.example.com/live_51.ts
"""
        cleaned, stats = strip_manifest_ads(sample_clean)
        self.assertFalse(stats["ads_found"])
        self.assertEqual(stats["segments_removed"], 0)
        self.assertEqual(cleaned.strip(), sample_clean.strip())


def run_live_test(channel: str) -> int:
    """Прогоняет тест стриппинга на живом HLS-потоке канала с рекламой."""
    print(f"=== Проверка Гипотезы 1 (Manifest Stripping) на живом канале: {channel} ===")

    client = httpx.Client(timeout=15.0)
    # 1. Получаем токен
    token_resp = client.post(
        "https://gql.twitch.tv/gql",
        json={"operationName": "PlaybackAccessToken_Template", "query": GQL_QUERY, "variables": {"isLive": True, "login": channel.lower(), "playerType": "site"}},
        headers={"Client-ID": TWITCH_CLIENT_ID}
    )
    if token_resp.status_code != 200:
        print(f"[ERROR] Ошибка получения токена: HTTP {token_resp.status_code}")
        return 1

    token_data = token_resp.json().get("data", {}).get("streamPlaybackAccessToken", {})
    val, sig = token_data.get("value"), token_data.get("signature")
    if not val or not sig:
        print("[ERROR] Канал офлайн или токен не получен")
        return 1

    # 2. Получаем мастер-плейлист
    master_url = f"https://usher.ttvnw.net/api/channel/hls/{channel.lower()}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(val, safe='')}&sig={quote(sig, safe='')}&allow_source=true"
    master_resp = client.get(master_url)
    if master_resp.status_code != 200 or "#EXTM3U" not in master_resp.text:
        print(f"[ERROR] Ошибка получения мастер-плейлиста: HTTP {master_resp.status_code}")
        return 1

    media_uris = [line.strip() for line in master_resp.text.splitlines() if line.startswith("http")]
    if not media_uris:
        print("[ERROR] Медиа-плейлисты не найдены в мастер-плейлисте")
        return 1

    media_url = media_uris[0]
    print(f"[STREAM] Загружен медиа-плейлист: {media_url[:60]}...")

    # 3. Скачиваем медиа-плейлист
    media_resp = client.get(media_url)
    raw_manifest = media_resp.text

    print(f"\n--- Исходный плейлист ({len(raw_manifest)} байт) ---")
    raw_has_ads = any(m.lower() in raw_manifest.lower() for m in AD_MARKERS)
    print(f"Обнаружена SSAI-реклама в исходном плейлисте: {'[AD FOUND] ДА' if raw_has_ads else '[NO ADS] НЕТ'}")

    # 4. Применяем стриппинг
    cleaned_manifest, stats = strip_manifest_ads(raw_manifest)

    print(f"\n--- Результаты стриппинга Manifest Stripping ---")
    print(f"Рекламных блоков найдено: {stats['ads_found']}")
    print(f"Рекламных сегментов удалено: {stats['segments_removed']}")
    print(f"Валидность результирующего HLS: {'[OK] Валиден' if stats['is_valid_hls'] else '[FAIL] Невалиден'}")

    clean_has_ads = any(m.lower() in cleaned_manifest.lower() for m in AD_MARKERS)
    print(f"Наличие рекламы после обработки: {'[FAIL] ОСТАЛАСЬ' if clean_has_ads else '[OK] ПОЛНОСТЬЮ УДАЛЕНА'}")

    print("\n--- Превью очищенного манифеста (первые 15 строк) ---")
    print("\n".join(cleaned_manifest.splitlines()[:15]))

    return 0 if not clean_has_ads and stats["is_valid_hls"] else 1


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--channel":
        ch = sys.argv[2] if len(sys.argv) > 2 else "ewc_plus_en"
        sys.exit(run_live_test(ch))
    else:
        unittest.main()
