"""PC client: GQL token, usher master, media — direct и SOCKS (E1–E4)."""

from __future__ import annotations

import json
import re
from typing import Any
from urllib.parse import quote, urljoin, urlparse

import httpx

TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"

GQL_QUERY = """
query PlaybackAccessToken_Template($login: String!, $isLive: Boolean!, $playerType: String!) {
  streamPlaybackAccessToken(channelName: $login, params: {platform: "web", playerBackend: "mediaplayer", playerType: $playerType}) @include(if: $isLive) {
    value
    signature
  }
}
""".strip()

AD_MARKERS = (
    "stitched",
    "EXT-X-DATERANGE",
    "X-TV-TWITCH-AD",
    "Amazon",
    "ad_break",
    "midroll",
)


def _client(socks5: str | None, timeout: float = 30.0) -> httpx.Client:
    proxies = socks5 if socks5 else None
    return httpx.Client(
        timeout=timeout,
        follow_redirects=True,
        proxy=proxies,
        headers={"User-Agent": "OpenTwitchAutolab/0.1"},
    )


def fetch_playback_token(channel: str, socks5: str | None = None) -> dict[str, Any]:
    payload = {
        "operationName": "PlaybackAccessToken_Template",
        "query": GQL_QUERY,
        "variables": {
            "isLive": True,
            "login": channel.lower(),
            "playerType": "site",
        },
    }
    with _client(socks5) as c:
        r = c.post(
            "https://gql.twitch.tv/gql",
            json=payload,
            headers={
                "Client-ID": TWITCH_CLIENT_ID,
                "Content-Type": "application/json",
            },
        )
        body = r.text
        out: dict[str, Any] = {
            "ok": r.is_success,
            "status": r.status_code,
            "via": "socks5" if socks5 else "direct",
        }
        if not r.is_success:
            out["error"] = body[:300]
            return out
        data = r.json()
        token_obj = (data.get("data") or {}).get("streamPlaybackAccessToken")
        if not token_obj:
            out["ok"] = False
            out["error"] = "no streamPlaybackAccessToken (offline?)"
            out["raw_keys"] = list((data.get("data") or {}).keys())
            return out
        out["token"] = token_obj.get("value")
        out["sig"] = token_obj.get("signature")
        return out


def fetch_usher_master(channel: str, token: str, sig: str, socks5: str | None = None) -> dict[str, Any]:
    path = (
        f"/api/channel/hls/{channel.lower()}.m3u8"
        f"?client_id={TWITCH_CLIENT_ID}"
        f"&token={quote(token, safe='')}"
        f"&sig={quote(sig, safe='')}"
        f"&allow_source=true&allow_audio_only=true"
    )
    url = f"https://usher.ttvnw.net{path}"
    with _client(socks5) as c:
        r = c.get(url, headers={"Client-ID": TWITCH_CLIENT_ID})
        text = r.text
        out: dict[str, Any] = {
            "ok": r.is_success and "#EXTM3U" in text,
            "status": r.status_code,
            "via": "socks5" if socks5 else "direct",
            "url": url,
            "bytes": len(text.encode("utf-8", errors="replace")),
            "body": text if r.is_success else text[:500],
        }
        return out


def parse_master_variants(master: str) -> list[dict[str, Any]]:
    lines = master.splitlines()
    variants: list[dict[str, Any]] = []
    pending: dict[str, Any] | None = None
    for line in lines:
        if line.startswith("#EXT-X-STREAM-INF:"):
            pending = {"inf": line}
            m = re.search(r"BANDWIDTH=(\d+)", line)
            if m:
                pending["bandwidth"] = int(m.group(1))
            m = re.search(r'RESOLUTION=(\d+x\d+)', line)
            if m:
                pending["resolution"] = m.group(1)
            if "SOURCE" in line.upper() or "chunked" in line.lower():
                pending["source_hint"] = True
        elif pending is not None and line and not line.startswith("#"):
            pending["uri"] = line.strip()
            variants.append(pending)
            pending = None
    return variants


def e4_quality(variants: list[dict[str, Any]]) -> dict[str, Any]:
    resolutions = [v.get("resolution") for v in variants if v.get("resolution")]
    has_1440 = any(r and ("2560x1440" in r or "1440" in r) for r in resolutions)
    has_1080 = any(r and ("1920x1080" in r or "1080" in r) for r in resolutions)
    max_bw = max((v.get("bandwidth") or 0) for v in variants) if variants else 0
    source_hint = any(v.get("source_hint") for v in variants)
    return {
        "variant_count": len(variants),
        "resolutions": resolutions,
        "has_1440": has_1440,
        "has_1080": has_1080,
        "max_bandwidth": max_bw,
        "source_hint": source_hint,
        "pass": len(variants) > 0 and (has_1080 or has_1440 or source_hint or max_bw > 0),
    }


def pick_media_url(master: str, master_url: str) -> str | None:
    variants = parse_master_variants(master)
    if not variants:
        return None
    # highest bandwidth
    best = max(variants, key=lambda v: v.get("bandwidth") or 0)
    uri = best.get("uri") or ""
    if uri.startswith("http"):
        return uri
    return urljoin(master_url, uri)


