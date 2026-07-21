# OpenStream Engine — OpenWrt package
#
# Include via feed (see docs/BUILD_OPENWRT.md) or copy into package/.
# Rust binary is prebuilt (host zigbuild / CI artifact), not compiled in buildroot.

include $(TOPDIR)/rules.mk

PKG_NAME:=openstream-engine
PKG_VERSION:=0.4.2
PKG_RELEASE:=1

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
  Modular HLS/DASH manifest processing for OpenWrt.
  Twitch/Kick/Trovo/YouTube/DASH plugins. Explicit proxy by default.
  Uses own nft table inet openstream — does not touch zapret/podkop.
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
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/streamproxyd.init $(1)/etc/init.d/streamproxyd
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) ./files/openstream.config $(1)/etc/config/openstream
	$(INSTALL_DIR) $(1)/etc/openstream
	$(INSTALL_CONF) ./files/config.yaml $(1)/etc/openstream/config.yaml
	$(INSTALL_DIR) $(1)/usr/share/openstream/nft
	$(INSTALL_DATA) ./files/openstream.nft $(1)/usr/share/openstream/nft/openstream.nft
endef

$(eval $(call BuildPackage,openstream-engine))
