"""Playwright: открыть канал Twitch, собрать network + HAR."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


def _classify(host: str, path: str) -> str:
    h = host.lower()
    p = path.lower()
    if "gql.twitch.tv" in h:
        return "gql"
    if "usher.ttvnw.net" in h:
        return "usher"
    if "playlist" in h or "live-video.net" in h:
        return "playlist"
    if "video-weaver" in h or "video-edge" in h:
        return "weaver"
    if p.endswith(".ts") or p.endswith(".m4s") or p.endswith(".mp4"):
        return "segment"
    if ".m3u8" in p:
        return "m3u8"
    if "ttvnw" in h or "twitch.tv" in h:
        return "twitch_other"
    return "other"


def capture_channel(
    channel: str,
    out_dir: Path,
    *,
    headed: bool = False,
    duration_sec: float = 45.0,
    timeout_sec: float = 90.0,
    user_data_dir: Path | None = None,
) -> dict[str, Any]:
    from playwright.sync_api import sync_playwright

    out_dir.mkdir(parents=True, exist_ok=True)
    events: list[dict[str, Any]] = []
    t0 = time.time()

    def on_request(req: Any) -> None:
        try:
            u = urlparse(req.url)
            events.append(
                {
                    "t": round(time.time() - t0, 3),
                    "type": "request",
                    "method": req.method,
                    "url": req.url,
                    "host": u.hostname or "",
                    "path": u.path or "/",
                    "class": _classify(u.hostname or "", u.path or "/"),
                }
            )
        except Exception:
            pass

    def on_response(resp: Any) -> None:
        try:
            u = urlparse(resp.url)
            events.append(
                {
                    "t": round(time.time() - t0, 3),
                    "type": "response",
                    "status": resp.status,
                    "url": resp.url,
                    "host": u.hostname or "",
                    "path": u.path or "/",
                    "class": _classify(u.hostname or "", u.path or "/"),
                }
            )
        except Exception:
            pass

    url = f"https://www.twitch.tv/{channel}"
    har_path = out_dir / "capture.har"
    saw_gql = False
    saw_usher = False

    with sync_playwright() as p:
        browser_obj = None
        if user_data_dir:
            user_data_dir.mkdir(parents=True, exist_ok=True)
            context = p.chromium.launch_persistent_context(
                str(user_data_dir),
                headless=not headed,
                args=["--disable-quic"],
                ignore_https_errors=True,
                viewport={"width": 1280, "height": 720},
                record_har_path=str(har_path),
            )
            page = context.pages[0] if context.pages else context.new_page()
        else:
            browser_obj = p.chromium.launch(headless=not headed, args=["--disable-quic"])
            context = browser_obj.new_context(
                record_har_path=str(har_path),
                ignore_https_errors=True,
                viewport={"width": 1280, "height": 720},
            )
            page = context.new_page()

        page.on("request", on_request)
        page.on("response", on_response)
        page.goto(url, wait_until="domcontentloaded", timeout=int(timeout_sec * 1000))

        deadline = time.time() + timeout_sec
        while time.time() < deadline:
            classes = {e.get("class") for e in events}
            saw_gql = saw_gql or "gql" in classes
            saw_usher = saw_usher or "usher" in classes or "m3u8" in classes
            if saw_gql and saw_usher and (time.time() - t0) >= min(duration_sec, 15):
                break
            page.wait_for_timeout(500)

        remain = duration_sec - (time.time() - t0)
        if remain > 0:
            page.wait_for_timeout(int(min(remain, 300) * 1000))

        context.close()
        if browser_obj:
            browser_obj.close()

    network_path = out_dir / "network.json"
    network_path.write_text(json.dumps(events, indent=2), encoding="utf-8")

    return {
        "channel": channel,
        "url": url,
        "har": str(har_path),
        "network": str(network_path),
        "events": len(events),
        "saw_gql": saw_gql,
        "saw_usher_or_m3u8": saw_usher,
        "elapsed_sec": round(time.time() - t0, 2),
    }