def fetch_media(url: str, socks5: str | None = None) -> dict[str, Any]:
    with _client(socks5) as c:
        r = c.get(url)
        text = r.text
        ads = [m for m in AD_MARKERS if m.lower() in text.lower()]
        return {
            "ok": r.is_success and "#EXTM3U" in text,
            "status": r.status_code,
            "via": "socks5" if socks5 else "direct",
            "url": url,
            "host": urlparse(url).hostname,
            "bytes": len(text.encode("utf-8", errors="replace")),
            "ads_markers": ads,
            "ads_suspect": len(ads) > 0,
            "body_preview": text[:400],
        }


def run_client_gates(
    channel: str,
    *,
    socks5: str | None = None,
    browser_only: bool = False,
) -> dict[str, Any]:
    """E1–E4. Without socks5: E1/E2 skipped; E3/E4 on direct path."""
    report: dict[str, Any] = {"channel": channel, "gates": {}}

    if browser_only:
        for g in ("E1", "E2", "E3", "E4"):
            report["gates"][g] = {"status": "skipped", "reason": "browser-only"}
        return report

    # Token+master preferably via socks for E1; always try direct for local E3/E4
    token_socks = fetch_playback_token(channel, socks5) if socks5 else None
    token_direct = fetch_playback_token(channel, None)

    if socks5:
        if not token_socks or not token_socks.get("ok"):
            report["gates"]["E1"] = {
                "status": "fail",
                "token": token_socks,
            }
            report["gates"]["E2"] = {"status": "skipped", "reason": "E1 failed"}
        else:
            master_socks = fetch_usher_master(
                channel, token_socks["token"], token_socks["sig"], socks5
            )
            media_url = None
            media_direct = None
            if master_socks.get("ok"):
                media_url = pick_media_url(master_socks["body"], master_socks["url"])
                if media_url:
                    media_direct = fetch_media(media_url, None)  # ISP path
            e1_ok = bool(
                master_socks.get("ok")
                and media_direct
                and media_direct.get("ok")
            )
            report["gates"]["E1"] = {
                "status": "pass" if e1_ok else "fail",
                "token_socks": {"ok": token_socks.get("ok"), "status": token_socks.get("status")},
                "master_socks": {
                    "ok": master_socks.get("ok"),
                    "status": master_socks.get("status"),
                },
                "media_direct": {
                    "ok": (media_direct or {}).get("ok"),
                    "status": (media_direct or {}).get("status"),
                    "host": (media_direct or {}).get("host"),
                },
            }
            report["gates"]["E2"] = {
                "status": "pass" if e1_ok else "fail",
                "hosts_via_vps_hypothesis": ["gql.twitch.tv", "usher.ttvnw.net"],
                "media_host": (media_direct or {}).get("host"),
                "note": "confirm against flow_map after E0",
            }
            if master_socks.get("ok"):
                variants = parse_master_variants(master_socks["body"])
                q = e4_quality(variants)
                report["gates"]["E4"] = {
                    "status": "pass" if q["pass"] else "fail",
                    **{k: v for k, v in q.items() if k != "pass"},
                }
            else:
                report["gates"]["E4"] = {"status": "skipped", "reason": "no master"}
            if media_direct and media_direct.get("ok"):
                report["gates"]["E3"] = {
                    "status": "pass" if not media_direct.get("ads_suspect") else "fail",
                    "ads_markers": media_direct.get("ads_markers"),
                    "note": "no midroll in window still possible",
                }
            else:
                report["gates"]["E3"] = {
                    "status": "fail" if media_url else "skipped",
                    "reason": "media direct fetch failed or no url",
                }
        return report

    # No SOCKS: local direct E3/E4
    report["gates"]["E1"] = {"status": "skipped", "reason": "no --socks5"}
    report["gates"]["E2"] = {"status": "skipped", "reason": "no --socks5"}
    if not token_direct.get("ok"):
        report["gates"]["E3"] = {"status": "fail", "token": token_direct}
        report["gates"]["E4"] = {"status": "skipped"}
        return report
    master = fetch_usher_master(
        channel, token_direct["token"], token_direct["sig"], None
    )
    if not master.get("ok"):
        report["gates"]["E3"] = {"status": "fail", "master": master}
        report["gates"]["E4"] = {"status": "fail", "master": master}
        return report
    variants = parse_master_variants(master["body"])
    q = e4_quality(variants)
    report["gates"]["E4"] = {
        "status": "pass" if q["pass"] else "fail",
        **{k: v for k, v in q.items() if k != "pass"},
    }
    media_url = pick_media_url(master["body"], master["url"])
    if not media_url:
        report["gates"]["E3"] = {"status": "skipped", "reason": "no media uri"}
        return report
    media = fetch_media(media_url, None)
    report["gates"]["E3"] = {
        "status": "pass" if media.get("ok") and not media.get("ads_suspect") else (
            "fail" if media.get("ok") else "fail"
        ),
        "ads_markers": media.get("ads_markers"),
        "media_status": media.get("status"),
        "via": "direct",
    }
    return report
