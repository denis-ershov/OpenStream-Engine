"""PC client: GQL token, usher master, media — direct и SOCKS (E1–E4)."""

from __future__ import annotations

import json
import re
import socket
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

_original_getaddrinfo = socket.getaddrinfo


def _resolve_doh(host: str) -> str | None:
    try:
        r = httpx.get(
            "https://cloudflare-dns.com/dns-query",
            params={"name": host, "type": "A"},
            headers={"accept": "application/dns-json"},
            timeout=5.0
        )
        data = r.json()
        for answer in data.get("Answer", []):
            if answer.get("type") == 1:  # A record
                return answer.get("data")
    except Exception:
        pass
    return None


def _patch_dns(gql_ip: str | None, usher_ip: str | None) -> None:
    def custom_getaddrinfo(host: str, port: int, family: int = 0, type: int = 0, proto: int = 0, flags: int = 0) -> list[Any]:
        if host == "gql.twitch.tv" and gql_ip:
            return _original_getaddrinfo(gql_ip, port, family, type, proto, flags)
        if host == "usher.ttvnw.net" and usher_ip:
            return _original_getaddrinfo(usher_ip, port, family, type, proto, flags)
        return _original_getaddrinfo(host, port, family, type, proto, flags)
    socket.getaddrinfo = custom_getaddrinfo


def _unpatch_dns() -> None:
    socket.getaddrinfo = _original_getaddrinfo


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
        ads = []
        for m in AD_MARKERS:
            if m.lower() in text.lower():
                if m == "EXT-X-DATERANGE" and "twitch-stitched-ad" not in text.lower():
                    continue
                ads.append(m)
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
    """E1–E4. Tests routing combinations to bypass ads while keeping quality."""
    report: dict[str, Any] = {"channel": channel, "gates": {}, "combo_routes": {}}

    if browser_only:
        for g in ("E1", "E2", "E3", "E4"):
            report["gates"][g] = {"status": "skipped", "reason": "browser-only"}
        return report

    # 1. Резолвим реальные IP через DoH
    gql_real_ip = _resolve_doh("gql.twitch.tv")
    usher_real_ip = _resolve_doh("usher.ttvnw.net")

    # Функция для проведения теста одного маршрута
    def test_single_route(gql_via_doh: bool, usher_via_doh: bool) -> dict[str, Any]:
        gql_ip = gql_real_ip if gql_via_doh else None
        usher_ip = usher_real_ip if usher_via_doh else None

        _patch_dns(gql_ip, usher_ip)
        try:
            # GQL
            gql_socks = socks5 if (not gql_via_doh) else None
            t_res = fetch_playback_token(channel, gql_socks)
            if not t_res.get("ok"):
                return {"ok": False, "error": f"GQL failed: {t_res.get('error')}"}

            # Usher
            usher_socks = socks5 if (not usher_via_doh) else None
            m_res = fetch_usher_master(channel, t_res["token"], t_res["sig"], usher_socks)
            if not m_res.get("ok"):
                return {"ok": False, "error": f"Usher failed: {m_res.get('status')}"}

            # Parse Master
            variants = parse_master_variants(m_res["body"])
            q = e4_quality(variants)

            # Media (всегда напрямую, без патчей)
            media_url = pick_media_url(m_res["body"], m_res["url"])
            if not media_url:
                return {"ok": False, "error": "No media URL"}

            _unpatch_dns()
            media_res = fetch_media(media_url, None)

            # Поиск рекламы
            ads = []
            if media_res.get("ok"):
                ads = media_res.get("ads_markers") or []

            return {
                "ok": True,
                "resolutions": q.get("resolutions"),
                "has_1080": q.get("has_1080"),
                "has_1440": q.get("has_1440"),
                "ads_markers": ads,
                "has_ads": len(ads) > 0,
                "token_ip": json.loads(t_res["token"]).get("user_ip") if t_res.get("token") else None
            }
        except Exception as e:
            return {"ok": False, "error": str(e)}
        finally:
            _unpatch_dns()

    # Запускаем комбо-маршруты
    routes = {
        "R0_direct_all": (True, True),
        "R1_base_geo_split": (False, False),
        "R3_smart_geo_split": (True, False),  # GQL direct (RU), Usher proxy (EU)
        "R2_smart_geo_split_reverse": (False, True)  # GQL proxy (EU), Usher direct (RU)
    }

    combo_results = {}
    for name, (g_doh, u_doh) in routes.items():
        print(f"[client] testing combo route: {name}...", flush=True)
        combo_results[name] = test_single_route(g_doh, u_doh)

    report["combo_routes"] = combo_results

    # Выставляем результирующие гейты для автолаба
    # E1: Работоспособность прокси/SmartDNS
    e1_ok = bool(combo_results["R1_base_geo_split"].get("ok") or combo_results["R3_smart_geo_split"].get("ok"))
    report["gates"]["E1"] = {
        "status": "pass" if e1_ok else "fail",
        "doh_ips": {"gql": gql_real_ip, "usher": usher_real_ip}
    }

    # E2: Минимальный набор хостов (подтверждаем)
    report["gates"]["E2"] = {
        "status": "pass" if e1_ok else "fail",
        "recommended_route": "R3_smart_geo_split (GQL RU + Usher EU)" if (
            combo_results["R3_smart_geo_split"].get("ok") and not combo_results["R3_smart_geo_split"].get("has_ads")
        ) else "R1_base_geo_split"
    }

    # E3: Удалось ли получить качественный поток БЕЗ рекламы?
    clean_route = None
    for name in ("R3_smart_geo_split", "R1_base_geo_split", "R0_direct_all"):
        res = combo_results.get(name) or {}
        if res.get("ok") and not res.get("has_ads") and (res.get("has_1080") or res.get("has_1440")):
            clean_route = name
            break

    if clean_route:
        report["gates"]["E3"] = {
            "status": "pass",
            "working_route": clean_route,
            "ads_markers": combo_results[clean_route].get("ads_markers")
        }
    else:
        report["gates"]["E3"] = {
            "status": "fail",
            "reason": "all routes with quality contain ads or failed",
            "direct_ads": combo_results["R0_direct_all"].get("ads_markers"),
            "smart_ads": combo_results["R3_smart_geo_split"].get("ads_markers")
        }

    # E4: Качество
    best_res = combo_results.get(clean_route or "R1_base_geo_split") or {}
    report["gates"]["E4"] = {
        "status": "pass" if best_res.get("ok") else "fail",
        "resolutions": best_res.get("resolutions"),
        "has_1080": best_res.get("has_1080"),
        "has_1440": best_res.get("has_1440")
    }

    return report
