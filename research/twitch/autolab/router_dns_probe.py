#!/usr/bin/env python3
"""Проверка Twitch через DNS роутера без MITM и VPN на клиенте."""

from __future__ import annotations

import argparse
import json
import random
import socket
import struct
import sys
import time
from dataclasses import dataclass
from typing import Any
from urllib.parse import quote, urljoin

import httpx

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"
GQL_QUERY = """query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) { value signature }
}"""
AD_MARKERS = ("twitch-stitched-ad", "X-TV-TWITCH-AD", "ad_break", "midroll")
_ORIGINAL_GETADDRINFO = socket.getaddrinfo


class ProbeError(RuntimeError):
    """Ошибка проверки без записи токенов и URL в отчёт."""


def _skip_dns_name(data: bytes, offset: int) -> int:
    while offset < len(data):
        length = data[offset]
        if length & 0xC0 == 0xC0:
            return offset + 2
        if length == 0:
            return offset + 1
        offset += length + 1
    raise ProbeError("повреждённый DNS-ответ")


def resolve_a_via_router(host: str, resolver: str, timeout: float = 3.0) -> list[str]:
    """A-записи только от UDP DNS роутера, без системного resolver/DoH."""
    query_id = random.randrange(0, 65536)
    labels = host.rstrip(".").split(".")
    question = b"".join(bytes([len(label)]) + label.encode("idna") for label in labels) + b"\0"
    payload = struct.pack("!HHHHHH", query_id, 0x0100, 1, 0, 0, 0) + question + struct.pack("!HH", 1, 1)
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.settimeout(timeout)
        sock.sendto(payload, (resolver, 53))
        data, _ = sock.recvfrom(4096)
    if len(data) < 12:
        raise ProbeError(f"DNS {resolver} вернул короткий ответ для {host}")
    response_id, flags, questions, answers, _, _ = struct.unpack("!HHHHHH", data[:12])
    if response_id != query_id or flags & 0x000F:
        raise ProbeError(f"DNS {resolver} не разрешил {host}")
    offset = 12
    for _ in range(questions):
        offset = _skip_dns_name(data, offset) + 4
    addresses: list[str] = []
    for _ in range(answers):
        offset = _skip_dns_name(data, offset)
        if offset + 10 > len(data):
            raise ProbeError("повреждённая DNS-запись")
        record_type, record_class, _, rdlength = struct.unpack("!HHIH", data[offset : offset + 10])
        offset += 10
        rdata = data[offset : offset + rdlength]
        offset += rdlength
        if record_type == 1 and record_class == 1 and rdlength == 4:
            addresses.append(socket.inet_ntoa(rdata))
    if not addresses:
        raise ProbeError(f"DNS {resolver} не вернул A-запись для {host}")
    return addresses


@dataclass
class RouterDns:
    resolver: str
    cache: dict[str, list[str]]

    def resolve(self, host: str) -> list[str]:
        host = host.lower().rstrip(".")
        if host not in self.cache:
            self.cache[host] = resolve_a_via_router(host, self.resolver)
        return self.cache[host]


class RouterDnsBindings:
    def __init__(self, dns: RouterDns) -> None:
        self.dns = dns

    def __enter__(self) -> "RouterDnsBindings":
        def getaddrinfo(host: str, port: int, family: int = 0, type: int = 0, proto: int = 0, flags: int = 0) -> list[Any]:
            if host.endswith((".twitch.tv", ".ttvnw.net", ".live-video.net", ".cloudfront.net")):
                return _ORIGINAL_GETADDRINFO(self.dns.resolve(host)[0], port, family, type, proto, flags)
            return _ORIGINAL_GETADDRINFO(host, port, family, type, proto, flags)
        socket.getaddrinfo = getaddrinfo
        return self

    def __exit__(self, *_: object) -> None:
        socket.getaddrinfo = _ORIGINAL_GETADDRINFO


def parse_master_variants(master: str) -> list[dict[str, Any]]:
    variants: list[dict[str, Any]] = []
    pending: dict[str, Any] | None = None
    for line in master.splitlines():
        if line.startswith("#EXT-X-STREAM-INF:"):
            pending = {"inf": line}
            for part in line.split(","):
                if part.startswith("BANDWIDTH="):
                    pending["bandwidth"] = int(part.split("=", 1)[1])
                if part.startswith("RESOLUTION="):
                    pending["resolution"] = part.split("=", 1)[1]
        elif pending is not None and line and not line.startswith("#"):
            pending["uri"] = line.strip()
            variants.append(pending)
            pending = None
    return variants


