#!/bin/sh
# Smoke checklist: transparent Twitch catch on GL-MT6000 (or any OpenWrt).
# Run ON the router after installing openstream-engine with mode=transparent.
# Usage: sh smoke-transparent-mt6000.sh
set -eu

echo "==> daemon"
pidof streamproxyd >/dev/null || {
	echo "FAIL: streamproxyd not running"
	exit 1
}

echo "==> CA"
[ -f /etc/openstream/ca.crt ] || {
	echo "FAIL: missing /etc/openstream/ca.crt — start daemon once / check tls paths"
	exit 1
}
ls -l /etc/openstream/ca.crt

echo "==> mode"
grep -E '^mode:' /etc/openstream/config.yaml || true

echo "==> nft table/set"
nft list table inet openstream 2>/dev/null || echo "WARN: table missing (will be created on start)"
nft list set inet openstream openstream_hls 2>/dev/null || echo "WARN: set missing"

echo "==> refresh hostlist"
if [ -x /usr/libexec/openstream-refresh-hls-set ]; then
	/usr/libexec/openstream-refresh-hls-set || true
	nft list set inet openstream openstream_hls 2>/dev/null | head -40
else
	echo "WARN: refresh script missing"
fi

echo "==> status API"
wget -qO- http://127.0.0.1:18080/api/status | head -c 800
echo

echo "==> metrics (playlists before stream)"
wget -qO- http://127.0.0.1:18080/metrics 2>/dev/null | grep -E 'openstream_playlists|openstream_ads' || true

cat <<'EOF'

Manual steps (client):
  1. Install /etc/openstream/ca.crt as trusted root (no HTTP proxy).
  2. Open twitch.tv in browser, start a stream (zapret OK; podkop: divert before TUN).
  3. On router:
       wget -qO- http://127.0.0.1:18080/metrics | grep playlists
     Expect: openstream_playlists_total > 0

  If still 0:
       nft list set inet openstream openstream_hls
       # add missing CDN IPs to hostlist or enable dnsmasq nftset
       logread -e streamproxyd | grep -iE 'mitm|transparent|sni'
EOF
