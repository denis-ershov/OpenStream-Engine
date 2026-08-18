# OpenStream Engine — OpenWrt package
#
# Include via feed (see docs/BUILD_OPENWRT.md) or copy into package/.
# Rust binary is prebuilt (host zigbuild / CI artifact), not compiled in buildroot.

include $(TOPDIR)/rules.mk

PKG_NAME:=openstream-engine
PKG_VERSION:=0.4.2
PKG_RELEASE:=35

PKG_LICENSE:=MIT
PKG_MAINTAINER:=Denis Ershov

PKG_BUILD_PARALLEL:=1

include $(INCLUDE_DIR)/package.mk

# Path to prebuilt aarch64/arm/mips musl binary (Cortex-A53: aarch64-unknown-linux-musl).
OPENSTREAM_BIN ?= $(CURDIR)/streamproxyd

define Package/openstream-engine
  SECTION:=net
  CATEGORY:=Network
  TITLE:=OpenStream Engine HLS/DASH proxy
  DEPENDS:=+ca-bundle
  URL:=https://github.com/denis-ershov/OpenStream-Engine
endef

define Package/openstream-engine/description
  Playlist Edge HLS/DASH for OpenWrt (no client CA by default).
  Optional transparent MITM divert. Own nft table inet openstream.
endef

define Package/openstream-engine/conffiles
/etc/config/openstream
/etc/openstream/config.yaml
endef

define Build/Prepare
	mkdir -p $(PKG_BUILD_DIR)
	@if [ ! -f "$(OPENSTREAM_BIN)" ]; then \
		echo "ERROR: prebuilt binary not found: $(OPENSTREAM_BIN)"; \
		echo "Build on host:"; \
		echo "  ./scripts/build-a53.sh --copy-pkg"; \
		echo "Or: export OPENSTREAM_BIN=/path/to/streamproxyd"; \
		echo "See docs/BUILD_OPENWRT.md"; \
		exit 1; \
	fi
	$(CP) $(OPENSTREAM_BIN) $(PKG_BUILD_DIR)/streamproxyd
endef

define Build/Compile
	true
endef

define Package/openstream-engine/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_BUILD_DIR)/streamproxyd $(1)/usr/bin/streamproxyd
	$(INSTALL_DIR) $(1)/usr/libexec
	$(INSTALL_BIN) ./files/openstream-uci2yaml $(1)/usr/libexec/openstream-uci2yaml
	$(INSTALL_BIN) ./files/openstream-compose-hostlist $(1)/usr/libexec/openstream-compose-hostlist
	$(INSTALL_BIN) ./files/openstream-update-hostlists $(1)/usr/libexec/openstream-update-hostlists
	$(INSTALL_BIN) ./files/openstream-refresh-hls-set $(1)/usr/libexec/openstream-refresh-hls-set
	$(INSTALL_BIN) ./files/openstream-refresh-opkg-list $(1)/usr/libexec/openstream-refresh-opkg-list
	$(INSTALL_BIN) ./files/openstream-resolve-smartdns $(1)/usr/libexec/openstream-resolve-smartdns
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/streamproxyd.init $(1)/etc/init.d/streamproxyd
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) ./files/openstream.config $(1)/etc/config/openstream
	$(INSTALL_DIR) $(1)/etc/openstream
	$(INSTALL_CONF) ./files/config.yaml $(1)/etc/openstream/config.yaml
	$(INSTALL_DIR) $(1)/usr/share/openstream/nft
	$(INSTALL_DATA) ./files/openstream.nft $(1)/usr/share/openstream/nft/openstream.nft
	$(INSTALL_DATA) ./files/hostlist-hls.txt $(1)/usr/share/openstream/hostlist-hls.txt
	$(INSTALL_DIR) $(1)/usr/share/openstream/hostlists
	$(INSTALL_DATA) ./files/hostlists/*.txt $(1)/usr/share/openstream/hostlists/
	$(INSTALL_DATA) ./files/dnsmasq-openstream.conf $(1)/usr/share/openstream/dnsmasq-openstream.conf
	$(INSTALL_DIR) $(1)/etc/uci-defaults
	$(INSTALL_BIN) ./files/uci-defaults-openstream-transparent $(1)/etc/uci-defaults/41_openstream-transparent
endef

$(eval $(call BuildPackage,openstream-engine))