def ad_markers(playlist: str) -> list[str]:
    lower = playlist.lower()
    found = [marker for marker in AD_MARKERS if marker.lower() in lower]
    if "#ext-x-daterange" in lower and "twitch-stitched-ad" in lower:
        found.append("EXT-X-DATERANGE:twitch-stitched-ad")
    return found


def _client() -> httpx.Client:
    try:
        return httpx.Client(timeout=30.0, follow_redirects=True, trust_env=False, headers={"User-Agent": "OpenStreamRouterDnsProbe/1.0", "Client-ID": TWITCH_CLIENT_ID})
    except TypeError:
        return httpx.Client(timeout=30.0, trust_env=False, headers={"User-Agent": "OpenStreamRouterDnsProbe/1.0", "Client-ID": TWITCH_CLIENT_ID})


def run_probe(channel: str, resolver: str, duration: int) -> dict[str, Any]:
    dns = RouterDns(resolver, {})
    with RouterDnsBindings(dns), _client() as client:
        token_response = client.post("https://gql.twitch.tv/gql", json={"operationName": "PlaybackAccessToken_Template", "query": GQL_QUERY, "variables": {"isLive": True, "login": channel.lower(), "playerType": "site"}})
        if not (200 <= token_response.status_code < 300):
            raise ProbeError(f"GQL вернул HTTP {token_response.status_code}")
        token = ((token_response.json().get("data") or {}).get("streamPlaybackAccessToken") or {})
        value, signature = token.get("value"), token.get("signature")
        if not value or not signature:
            raise ProbeError("GQL не выдал PlaybackAccessToken: канал офлайн или ответ изменился")
        token_flags = json.loads(value)
        master_url = f"https://usher.ttvnw.net/api/channel/hls/{channel.lower()}.m3u8?client_id={TWITCH_CLIENT_ID}&token={quote(value, safe='')}&sig={quote(signature, safe='')}&allow_source=true&allow_audio_only=true"
        master_response = client.get(master_url)
        if not (200 <= master_response.status_code < 300) or "#EXTM3U" not in master_response.text:
            raise ProbeError(f"Usher вернул HTTP {master_response.status_code}")
        variants = parse_master_variants(master_response.text)
        if not variants:
            raise ProbeError("master playlist не содержит вариантов")
        media_url = urljoin(master_url, str(max(variants, key=lambda item: item.get("bandwidth", 0))["uri"]))
        observations: list[dict[str, Any]] = []
        started = time.monotonic()
        while True:
            response = client.get(media_url)
            if not (200 <= response.status_code < 300) or "#EXTM3U" not in response.text:
                raise ProbeError(f"media playlist вернул HTTP {response.status_code}")
            observations.append({"at_sec": round(time.monotonic() - started), "ads": ad_markers(response.text)})
            if time.monotonic() - started >= duration:
                break
            time.sleep(min(5, duration - (time.monotonic() - started)))
    resolutions = [item.get("resolution") for item in variants if item.get("resolution")]
    ads_found = any(item["ads"] for item in observations)
    return {"channel": channel, "resolver": resolver, "dns_answers": dns.cache, "token": {"show_ads": token_flags.get("show_ads"), "server_ads": token_flags.get("server_ads")}, "quality": {"resolutions": resolutions, "has_1440": any("2560x1440" in value for value in resolutions)}, "playlist_observations": observations, "ads_found": ads_found, "pass": not ads_found and token_flags.get("show_ads") is False}


def main() -> int:
    parser = argparse.ArgumentParser(description="Twitch: проверка действующей DNS-стратегии роутера")
    parser.add_argument("--channel", required=True)
    parser.add_argument("--resolver", required=True, help="LAN-адрес dnsmasq роутера")
    parser.add_argument("--duration", type=int, default=600, help="polling media playlist, минимум 60 с")
    parser.add_argument("--output", default="-", help="путь JSON или '-' для stdout")
    args = parser.parse_args()
    if args.duration < 60:
        parser.error("--duration должен быть не менее 60 секунд")
    try:
        report = run_probe(args.channel, args.resolver, args.duration)
    except (ProbeError, httpx.HTTPError, json.JSONDecodeError) as error:
        print(f"probe failed: {error}", file=sys.stderr)
        return 2
    output = json.dumps(report, indent=2, ensure_ascii=False)
    if args.output == "-":
        print(output)
    else:
        with open(args.output, "w", encoding="utf-8") as handle:
            handle.write(output + "\n")
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
