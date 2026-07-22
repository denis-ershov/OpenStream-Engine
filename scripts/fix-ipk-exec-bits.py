#!/usr/bin/env python3
"""Force 0755 on binaries inside an OpenWrt .ipk (gzip/tar), fixing Windows/Git-bash packs."""
from __future__ import annotations

import gzip
import io
import os
import sys
import tarfile
import tempfile


EXEC_PREFIXES = (
    "usr/bin/",
    "usr/libexec/",
    "etc/init.d/",
    "etc/uci-defaults/",
    "./usr/bin/",
    "./usr/libexec/",
    "./etc/init.d/",
    "./etc/uci-defaults/",
)
EXEC_CONTROL = {"postinst", "prerm", "postrm", "./postinst", "./prerm", "./postrm"}


def _want_exec(name: str) -> bool:
    n = name.lstrip("./")
    if n in ("postinst", "prerm", "postrm") or name in EXEC_CONTROL:
        return True
    return any(name.startswith(p) or n.startswith(p.lstrip("./")) for p in EXEC_PREFIXES)


def _fix_inner_tar(data: bytes, force_exec: bool) -> bytes:
    in_buf = io.BytesIO(data)
    out_buf = io.BytesIO()
    with tarfile.open(fileobj=in_buf, mode="r:*") as tin, tarfile.open(
        fileobj=out_buf, mode="w"
    ) as tout:
        for m in tin.getmembers():
            if force_exec and _want_exec(m.name) and m.isfile():
                m.mode = 0o755
            elif m.isdir():
                m.mode = 0o755
            elif m.isfile() and (m.mode & 0o111) == 0:
                m.mode = 0o644
            f = tin.extractfile(m) if m.isfile() else None
            tout.addfile(m, f)
    return out_buf.getvalue()


def fix_ipk(path: str) -> None:
    with gzip.open(path, "rb") as gz:
        outer_raw = gz.read()
    outer_in = io.BytesIO(outer_raw)
    parts: dict[str, bytes] = {}
    with tarfile.open(fileobj=outer_in, mode="r:") as outer:
        for m in outer.getmembers():
            if not m.isfile():
                continue
            f = outer.extractfile(m)
            assert f is not None
            parts[m.name.lstrip("./")] = f.read()

    if "data.tar.gz" not in parts or "control.tar.gz" not in parts:
        raise SystemExit(f"not an OpenWrt ipk layout: {path}")

    data_raw = gzip.decompress(parts["data.tar.gz"])
    data_fixed = _fix_inner_tar(data_raw, force_exec=True)
    parts["data.tar.gz"] = gzip.compress(data_fixed, compresslevel=9)

    ctrl_raw = gzip.decompress(parts["control.tar.gz"])
    ctrl_fixed = _fix_inner_tar(ctrl_raw, force_exec=True)
    parts["control.tar.gz"] = gzip.compress(ctrl_fixed, compresslevel=9)

    out_buf = io.BytesIO()
    with tarfile.open(fileobj=out_buf, mode="w") as outer:
        for name in ("debian-binary", "data.tar.gz", "control.tar.gz"):
            blob = parts[name]
            info = tarfile.TarInfo(name=name)
            info.size = len(blob)
            info.mode = 0o644
            info.mtime = 0
            outer.addfile(info, io.BytesIO(blob))

    tmp = path + ".tmp"
    with gzip.open(tmp, "wb", compresslevel=9) as gz:
        gz.write(out_buf.getvalue())
    os.replace(tmp, path)
    print(f"fixed exec bits: {path}")


def check_ipk(path: str) -> None:
    with gzip.open(path, "rb") as g:
        raw = g.read()
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as t:
        data = t.extractfile("data.tar.gz")
        if data is None:
            raise SystemExit(f"no data.tar.gz in {path}")
        blob = data.read()
    inner = gzip.decompress(blob)
    with tarfile.open(fileobj=io.BytesIO(inner), mode="r:") as t:
        for m in t.getmembers():
            if m.name.endswith("usr/bin/streamproxyd"):
                print(f"  {m.name} mode={oct(m.mode)}")
                if (m.mode & 0o111) == 0:
                    raise SystemExit("ERROR: streamproxyd missing +x")
                print("  OK: executable bit present in archive")
                return
    raise SystemExit("ERROR: streamproxyd not found in ipk")


def main() -> None:
    args = sys.argv[1:]
    if not args:
        print(f"usage: {sys.argv[0]} [--check] <file.ipk>...", file=sys.stderr)
        raise SystemExit(2)
    if args[0] == "--check":
        if len(args) < 2:
            raise SystemExit(2)
        check_ipk(args[1])
        return
    for p in args:
        fix_ipk(p)


if __name__ == "__main__":
    main()
