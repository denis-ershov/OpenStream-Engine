# OpenStream Engine — OpenWrt package
#
# Include via feed (see docs/BUILD_OPENWRT.md) or copy into package/.
# Rust binary is prebuilt (host zigbuild / CI artifact), not compiled in buildroot.

include $(TOPDIR)/rules.mk

PKG_NAME:=openstream-engine
PKG_VERSION:=0.4.2
PKG_RELEASE:=37

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
  # Зависимости обязательны для заявленной функциональности:
  #  * dnsmasq-full    — единственная сборка с поддержкой `nftset=` (базовый
  #                      dnsmasq её НЕ умеет, без неё не наполняется ни один сет);
  #  * kmod-nft-queue  — NFQUEUE 1088 для Zapret2 (nfqws2);
  #  * kmod-nft-tproxy — statement `tproxy` в mangle_prerouting для sing-box;
  #  * ip-full         — `ip rule` / `ip route` для policy routing по fwmark
  #                      (busybox ip не гарантирует поддержку `rule`);
  #  * ucode-mod-*     — модули fs/uci, импортируемые openstream-render.uc.
  DEPENDS:=+ca-bundle +dnsmasq-full +kmod-nft-queue +kmod-nft-tproxy +ip-full \
	   +ucode +ucode-mod-fs +ucode-mod-uci
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
	# Оркестратор обновлений и автообновления по cron
	$(INSTALL_BIN) ./files/openstream-update $(1)/usr/libexec/openstream-update
	# Рендерер конфигураций (ucode). Ранее не устанавливался, из-за чего init
	# всегда уходил в fallback, а RPC вызывал несуществующий файл.
	$(INSTALL_BIN) ./files/openstream-render.uc $(1)/usr/libexec/openstream-render.uc
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/streamproxyd.init $(1)/etc/init.d/streamproxyd
	$(INSTALL_DIR) $(1)/etc/config
	$(INSTALL_CONF) ./files/openstream.config $(1)/etc/config/openstream
	$(INSTALL_DIR) $(1)/etc/openstream
	$(INSTALL_CONF) ./files/config.yaml $(1)/etc/openstream/config.yaml
	# Каталог декларативных правил. Единый путь для init, RPC и рендерера:
	# /etc/openstream/rules (ранее init читал /usr/share/openstream/rules,
	# куда правила не устанавливались вообще).
	$(INSTALL_DIR) $(1)/etc/openstream/rules
	$(INSTALL_CONF) ../../rules/streaming/*.osrule.yaml $(1)/etc/openstream/rules/
	$(INSTALL_CONF) ../../rules/privacy/*.osrule.yaml $(1)/etc/openstream/rules/
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
