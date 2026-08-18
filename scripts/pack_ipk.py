#!/usr/bin/env python3
"""Cross-platform Python IPK pack script for OpenStream Engine packages."""
import gzip
import io
import os
import tarfile
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "dist" / "openwrt-24.10-a53"
IPK_OUT = DIST / "ipk"
BIN_OUT = DIST / "bin"
VERSION = "0.4.2"
RELEASE = "35"
ARCH = "aarch64_cortex-a53"

def make_tar_gz(entries: list[tuple[str, bytes, int]]) -> bytes:
    """Creates a tar.gz archive from (path, content, mode) entries."""
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        for name, data, mode in entries:
            ti = tarfile.TarInfo(name=name.lstrip("/"))
            ti.size = len(data)
            ti.mode = mode
            ti.mtime = 0
            tar.addfile(ti, io.BytesIO(data))
    return buf.getvalue()

def pack_ipk(output_ipk: Path, control_entries: list[tuple[str, bytes, int]], data_entries: list[tuple[str, bytes, int]]):
    ctrl_gz = make_tar_gz(control_entries)
    data_gz = make_tar_gz(data_entries)
    deb_bin = b"2.0\n"

    outer_buf = io.BytesIO()
    with tarfile.open(fileobj=outer_buf, mode="w") as outer:
        for name, blob in [("debian-binary", deb_bin), ("data.tar.gz", data_gz), ("control.tar.gz", ctrl_gz)]:
            ti = tarfile.TarInfo(name=name)
            ti.size = len(blob)
            ti.mode = 0o644
            ti.mtime = 0
            outer.addfile(ti, io.BytesIO(blob))
    
    # Outer archive is gzip-compressed tar
    ipk_bytes = gzip.compress(outer_buf.getvalue(), compresslevel=9)
    output_ipk.write_bytes(ipk_bytes)
    print(f"Packed: {output_ipk.name} ({len(ipk_bytes)} bytes)")

def collect_file(rel_path: str, local_path: Path, mode: int = 0o644) -> tuple[str, bytes, int]:
    return rel_path, local_path.read_bytes(), mode

