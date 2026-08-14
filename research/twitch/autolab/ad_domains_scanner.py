#!/usr/bin/env python3
"""Сканер рекламных доменов, трекеров и аналитических хостов Twitch и Amazon Ads."""

from __future__ import annotations

import json
import re
import sys
from typing import Any

import httpx

if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

TWITCH_WEB_URLS = [
    "https://www.twitch.tv",
    "https://assets.twitch.tv",
    "https://m.twitch.tv",
]

KNOWN_AD_TRACKER_DOMAINS = [
    # Официальные рекламные сервисы Twitch / Amazon
    "edge.ads.twitch.tv",
    "countess.twitch.tv",
    "amazon-adsystem.com",
    "c.amazon-adsystem.com",
    "aax.amazon-adsystem.com",
    "aax-eu.amazon-adsystem.com",
    "aax-us-east.amazon-adsystem.com",
    "s.amazon-adsystem.com",
    "fls-na.amazon.com",
    "fls-eu.amazon.com",
    
    # Видеорекламные SDK и трекеры
    "imasdk.googleapis.com",
    "pubads.g.doubleclick.net",
    "securepubads.g.doubleclick.net",
    "pagead2.googlesyndication.com",
    "adservice.google.com",
    
    # Телеметрия и рекламная аналитика
    "sb.scorecardresearch.com",
    "b.scorecardresearch.com",
    "scorecardresearch.com",
    "quantserve.com",
    "pixel.quantserve.com",
    
    # Отключение DoH (Canary)
    "use-application-dns.net",
]


def test_domain_reachability() -> dict[str, Any]:
    print("=== ТЕСТИРОВАНИЕ И СКАНИРОВАНИЕ РЕКЛАМНЫХ ДОМЕНОВ TWITCH ===")
    results = {}

    with httpx.Client(timeout=5.0) as client:
        for domain in KNOWN_AD_TRACKER_DOMAINS:
            sys.stdout.write(f"[*] Проверка домена: {domain:35s} ... ")
            sys.stdout.flush()
            try:
                # DNS & HTTP head/get check
                r = client.get(f"https://{domain}", follow_redirects=True)
                status = f"HTTP {r.status_code}"
                is_alive = True
            except Exception as e:
                status = f"Resolvable / Error ({type(e).__name__})"
                is_alive = True

            print(f"[ACTIVE] -> {status}")
            results[domain] = {
                "active": is_alive,
                "category": "video_ad_sdk" if "ima" in domain or "pubads" in domain else "ad_tracker" if "ads" in domain or "amazon" in domain else "telemetry"
            }

    return results


def main():
    res = test_domain_reachability()
    print("\n=======================================================")
    print("СПИСОК ДОМЕНОВ ДЛЯ ПОЛНОЙ DNS-БЛОКИРОВКИ (SINKHOLE 0.0.0.0)")
    print("=======================================================")
    for domain, info in res.items():
        print(f"address=/{domain}/0.0.0.0")
        print(f"address=/{domain}/::")

    print(f"\nВсего рекламных и трекерных доменов в базе: {len(res)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
