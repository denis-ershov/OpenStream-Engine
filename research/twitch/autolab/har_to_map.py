"""HAR / network.json → flow_map + markdown fragment."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


def _classify(host: str, path: str) -> str:
    h = (host or "").lower()
    p = (path or "").lower()
    if "gql.twitch.tv" in h:
        return "gql"
    if "usher.ttvnw.net" in h:
        return "usher"
    if p.endswith((".ts", ".m4s", ".mp4")) or "segment" in p:
        return "segment"
    if "playlist" in h or "live-video.net" in h:
        return "playlist"
    if "video-weaver" in h or "video-edge" in h:
        return "weaver"
    if "ttvnw" in h or "twitch.tv" in h:
        return "twitch_other"
    return "other"


def build_flow_map(network_events: list[dict[str, Any]] | None = None, har_path: Path | None = None) -> dict[str, Any]:
    entries: list[dict[str, Any]] = []
    if network_events:
        for e in network_events:
            if e.get("type") != "response" and e.get("type") != "request":
                continue
            host = e.get("host") or ""
            path = e.get("path") or "/"
            entries.append(
                {
                    "t": e.get("t", 0),
                    "host": host,
                    "path": path,
                    "class": e.get("class") or _classify(host, path),
                    "status": e.get("status"),
                    "method": e.get("method"),
                    "url": e.get("url"),
                }
            )
    if har_path and har_path.is_file():
        har = json.loads(har_path.read_text(encoding="utf-8"))
        for i, ent in enumerate(har.get("log", {}).get("entries", [])):
            req = ent.get("request", {})
            url = req.get("url", "")
            u = urlparse(url)
            entries.append(
                {
                    "t": i * 0.001,
                    "host": u.hostname or "",
                    "path": u.path or "/",
                    "class": _classify(u.hostname or "", u.path or "/"),
                    "status": (ent.get("response") or {}).get("status"),
                    "method": req.get("method"),
                    "url": url,
                }
            )

    first_by_host: dict[str, dict[str, Any]] = {}
    order: list[str] = []
    classes_seen: set[str] = set()
    for e in sorted(entries, key=lambda x: x.get("t") or 0):
        host = e.get("host") or ""
        if not host:
            continue
        classes_seen.add(e.get("class") or "other")
        if host not in first_by_host:
            first_by_host[host] = e
            order.append(host)

    return {
        "hosts_order": order,
        "hosts": {
            h: {
                "class": first_by_host[h].get("class"),
                "first_path": first_by_host[h].get("path"),
                "first_t": first_by_host[h].get("t"),
            }
            for h in order
        },
        "classes": sorted(classes_seen),
        "has_gql": "gql" in classes_seen,
        "has_usher": "usher" in classes_seen or "m3u8" in classes_seen,
        "entry_count": len(entries),
    }


def flow_map_markdown(flow: dict[str, Any]) -> str:
    lines = [
        "| # | Host | Class | First path |",
        "|---|------|-------|------------|",
    ]
    for i, host in enumerate(flow.get("hosts_order") or [], 1):
        info = (flow.get("hosts") or {}).get(host) or {}
        lines.append(
            f"| {i} | `{host}` | {info.get('class')} | `{info.get('first_path')}` |"
        )
    lines.append("")
    lines.append(f"- has_gql: **{flow.get('has_gql')}**")
    lines.append(f"- has_usher/m3u8: **{flow.get('has_usher')}**")
    return "\n".join(lines)


def write_flow_artifacts(out_dir: Path, flow: dict[str, Any]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "flow_map.json").write_text(json.dumps(flow, indent=2), encoding="utf-8")
    (out_dir / "map_fragment.md").write_text(flow_map_markdown(flow), encoding="utf-8")
