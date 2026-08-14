#!/usr/bin/env python3
"""Тестирование безопасности конфигурации dnsmasq и правил маршрутизации OpenWrt."""

from __future__ import annotations

import re
import sys
import unittest

if sys.stdout.encoding != 'utf-8':
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except Exception:
        pass

# Полный список 21 домена для sinkhole
AD_DOMAINS = [
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
    "imasdk.googleapis.com",
    "pubads.g.doubleclick.net",
    "securepubads.g.doubleclick.net",
    "pagead2.googlesyndication.com",
    "adservice.google.com",
    "sb.scorecardresearch.com",
    "b.scorecardresearch.com",
    "scorecardresearch.com",
    "quantserve.com",
    "pixel.quantserve.com",
    "use-application-dns.net",
]


def generate_dnsmasq_conf(preset: str) -> str:
    """Генерирует контент openstream.conf для заданного пресета аналогично streamproxyd.init."""
    lines = ["# OpenStream Engine generated DNS routes"]

    def emit_ad_sinkhole():
        for d in AD_DOMAINS:
            lines.append(f"address=/{d}/0.0.0.0")
            lines.append(f"address=/{d}/::")

    if preset == "ru_smartdns_noads_quality":
        lines.append("server=/gql.twitch.tv/77.88.8.8")
        lines.append("server=/gql.twitch.tv/77.88.8.1")
        lines.append("server=/usher.ttvnw.net/83.220.169.155")
        lines.append("server=/usher.ttvnw.net/212.109.195.93")
        emit_ad_sinkhole()
    elif preset == "manifest_strip_edge":
        lines.append("server=/usher.ttvnw.net/83.220.169.155")
        lines.append("server=/usher.ttvnw.net/212.109.195.93")
        emit_ad_sinkhole()
    elif preset == "smartdns_quality_unlock":
        lines.append("server=/usher.ttvnw.net/83.220.169.155")
        lines.append("server=/usher.ttvnw.net/212.109.195.93")
        emit_ad_sinkhole()

    return "\n".join(lines) + "\n"


class TestOpenWrtSafety(unittest.TestCase):
    """Проверка безопасности и синтаксиса генерируемых конфигов dnsmasq."""

    def test_syntax_validity(self):
        """Проверяет каждую строку на соответствие спецификации dnsmasq."""
        for preset in ["ru_smartdns_noads_quality", "manifest_strip_edge", "smartdns_quality_unlock"]:
            conf = generate_dnsmasq_conf(preset)
            for line in conf.splitlines():
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                # Проверяем формат address=
                if line.startswith("address="):
                    m = re.match(r"^address=/[a-zA-Z0-9.\-_]+/(0\.0\.0\.0|::)$", line)
                    self.assertIsNotNone(m, f"Некорректная директива address: {line}")
                # Проверяем формат server=
                elif line.startswith("server="):
                    m = re.match(r"^server=/[a-zA-Z0-9.\-_]+/\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$", line)
                    self.assertIsNotNone(m, f"Некорректная директива server: {line}")

    def test_no_forbidden_entries(self):
        """Гарантирует, что видеопотоки (live-video.net, cloudfront) не блокируются и не ломают скорость."""
        for preset in ["ru_smartdns_noads_quality", "manifest_strip_edge"]:
            conf = generate_dnsmasq_conf(preset)
            self.assertNotIn("address=/live-video.net/", conf, "Критическая ошибка: видеопоток заблокирован!")
            self.assertNotIn("address=/ttvnw.net/", conf, "Критическая ошибка: ttvnw.net заблокирован!")
            self.assertNotIn("server=/live-video.net/", conf, "Видеопотоки должны идти напрямую без DNS-прокси!")

    def test_sinkhole_coverage(self):
        """Проверяет, что все 21 домен рекламы включены в sinkhole."""
        conf = generate_dnsmasq_conf("ru_smartdns_noads_quality")
        for domain in AD_DOMAINS:
            self.assertIn(f"address=/{domain}/0.0.0.0", conf)
            self.assertIn(f"address=/{domain}/::", conf)


if __name__ == "__main__":
    unittest.main()
