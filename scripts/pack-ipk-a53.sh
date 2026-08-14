#!/usr/bin/env bash
# Cross-build Cortex-A53 (aarch64 musl) + pack OpenWrt 24.x .ipk (opkg).
# Outer format: gzip(tar) — matches OpenWrt 24.10 scripts/ipkg-build (NOT classic ar/.deb).
# Usage: ./scripts/pack-ipk-a53.sh [--slim]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export LC_ALL=C
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}"
export PKG_SOURCE_DATE_EPOCH="${PKG_SOURCE_DATE_EPOCH:-$SOURCE_DATE_EPOCH}"

VERSION="${OPENSTREAM_VERSION:-0.4.2}"
RELEASE="${OPENSTREAM_RELEASE:-31}"
ARCH="${OPENSTREAM_IPK_ARCH:-aarch64_cortex-a53}"
TARGET="aarch64-unknown-linux-musl"
IPKG_BUILD="$ROOT/scripts/ipkg-build"
SLIM=0

for arg in "$@"; do
  case "$arg" in
    --slim) SLIM=1 ;;
    -h|--help)
      echo "Usage: $0 [--slim]"
      exit 0
      ;;
  esac
done

DIST="$ROOT/dist/openwrt-24.10-a53"
BIN_OUT="$DIST/bin"
IPK_OUT="$DIST/ipk"
mkdir -p "$BIN_OUT" "$IPK_OUT"

FEATURE_ARGS=()
SUFFIX=""
if [[ "$SLIM" -eq 1 ]]; then
  FEATURE_ARGS=(--no-default-features --features slim-twitch)
  SUFFIX="-slim"
fi

SRC_BIN="$ROOT/target/$TARGET/release/streamproxyd"

if [[ "${SKIP_BUILD:-0}" == "1" ]]; then
  echo "==> SKIP_BUILD=1 — using existing $SRC_BIN"
elif [[ -x "$SRC_BIN" && "${FORCE_BUILD:-0}" != "1" ]]; then
  echo "==> Reusing existing binary $SRC_BIN (set FORCE_BUILD=1 to rebuild)"
else
  echo "==> Cross-compile streamproxyd ($TARGET)${SUFFIX}"
  export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"
  if [[ -f /usr/local/cargo/bin/cargo-zigbuild ]] || command -v cargo-zigbuild >/dev/null 2>&1; then
    cargo zigbuild --release -p streamproxyd --target "$TARGET" "${FEATURE_ARGS[@]}"
  elif command -v docker >/dev/null 2>&1; then
    TOOLCHAIN_BAK=""
    if [[ -f "$ROOT/rust-toolchain.toml" ]] && grep -q windows-gnu "$ROOT/rust-toolchain.toml"; then
      TOOLCHAIN_BAK="$ROOT/rust-toolchain.toml.hostbak"
      mv "$ROOT/rust-toolchain.toml" "$TOOLCHAIN_BAK"
    fi
    docker run --rm --entrypoint bash \
      -v "$ROOT:/work" \
      -w /work \
      -e CARGO_HOME=/work/.cargo-docker \
      -e PATH="/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
      messense/cargo-zigbuild:latest \
      -c "cargo zigbuild --release -p streamproxyd --target $TARGET ${FEATURE_ARGS[*]:-}"
    if [[ -n "$TOOLCHAIN_BAK" && -f "$TOOLCHAIN_BAK" ]]; then
      mv "$TOOLCHAIN_BAK" "$ROOT/rust-toolchain.toml"
    fi
  else
    rustup target add "$TARGET"
    cargo zigbuild --release -p streamproxyd --target "$TARGET" "${FEATURE_ARGS[@]}"
  fi
fi

if [[ ! -f "$SRC_BIN" ]]; then
  echo "ERROR: missing $SRC_BIN" >&2
  exit 1
fi

if [[ "${SKIP_BUILD:-0}" == "1" && -f "$BIN_OUT/streamproxyd${SUFFIX}" ]]; then
  echo "==> Using staged $BIN_OUT/streamproxyd${SUFFIX}"
else
  cp -f "$SRC_BIN" "$BIN_OUT/streamproxyd${SUFFIX}"
  chmod +x "$BIN_OUT/streamproxyd${SUFFIX}"
fi
ls -lh "$BIN_OUT/streamproxyd${SUFFIX}"
file "$BIN_OUT/streamproxyd${SUFFIX}" || true

