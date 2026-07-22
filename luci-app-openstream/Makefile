#
# luci-app-openstream — LuCI UI (English source strings).
# Russian UI: separate package luci-i18n-openstream-ru (auto via luci.mk from po/ru/).
#

include $(TOPDIR)/rules.mk

LUCI_TITLE:=LuCI support for OpenStream Engine
LUCI_DESCRIPTION:=Web UI for OpenStream Engine (Services menu). Install luci-i18n-openstream-ru for Russian.
LUCI_DEPENDS:=+luci-base +openstream-engine
LUCI_PKGARCH:=all

PKG_LICENSE:=MIT
PKG_VERSION:=0.4.2
PKG_RELEASE:=5

include $(TOPDIR)/feeds/luci/luci.mk

# luci.mk builds luci-i18n-openstream-ru from po/ru/*.po automatically.
# call BuildPackage - OpenWrt buildroot signature