def build_packages():
    IPK_OUT.mkdir(parents=True, exist_ok=True)
    BIN_OUT.mkdir(parents=True, exist_ok=True)

    # Delete old release ipks
    for old in IPK_OUT.glob("*.ipk"):
        if f"{VERSION}-{RELEASE}" not in old.name:
            old.unlink()

    # 1. OpenStream Engine Package
    binary_path = ROOT / "package" / "openwrt" / "streamproxyd"
    if not binary_path.exists():
        binary_path = DIST / "bin" / "streamproxyd"
    
    engine_data = [
        collect_file("/usr/bin/streamproxyd", binary_path, 0o755),
        collect_file("/usr/libexec/openstream-uci2yaml", ROOT / "package/openwrt/files/openstream-uci2yaml", 0o755),
        collect_file("/usr/libexec/openstream-compose-hostlist", ROOT / "package/openwrt/files/openstream-compose-hostlist", 0o755),
        collect_file("/usr/libexec/openstream-update-hostlists", ROOT / "package/openwrt/files/openstream-update-hostlists", 0o755),
        collect_file("/usr/libexec/openstream-refresh-hls-set", ROOT / "package/openwrt/files/openstream-refresh-hls-set", 0o755),
        collect_file("/usr/libexec/openstream-refresh-opkg-list", ROOT / "package/openwrt/files/openstream-refresh-opkg-list", 0o755),
        collect_file("/usr/libexec/openstream-resolve-smartdns", ROOT / "package/openwrt/files/openstream-resolve-smartdns", 0o755),
        collect_file("/etc/init.d/streamproxyd", ROOT / "package/openwrt/files/streamproxyd.init", 0o755),
        collect_file("/etc/config/openstream", ROOT / "package/openwrt/files/openstream.config", 0o644),
        collect_file("/etc/openstream/config.yaml", ROOT / "package/openwrt/files/config.yaml", 0o644),
        collect_file("/usr/share/openstream/nft/openstream.nft", ROOT / "package/openwrt/files/openstream.nft", 0o644),
        collect_file("/usr/share/openstream/hostlist-hls.txt", ROOT / "package/openwrt/files/hostlist-hls.txt", 0o644),
        collect_file("/usr/share/openstream/dnsmasq-openstream.conf", ROOT / "package/openwrt/files/dnsmasq-openstream.conf", 0o644),
        collect_file("/etc/uci-defaults/41_openstream-transparent", ROOT / "package/openwrt/files/uci-defaults-openstream-transparent", 0o755),
    ]
    for hl in (ROOT / "package/openwrt/files/hostlists").glob("*.txt"):
        engine_data.append(collect_file(f"/usr/share/openstream/hostlists/{hl.name}", hl, 0o644))

    engine_control = f"""Package: openstream-engine
Version: {VERSION}-{RELEASE}
Depends: ca-bundle
License: MIT
Section: net
Architecture: {ARCH}
Installed-Size: {len(binary_path.read_bytes())}
Description: OpenStream Engine HLS/DASH proxy & Smart Split Router (No CA needed)
""".encode("utf-8")

    engine_postinst = b"""#!/bin/sh
[ -n "${IPKG_INSTROOT}" ] || {
	/etc/init.d/streamproxyd enable 2>/dev/null || true
	/etc/init.d/streamproxyd restart 2>/dev/null || true
}
exit 0
"""
    engine_prerm = b"""#!/bin/sh
[ -n "${IPKG_INSTROOT}" ] || {
	/etc/init.d/streamproxyd stop 2>/dev/null || true
	/etc/init.d/streamproxyd disable 2>/dev/null || true
}
exit 0
"""
    engine_conffiles = b"/etc/config/openstream\n/etc/openstream/config.yaml\n"

    engine_ctrl = [
        ("./control", engine_control, 0o644),
        ("./postinst", engine_postinst, 0o755),
        ("./prerm", engine_prerm, 0o755),
        ("./conffiles", engine_conffiles, 0o644),
    ]
    pack_ipk(IPK_OUT / f"openstream-engine_{VERSION}-{RELEASE}_{ARCH}.ipk", engine_ctrl, engine_data)

    # 2. LuCI App Package
    luci_data = [
        collect_file("/usr/lib/lua/luci/controller/openstream.lua", ROOT / "luci-app-openstream/luasrc/controller/openstream.lua", 0o644),
        collect_file("/usr/lib/lua/luci/model/cbi/openstream/twitch.lua", ROOT / "luci-app-openstream/luasrc/model/cbi/openstream/twitch.lua", 0o644),
        collect_file("/usr/lib/lua/luci/model/cbi/openstream/services.lua", ROOT / "luci-app-openstream/luasrc/model/cbi/openstream/services.lua", 0o644),
        collect_file("/usr/share/luci/menu.d/luci-app-openstream.json", ROOT / "luci-app-openstream/root/usr/share/luci/menu.d/luci-app-openstream.json", 0o644),
        collect_file("/usr/share/rpcd/acl.d/luci-app-openstream.json", ROOT / "luci-app-openstream/root/usr/share/rpcd/acl.d/luci-app-openstream.json", 0o644),
        collect_file("/etc/uci-defaults/40_luci-openstream", ROOT / "luci-app-openstream/root/etc/uci-defaults/40_luci-openstream", 0o755),
    ]
    for htm in (ROOT / "luci-app-openstream/luasrc/view/openstream").glob("*.htm"):
        luci_data.append(collect_file(f"/usr/lib/lua/luci/view/openstream/{htm.name}", htm, 0o644))

    luci_control = f"""Package: luci-app-openstream
Version: {VERSION}-{RELEASE}
Depends: luci-base, openstream-engine
License: MIT
Section: luci
Architecture: all
Installed-Size: 0
Description: LuCI web UI for OpenStream Engine
""".encode("utf-8")

    luci_postinst = b"""#!/bin/sh
[ -n "${IPKG_INSTROOT}" ] || {
	rm -f /tmp/luci-indexcache 2>/dev/null || true
	/etc/init.d/rpcd restart 2>/dev/null || true
}
exit 0
"""
    luci_ctrl = [
        ("./control", luci_control, 0o644),
        ("./postinst", luci_postinst, 0o755),
    ]
    pack_ipk(IPK_OUT / f"luci-app-openstream_{VERSION}-{RELEASE}_all.ipk", luci_ctrl, luci_data)

    # 3. LuCI i18n Package
    lmo_path = ROOT / "luci-app-openstream/root/usr/lib/lua/luci/i18n/openstream.ru.lmo"
    i18n_data = [
        collect_file("/usr/lib/lua/luci/i18n/openstream.ru.lmo", lmo_path, 0o644),
        (
            "/etc/uci-defaults/luci-i18n-openstream-ru",
            "#!/bin/sh\nuci -q batch <<-EOC\n\tset luci.languages.ru='Русский'\n\tcommit luci\nEOC\nexit 0\n".encode("utf-8"),
            0o755
        )
    ]
    i18n_control = f"""Package: luci-i18n-openstream-ru
Version: {VERSION}-{RELEASE}
Depends: luci-app-openstream
License: MIT
Section: luci
Architecture: all
Installed-Size: 0
Description: Russian translation for luci-app-openstream
""".encode("utf-8")

    i18n_ctrl = [
        ("./control", i18n_control, 0o644),
        ("./postinst", luci_postinst, 0o755),
    ]
    pack_ipk(IPK_OUT / f"luci-i18n-openstream-ru_{VERSION}-{RELEASE}_all.ipk", i18n_ctrl, i18n_data)

    # 4. Generate Packages index & SHA256SUMS
    packages_content = []
    for pkg in sorted(IPK_OUT.glob(f"*_{VERSION}-{RELEASE}_*.ipk")):
        data = pkg.read_bytes()
        sha = hashlib.sha256(data).hexdigest()
        size = len(data)
        
        # Read control.tar.gz from IPK
        with gzip.open(pkg, "rb") as gz:
            outer = tarfile.open(fileobj=io.BytesIO(gz.read()), mode="r:")
            ctrl_gz_data = outer.extractfile("control.tar.gz").read()
            with tarfile.open(fileobj=io.BytesIO(gzip.decompress(ctrl_gz_data)), mode="r:") as ctrl_tar:
                ctrl_txt = ctrl_tar.extractfile("./control").read().decode("utf-8").strip()
        
        packages_content.append(f"{ctrl_txt}\nFilename: {pkg.name}\nSize: {size}\nSHA256sum: {sha}\n")

    pkg_index = "\n".join(packages_content) + "\n"
    (IPK_OUT / "Packages").write_text(pkg_index, encoding="utf-8")
    (IPK_OUT / "Packages.gz").write_bytes(gzip.compress(pkg_index.encode("utf-8"), compresslevel=9))
    print(f"Generated Packages index with {len(packages_content)} entries.")

    # SHA256SUMS
    sha_lines = []
    for p in sorted(IPK_OUT.glob("*.ipk")):
        sha_lines.append(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  ipk/{p.name}")
    (DIST / "SHA256SUMS").write_text("\n".join(sha_lines) + "\n", encoding="utf-8")
    print("Updated SHA256SUMS.")

if __name__ == "__main__":
    build_packages()