write_default_scripts() {
  local ctrl="$1"
  cat > "$ctrl/postinst" <<'EOF'
#!/bin/sh
[ "${IPKG_NO_SCRIPT}" = "1" ] && exit 0
[ -s "${IPKG_INSTROOT}/lib/functions.sh" ] || exit 0
. "${IPKG_INSTROOT}/lib/functions.sh"
default_postinst "$0" "$@"
[ -n "${IPKG_INSTROOT}" ] && exit 0
# Windows/Git-bash tar иногда теряет +x — принудительно
chmod 0755 /usr/bin/streamproxyd 2>/dev/null || true
chmod 0755 /usr/libexec/openstream-* 2>/dev/null || true
chmod 0755 /etc/init.d/streamproxyd 2>/dev/null || true
rm -f /tmp/luci-indexcache* /tmp/luci-modulecache/* 2>/dev/null
[ -x /usr/libexec/openstream-refresh-opkg-list ] && /usr/libexec/openstream-refresh-opkg-list
[ -x /etc/init.d/rpcd ] && /etc/init.d/rpcd reload >/dev/null 2>&1
[ -x /etc/init.d/streamproxyd ] && /etc/init.d/streamproxyd enable 2>/dev/null || true
# Очищаем /etc/dnsmasq.conf от любых старых строк OpenStream
if [ -f /etc/dnsmasq.conf ]; then
  sed -i '/openstream/d' /etc/dnsmasq.conf 2>/dev/null || true
fi
# Исправляем сломанный confdir если он был добавлен предыдущими версиями OpenStream
if uci -q get dhcp.@dnsmasq[0].confdir | grep -q "dnsmasq.d" 2>/dev/null; then
  logger -t openstream-postinst "Removing broken confdir entries from dhcp config"
  uci -q delete dhcp.@dnsmasq[0].confdir 2>/dev/null
  uci -q commit dhcp 2>/dev/null
fi
/etc/init.d/dnsmasq restart >/dev/null 2>&1 || true
exit 0
EOF
  cat > "$ctrl/prerm" <<'EOF'
#!/bin/sh
[ -s "${IPKG_INSTROOT}/lib/functions.sh" ] || exit 0
. "${IPKG_INSTROOT}/lib/functions.sh"
default_prerm "$0" "$@"
# Очищаем точечные записи OpenStream из /etc/hosts без затрагивания остального файла
if [ -f /etc/hosts ] && grep -q "# openstream" /etc/hosts 2>/dev/null; then
  sed -i '/# openstream/d' /etc/hosts 2>/dev/null || true
  killall -HUP dnsmasq 2>/dev/null || true
fi
EOF
  chmod 0755 "$ctrl/postinst" "$ctrl/prerm"
}

# Embed Packages index so LuCI Installed tab can show Size + Description.
# (status file omits Description; LuCI falls back to available lists.)
install_opkg_list_meta() {
  local pkg_dir="$1"
  [[ -f "$IPK_OUT/Packages" ]] || return 0
  mkdir -p "$pkg_dir/usr/share/openstream/opkg" "$pkg_dir/usr/libexec"
  cp -f "$IPK_OUT/Packages" "$pkg_dir/usr/share/openstream/opkg/Packages"
  if [[ -f "$IPK_OUT/Packages.gz" ]]; then
    cp -f "$IPK_OUT/Packages.gz" "$pkg_dir/usr/share/openstream/opkg/Packages.gz"
  else
    gzip -9nc "$IPK_OUT/Packages" > "$pkg_dir/usr/share/openstream/opkg/Packages.gz"
  fi
  install -m 0755 "$ROOT/package/openwrt/files/openstream-refresh-opkg-list" \
    "$pkg_dir/usr/libexec/openstream-refresh-opkg-list"
}

# OpenWrt 24 layout: pkg_dir/{files..., CONTROL/{control,conffiles,postinst,prerm}}
# Outer .ipk via scripts/ipkg-build → gzip(tar), not ar.
#
# Do NOT put Size: inside CONTROL — opkg verifies local .ipk size against it and
# fails with "Checksum or size mismatch" when Size ≠ final file (Size is for
# Packages index only; see write_packages_index).
pack_via_ipkg_build() {
  local pkg_dir="$1"
  local dest="$2"
  chmod +x "$IPKG_BUILD"
  # Force Unix exec bits before tar (NTFS/Git-bash often drops them in the archive)
  if [[ -d "$pkg_dir/usr/bin" ]]; then
    find "$pkg_dir/usr/bin" -type f -exec chmod 0755 {} \;
  fi
  if [[ -d "$pkg_dir/usr/libexec" ]]; then
    find "$pkg_dir/usr/libexec" -type f -exec chmod 0755 {} \;
  fi
  if [[ -d "$pkg_dir/etc/init.d" ]]; then
    find "$pkg_dir/etc/init.d" -type f -exec chmod 0755 {} \;
  fi
  if [[ -d "$pkg_dir/etc/uci-defaults" ]]; then
    find "$pkg_dir/etc/uci-defaults" -type f -exec chmod 0755 {} \;
  fi
  find "$pkg_dir/CONTROL" -type f \( -name postinst -o -name prerm -o -name postrm \) -exec chmod 0755 {} \; 2>/dev/null || true
  # Strip any leftover Size from previous experiments
  sed -i -e '/^Size:/d' "$pkg_dir/CONTROL/control" 2>/dev/null || true
  SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-0}" PKG_SOURCE_DATE_EPOCH="${PKG_SOURCE_DATE_EPOCH:-0}" \
    "$IPKG_BUILD" "$pkg_dir" "$dest"
  # Git-bash/NTFS tar often stores bins as 0644 — rewrite modes in the .ipk
  local built
  built="$(ls -1t "$dest"/*.ipk 2>/dev/null | head -n 1 || true)"
  if [[ -n "$built" && -f "$ROOT/scripts/fix-ipk-exec-bits.py" ]]; then
    python3 "$ROOT/scripts/fix-ipk-exec-bits.py" "$built" 2>/dev/null \
      || python "$ROOT/scripts/fix-ipk-exec-bits.py" "$built" 2>/dev/null \
      || true
  fi
}

# Feed-style index so LuCI can show Size; not embedded in .ipk control.
write_packages_index() {
  local dir="$1"
  local out="$dir/Packages"
  : > "$out"
  local ipk name
  for ipk in "$dir"/*.ipk; do
    [[ -f "$ipk" ]] || continue
    local stage ctrl
    stage="$(mktemp -d)"
    gzip -dc "$ipk" | tar -C "$stage" -xf - 2>/dev/null || { rm -rf "$stage"; continue; }
    mkdir -p "$stage/ctrl"
    gzip -dc "$stage/control.tar.gz" | tar -C "$stage/ctrl" -xf -
    ctrl="$stage/ctrl/control"
    [[ -f "$ctrl" ]] || { rm -rf "$stage"; continue; }
    cat "$ctrl" >> "$out"
    echo "Filename: $(basename "$ipk")" >> "$out"
    echo "Size: $(wc -c < "$ipk" | tr -d ' ')" >> "$out"
    if command -v sha256sum >/dev/null 2>&1; then
      echo "SHA256sum: $(sha256sum "$ipk" | awk '{print $1}')" >> "$out"
    fi
    echo "" >> "$out"
    rm -rf "$stage"
  done
  gzip -9nc "$out" > "$out.gz"
  echo "Wrote $out and $out.gz"
}

PKG_NAME="openstream-engine"
if [[ -n "$SUFFIX" ]]; then
  PKG_NAME="openstream-engine${SUFFIX}"
fi

ensure_po2lmo() {
  PO2LMO="$ROOT/scripts/bin/po2lmo"
  LMO_CACHE="$ROOT/dist/openwrt-24.10-a53/cache/openstream.ru.lmo"
  mkdir -p "$ROOT/scripts/bin" "$(dirname "$LMO_CACHE")"

  use_lmo_cache() {
    if [[ ! -f "$LMO_CACHE" ]]; then
      local old stage
      old="$(ls -1t "$IPK_OUT"/luci-i18n-openstream-ru_*.ipk 2>/dev/null | head -n 1 || true)"
      if [[ -n "$old" && -f "$old" ]]; then
        stage="$(mktemp -d)"
        gzip -dc "$old" | tar -C "$stage" -xf -
        gzip -dc "$stage/data.tar.gz" | tar -C "$stage" -xf -
        cp -f "$stage/usr/lib/lua/luci/i18n/openstream.ru.lmo" "$LMO_CACHE"
        rm -rf "$stage"
      fi
    fi
    if [[ -f "$LMO_CACHE" ]]; then
      echo "==> Using cached $LMO_CACHE"
      PO2LMO=""
      return 0
    fi
    return 1
  }

  if [[ -f "$ROOT/scripts/po2lmo.py" ]]; then
    python3 "$ROOT/scripts/po2lmo.py" "$ROOT/luci-app-openstream/po/ru/openstream.po" "$LMO_CACHE" 2>/dev/null \
      || python "$ROOT/scripts/po2lmo.py" "$ROOT/luci-app-openstream/po/ru/openstream.po" "$LMO_CACHE" 2>/dev/null \
      || true
  fi

  # Git Bash on Windows cannot run Linux ELF po2lmo
  case "$(uname -s 2>/dev/null || echo unknown)" in
    MINGW*|MSYS*|CYGWIN*)
      use_lmo_cache || { echo "ERROR: need $LMO_CACHE on Windows host" >&2; exit 1; }
      return 0
      ;;
  esac

  if [[ -f "$ROOT/scripts/po2lmo/po2lmo" ]]; then
    cp -f "$ROOT/scripts/po2lmo/po2lmo" "$PO2LMO"
  fi
  chmod +x "$PO2LMO" 2>/dev/null || true
  if [[ -x "$PO2LMO" ]]; then
    return 0
  fi
  if command -v make >/dev/null 2>&1; then
    make -C "$ROOT/scripts/po2lmo" clean po2lmo
    cp -f "$ROOT/scripts/po2lmo/po2lmo" "$PO2LMO"
    chmod +x "$PO2LMO"
    return 0
  fi
  use_lmo_cache || { echo "ERROR: po2lmo unavailable" >&2; exit 1; }
}

# with_feed=0|1 — embed Packages.gz for LuCI Size/Description
pack_engine() {
  local with_feed="${1:-0}"
  echo "==> Pack ${PKG_NAME} .ipk ($ARCH) [feed=$with_feed]"
  local ENGINE_PKG
  ENGINE_PKG="$(mktemp -d)"
  mkdir -p \
    "$ENGINE_PKG/usr/bin" \
    "$ENGINE_PKG/usr/libexec" \
    "$ENGINE_PKG/etc/init.d" \
    "$ENGINE_PKG/etc/config" \
    "$ENGINE_PKG/etc/openstream" \
    "$ENGINE_PKG/etc/uci-defaults" \
    "$ENGINE_PKG/usr/share/openstream/nft" \
    "$ENGINE_PKG/usr/share/openstream/hostlists" \
    "$ENGINE_PKG/CONTROL"

  install -m 0755 "$BIN_OUT/streamproxyd${SUFFIX}" "$ENGINE_PKG/usr/bin/streamproxyd"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-uci2yaml" "$ENGINE_PKG/usr/libexec/openstream-uci2yaml"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-compose-hostlist" "$ENGINE_PKG/usr/libexec/openstream-compose-hostlist"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-update-hostlists" "$ENGINE_PKG/usr/libexec/openstream-update-hostlists"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-refresh-hls-set" "$ENGINE_PKG/usr/libexec/openstream-refresh-hls-set"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-refresh-opkg-list" "$ENGINE_PKG/usr/libexec/openstream-refresh-opkg-list"
  install -m 0755 "$ROOT/package/openwrt/files/openstream-resolve-smartdns" "$ENGINE_PKG/usr/libexec/openstream-resolve-smartdns"
  install -m 0755 "$ROOT/package/openwrt/files/streamproxyd.init" "$ENGINE_PKG/etc/init.d/streamproxyd"
  install -m 0644 "$ROOT/package/openwrt/files/openstream.config" "$ENGINE_PKG/etc/config/openstream"
  install -m 0644 "$ROOT/package/openwrt/files/config.yaml" "$ENGINE_PKG/etc/openstream/config.yaml"
  install -m 0644 "$ROOT/package/openwrt/files/openstream.nft" "$ENGINE_PKG/usr/share/openstream/nft/openstream.nft"
  install -m 0644 "$ROOT/package/openwrt/files/hostlist-hls.txt" "$ENGINE_PKG/usr/share/openstream/hostlist-hls.txt"
  install -m 0644 "$ROOT/package/openwrt/files/hostlists/"*.txt "$ENGINE_PKG/usr/share/openstream/hostlists/"
  install -m 0644 "$ROOT/package/openwrt/files/dnsmasq-openstream.conf" "$ENGINE_PKG/usr/share/openstream/dnsmasq-openstream.conf"
  install -m 0755 "$ROOT/package/openwrt/files/uci-defaults-openstream-transparent" \
    "$ENGINE_PKG/etc/uci-defaults/41_openstream-transparent"

  cat > "$ENGINE_PKG/CONTROL/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}-${RELEASE}
Depends: ca-bundle
License: MIT
Section: net
Architecture: ${ARCH}
Installed-Size: 0
Description: OpenStream Engine Playlist Edge (Twitch strip, no CA by default)
EOF
  printf '%s\n' \
    /etc/config/openstream \
    /etc/openstream/config.yaml \
    > "$ENGINE_PKG/CONTROL/conffiles"
  write_default_scripts "$ENGINE_PKG/CONTROL"
  [[ "$with_feed" == "1" ]] && install_opkg_list_meta "$ENGINE_PKG"

  pack_via_ipkg_build "$ENGINE_PKG" "$IPK_OUT"
  rm -rf "$ENGINE_PKG"
  ls -lh "$IPK_OUT/${PKG_NAME}_${VERSION}-${RELEASE}_${ARCH}.ipk"
}

pack_luci() {
  echo "==> Pack luci-app-openstream .ipk"
  local LUCI_PKG
  LUCI_PKG="$(mktemp -d)"
  mkdir -p \
    "$LUCI_PKG/usr/lib/lua/luci/controller" \
    "$LUCI_PKG/usr/lib/lua/luci/model/cbi/openstream" \
    "$LUCI_PKG/usr/lib/lua/luci/view/openstream" \
    "$LUCI_PKG/usr/share/luci/menu.d" \
    "$LUCI_PKG/usr/share/rpcd/acl.d" \
    "$LUCI_PKG/CONTROL"

  install -m 0644 "$ROOT/luci-app-openstream/luasrc/controller/openstream.lua" \
    "$LUCI_PKG/usr/lib/lua/luci/controller/openstream.lua"
  install -m 0644 "$ROOT/luci-app-openstream/luasrc/model/cbi/openstream/"*.lua \
    "$LUCI_PKG/usr/lib/lua/luci/model/cbi/openstream/"
  install -m 0644 "$ROOT/luci-app-openstream/luasrc/view/openstream/"*.htm \
    "$LUCI_PKG/usr/lib/lua/luci/view/openstream/"
  install -m 0644 "$ROOT/luci-app-openstream/root/usr/share/luci/menu.d/luci-app-openstream.json" \
    "$LUCI_PKG/usr/share/luci/menu.d/luci-app-openstream.json"
  install -m 0644 "$ROOT/luci-app-openstream/root/usr/share/rpcd/acl.d/luci-app-openstream.json" \
    "$LUCI_PKG/usr/share/rpcd/acl.d/luci-app-openstream.json"
  mkdir -p "$LUCI_PKG/etc/uci-defaults"
  install -m 0755 "$ROOT/luci-app-openstream/root/etc/uci-defaults/40_luci-openstream" \
    "$LUCI_PKG/etc/uci-defaults/40_luci-openstream"

  cat > "$LUCI_PKG/CONTROL/control" <<EOF
Package: luci-app-openstream
Version: ${VERSION}-${RELEASE}
Depends: luci-base, openstream-engine
License: MIT
Section: luci
Architecture: all
Installed-Size: 0
Description: LuCI web UI for OpenStream Engine (English). Optional: luci-i18n-openstream-ru
EOF
  write_default_scripts "$LUCI_PKG/CONTROL"
  # opkg meta (Packages.gz) — только в openstream-engine, иначе check_data_file_clashes

  pack_via_ipkg_build "$LUCI_PKG" "$IPK_OUT"
  rm -rf "$LUCI_PKG"
}

pack_i18n() {
  echo "==> Pack luci-i18n-openstream-ru"
  ensure_po2lmo
  local I18N_PKG
  I18N_PKG="$(mktemp -d)"
  mkdir -p \
    "$I18N_PKG/usr/lib/lua/luci/i18n" \
    "$I18N_PKG/etc/uci-defaults" \
    "$I18N_PKG/CONTROL"

  if [[ -n "${PO2LMO:-}" && -x "$PO2LMO" ]]; then
    "$PO2LMO" "$ROOT/luci-app-openstream/po/ru/openstream.po" \
      "$I18N_PKG/usr/lib/lua/luci/i18n/openstream.ru.lmo"
    cp -f "$I18N_PKG/usr/lib/lua/luci/i18n/openstream.ru.lmo" \
      "$ROOT/dist/openwrt-24.10-a53/cache/openstream.ru.lmo"
  else
    install -m 0644 "$ROOT/dist/openwrt-24.10-a53/cache/openstream.ru.lmo" \
      "$I18N_PKG/usr/lib/lua/luci/i18n/openstream.ru.lmo"
  fi

  cat > "$I18N_PKG/etc/uci-defaults/luci-i18n-openstream-ru" <<'EOF'
#!/bin/sh
uci -q batch <<-EOC
	set luci.languages.ru='Русский'
	commit luci
EOC
exit 0
EOF
  chmod 0755 "$I18N_PKG/etc/uci-defaults/luci-i18n-openstream-ru"

  cat > "$I18N_PKG/CONTROL/control" <<EOF
Package: luci-i18n-openstream-ru
Version: ${VERSION}-${RELEASE}
Depends: luci-app-openstream
License: MIT
Section: luci
Architecture: all
Installed-Size: 0
Description: Russian translation for luci-app-openstream
EOF
  write_default_scripts "$I18N_PKG/CONTROL"

  pack_via_ipkg_build "$I18N_PKG" "$IPK_OUT"
  rm -rf "$I18N_PKG"
  ls -lh "$IPK_OUT/luci-i18n-openstream-ru_${VERSION}-${RELEASE}_all.ipk"
}

# Pass 1: packages without embedded index (accurate Size for index)
# Удалить ВСЕ предыдущие .ipk этой линейки — в dist только текущий RELEASE
echo "==> Cleaning old .ipk in $IPK_OUT (keep only ${VERSION}-${RELEASE})"
find "$IPK_OUT" -maxdepth 1 -type f \( -name '*.ipk' -o -name 'Packages*' \) -delete 2>/dev/null || true
rm -f "$IPK_OUT"/Packages "$IPK_OUT"/Packages.gz 2>/dev/null || true

pack_engine 0
pack_luci
pack_i18n

write_packages_index "$IPK_OUT"

# Pass 2–3: embed Packages.gz ONLY in engine (no clash with luci-app)
echo "==> Pass 2: embed opkg list meta into engine only"
pack_engine 1
pack_luci
pack_i18n
write_packages_index "$IPK_OUT"

echo "==> Pass 3: stabilize Size in embedded Packages"
pack_engine 1
pack_luci
pack_i18n
# Снова вычистить чужие версии, если появились
find "$IPK_OUT" -maxdepth 1 -type f -name '*.ipk' ! -name "*_${VERSION}-${RELEASE}_*" -delete 2>/dev/null || true
write_packages_index "$IPK_OUT"

# Fix exec bits (Windows/Git-bash tar stores 0644) then verify
echo "==> Fix +x inside .ipk archives"
python "$ROOT/scripts/fix-ipk-exec-bits.py" "$IPK_OUT"/*.ipk \
  || python3 "$ROOT/scripts/fix-ipk-exec-bits.py" "$IPK_OUT"/*.ipk
write_packages_index "$IPK_OUT"

echo "==> Verify +x on streamproxyd inside .ipk"
eng_ipk="$IPK_OUT/openstream-engine_${VERSION}-${RELEASE}_${ARCH}.ipk"
python "$ROOT/scripts/fix-ipk-exec-bits.py" --check "$eng_ipk" \
  || python3 "$ROOT/scripts/fix-ipk-exec-bits.py" --check "$eng_ipk"

(
  cd "$DIST"
  sha256sum bin/* ipk/* > SHA256SUMS 2>/dev/null || shasum -a 256 bin/* ipk/* > SHA256SUMS
)

echo ""
echo "Verify (must be gzip, NOT 'Debian binary package' / ar):"
file "$IPK_OUT"/*.ipk || true
echo "Artifacts (current release only):"
find "$DIST/ipk" -type f | sort
echo "Done. Release ${VERSION}-${RELEASE}"
echo "After install: chmod is forced in postinst; if Permission denied: chmod 0755 /usr/bin/streamproxyd"
