#!/usr/bin/env python3
"""OpenTwitch Autolab orchestrator: browser → flow map → client E1–E4 → report."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

from browser_capture import capture_channel
from client_e1_e4 import run_client_gates
from har_to_map import build_flow_map, write_flow_artifacts

ROOT = Path(__file__).resolve().parent
RESULTS = ROOT.parent / "results"


def _session_id(channel: str) -> str:
    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"{ts}_{channel.lower()}"


def _write_report(out: Path, payload: dict) -> None:
    gates = payload.get("gates") or {}
    lines = [
        f"# Autolab report — `{payload.get('channel')}`",
        "",
        f"- session: `{payload.get('session_id')}`",
        f"- time: {payload.get('created')}",
        "",
        "## Gates",
        "",
        "| Gate | Status |",
        "|------|--------|",
    ]
    for g in ("E0", "E1", "E2", "E3", "E4"):
        st = (gates.get(g) or {}).get("status", "?")
        lines.append(f"| {g} | **{st}** |")
    lines.append("")

    combo = payload.get("gates", {}).get("combo_routes") or payload.get("combo_routes") or {}
    if combo:
        lines.extend([
            "## Combo Routes (Split Analysis)",
            "",
            "| Route ID | GQL | Usher | Status | Resolutions | Ads? | Token IP | Note |",
            "|----------|-----|-------|--------|-------------|------|----------|------|"
        ])
        route_defs = {
            "R0_direct_all": ("RU (direct)", "RU (direct)", "Чистый РФ путь"),
            "R1_base_geo_split": ("EU (proxy)", "EU (proxy)", "Базовый geo-split"),
            "R3_smart_geo_split": ("RU (direct)", "EU (proxy)", "Оптимальный (Токен RU + Master EU)"),
            "R2_smart_geo_split_reverse": ("EU (proxy)", "RU (direct)", "Реверсивный split")
        }
        for r_id, (gql_loc, usher_loc, note) in route_defs.items():
            res = combo.get(r_id) or {}
            if not res:
                lines.append(f"| `{r_id}` | {gql_loc} | {usher_loc} | *not tested* | | | | {note} |")
                continue
            if not res.get("ok"):
                lines.append(f"| `{r_id}` | {gql_loc} | {usher_loc} | **fail** | - | - | - | {res.get('error', 'Unknown error')} |")
                continue
            resolutions = ", ".join(res.get("resolutions") or [])
            has_ads = "YES" if res.get("has_ads") else "NO"
            ads_str = f"{has_ads} ({', '.join(res.get('ads_markers') or [])})" if res.get("has_ads") else "NO"
            lines.append(
                f"| `{r_id}` | {gql_loc} | {usher_loc} | **ok** | {resolutions} | {ads_str} | `{res.get('token_ip')}` | {note} |"
            )
        lines.append("")

    if payload.get("flow_markdown"):
        lines.append("## Flow map")
        lines.append("")
        lines.append(payload["flow_markdown"])
        lines.append("")
    lines.append("## Raw")
    lines.append("")
    lines.append("See `report.json`, `flow_map.json`, `capture.har`.")
    (out / "REPORT.md").write_text("\n".join(lines), encoding="utf-8")
    (out / "report.json").write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")

    session_md = out / "SESSION.md"
    session_md.write_text(
        "\n".join(
            [
                f"# Session {payload.get('session_id')}",
                "",
                f"- channel: `{payload.get('channel')}`",
                f"- created: {payload.get('created')}",
                f"- browser: {payload.get('browser')}",
                f"- socks5: {payload.get('socks5') or 'none'}",
                "",
                "## Hosts order",
                "",
                "\n".join(f"- `{h}`" for h in (payload.get('hosts_order') or [])),
                "",
            ]
        ),
        encoding="utf-8",
    )


def main() -> int:
    ap = argparse.ArgumentParser(description="OpenTwitch autolab (browser + PC client)")
    ap.add_argument("--channel", default="gohamedia", help="Twitch channel login")
    ap.add_argument("--socks5", default=None, help="socks5://host:port for E1 VPS path")
    ap.add_argument("--browser-only", action="store_true", help="Skip client E1–E4")
    ap.add_argument("--skip-browser", action="store_true", help="Only client gates")
    ap.add_argument("--headed", action="store_true", help="Show browser")
    ap.add_argument("--duration", type=float, default=45.0, help="Seconds to keep page open")
    ap.add_argument("--timeout", type=float, default=90.0, help="Max wait for gql/usher")
    ap.add_argument("--allow-partial", action="store_true", help="E0 pass without gql+usher")
    ap.add_argument("--user-data", type=Path, default=None, help="Persistent Chromium profile")
    args = ap.parse_args()

    sid = _session_id(args.channel)
    out = RESULTS / sid
    out.mkdir(parents=True, exist_ok=True)

    created = datetime.now(timezone.utc).isoformat()
    gates: dict = {}
    flow = {}
    browser_meta = None
    hosts_order: list = []
    flow_md = ""

    if not args.skip_browser:
        print(f"[autolab] browser capture → {args.channel}", flush=True)
        user_data = args.user_data or (ROOT / "user_data")
        try:
            browser_meta = capture_channel(
                args.channel,
                out,
                headed=args.headed,
                duration_sec=args.duration,
                timeout_sec=args.timeout,
                user_data_dir=user_data if args.headed else None,
            )
        except Exception as e:
            print(f"[autolab] browser failed: {e}", file=sys.stderr)
            gates["E0"] = {"status": "fail", "error": str(e)}
            browser_meta = None

        if browser_meta:
            net_path = Path(browser_meta["network"])
            events = json.loads(net_path.read_text(encoding="utf-8")) if net_path.is_file() else []
            har = Path(browser_meta["har"]) if browser_meta.get("har") else None
            flow = build_flow_map(events, har if har and har.is_file() else None)
            write_flow_artifacts(out, flow)
            flow_md = (out / "map_fragment.md").read_text(encoding="utf-8")
            hosts_order = flow.get("hosts_order") or []
            e0_ok = bool(flow.get("has_gql") and flow.get("has_usher")) or args.allow_partial
            gates["E0"] = {
                "status": "pass" if e0_ok else "fail",
                "has_gql": flow.get("has_gql"),
                "has_usher": flow.get("has_usher"),
                "hosts": len(hosts_order),
                "browser": browser_meta,
            }
    else:
        gates["E0"] = {"status": "skipped", "reason": "--skip-browser"}

    combo_routes = {}
    if not args.browser_only:
        print("[autolab] client gates E1–E4", flush=True)
        client = run_client_gates(
            args.channel,
            socks5=args.socks5,
            browser_only=False,
        )
        gates.update(client.get("gates") or {})
        combo_routes = client.get("combo_routes") or {}
    else:
        for g in ("E1", "E2", "E3", "E4"):
            gates.setdefault(g, {"status": "skipped", "reason": "browser-only"})

    payload = {
        "session_id": sid,
        "created": created,
        "channel": args.channel,
        "socks5": args.socks5,
        "browser": browser_meta,
        "hosts_order": hosts_order,
        "flow_markdown": flow_md,
        "gates": gates,
        "combo_routes": combo_routes,
    }
    _write_report(out, payload)

    print(f"[autolab] wrote {out}", flush=True)
    for g in ("E0", "E1", "E2", "E3", "E4"):
        print(f"  {g}: {(gates.get(g) or {}).get('status')}", flush=True)

    # Fail exit if required gates failed (skip ok)
    required = ["E0"]
    if args.socks5 and not args.browser_only:
        required.append("E1")
    failed = [
        g
        for g in required
        if (gates.get(g) or {}).get("status") == "fail"
    ]
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
