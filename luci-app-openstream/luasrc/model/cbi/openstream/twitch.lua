local m, s, o

local function apply_daemon()
	luci.sys.call("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	luci.sys.call("/etc/init.d/streamproxyd reload >/dev/null 2>&1")
end

m = Map("openstream", translate("Twitch module"),
	translate("Smart Split Routing and Ad-Bypass without client certificates."))

m.on_after_commit = function()
	apply_daemon()
end

s = m:section(NamedSection, "twitch", "module", translate("Twitch Routing Preset"))
s.anonymous = true

o = s:option(Flag, "enabled", translate("Enable Twitch Module"))
o.default = "1"
o.rmempty = false

o = s:option(ListValue, "preset", translate("Routing Preset"))
o:value("ru_smartdns_noads_quality", translate("🇷🇺 Russia / CIS: No ads + 1440p (SmartDNS — No VPN needed) [Recommended]"))
o:value("smartdns_quality_unlock", translate("🌍 Quality Unlock: 1440p/Source (SmartDNS — No VPN needed)"))
o:value("ru_vpn_noads_quality", translate("🛡️ Russia: No ads + 1440p (via Router VPN)"))
o:value("eu_bypass_ads", translate("🇪🇺 Europe / US: Bypass ads via RU DNS"))
o:value("full_bypass", translate("🔒 Full Bypass: Route entire Twitch via VPN"))
o:value("custom", translate("⚙️ Custom: Fine-grained matrix routing"))
o:value("off", translate("⏹️ Disabled"))
o.default = "ru_smartdns_noads_quality"
o.description = translate("Select an automated routing strategy. SmartDNS presets work out-of-the-box without requiring any VPN tunnels.")

-- Custom matrix options (visible only when preset == 'custom')
o = s:option(ListValue, "route_token", translate("Playback Token (gql.twitch.tv)"))
o:depends("preset", "custom")
o:value("dns_yandex", translate("Yandex DNS (RU — No Ads)"))
o:value("dns_mskix", translate("MSK-IX DNS (Fastest RU)"))
o:value("smartdns_comss", translate("Comss.one SmartDNS"))
o:value("dns_cloudflare", translate("Cloudflare DNS (Global)"))
o:value("direct", translate("Direct WAN (Local ISP)"))
o:value("vpn_eu", translate("Main / European VPN"))
o:value("vpn_ru", translate("Russian VPN (No Ads)"))
o.default = "dns_yandex"
o.description = translate("Location where authorization token is requested. RU DNS / IP gives ad-free token.")

o = s:option(ListValue, "route_master", translate("Master Playlist (usher.ttvnw.net)"))
o:depends("preset", "custom")
o:value("smartdns_comss", translate("Comss.one SmartDNS (1440p No VPN)"))
o:value("vpn_eu", translate("Main / European VPN"))
o:value("dns_yandex", translate("Yandex DNS (RU)"))
o:value("dns_cloudflare", translate("Cloudflare DNS (Global)"))
o:value("direct", translate("Direct WAN (Local ISP)"))
o.default = "smartdns_comss"
o.description = translate("Location where quality variants are fetched. SmartDNS / EU VPN unlocks 1080p/1440p/Source.")

o = s:option(ListValue, "route_media", translate("Media Playlists (playlist.ttvnw.net)"))
o:depends("preset", "custom")
o:value("direct", translate("Direct WAN (Local ISP)"))
o:value("smartdns_comss", translate("Comss.one SmartDNS"))
o:value("vpn_eu", translate("Main / European VPN"))
o.default = "direct"

o = s:option(ListValue, "route_segments", translate("Video Streams / CDN (live-video.net, ttvnw.net)"))
o:depends("preset", "custom")
o:value("direct", translate("Direct WAN (Local ISP)"))
o:value("smartdns_comss", translate("Comss.one SmartDNS"))
o:value("vpn_eu", translate("Main / European VPN"))
o.default = "direct"
o.description = translate("Heavy video stream chunks. Direct WAN preserves 100% of your VPN bandwidth and speed.")

o = s:option(ListValue, "route_ads", translate("Banners & Ad Trackers (edge.ads.twitch.tv)"))
o:depends("preset", "custom")
o:value("block", translate("DNS Sinkhole 0.0.0.0 (Block)"))
o:value("direct", translate("Direct (Allow)"))
o.default = "block"
o.description = translate("Blocks banner ads and analytics trackers on router DNS level.")

-- SmartDNS Network Interface
o = s:option(ListValue, "wan_interface", translate("WAN Interface for SmartDNS"))
o:value("auto", translate("Auto-detect (Recommended)"))
local nixio = require "nixio"
local dev_iter = nixio.fs.dir("/sys/class/net")
if dev_iter then
	for dev in dev_iter do
		if dev ~= "lo" and not dev:find("^ifb") then
			o:value(dev, dev)
		end
	end
end
o.default = "auto"
o.description = translate("Network interface used for direct SmartDNS and RU DNS requests in bypass of VPN tunnels.")

-- VPN Routing Configuration Section
s = m:section(TypedSection, "routing", translate("VPN Policy Routing Sets"))
s.anonymous = true
s.description = translate("Specify nftset (OpenWrt 22+) or ipset names configured in your router's VPN/PBR clients.")

o = s:option(Value, "vpn_set_eu", translate("Main / EU VPN Set"))
o.default = "4#inet#fw4#vpn_domains"
o.placeholder = "4#inet#fw4#vpn_domains"
o.description = translate("e.g. 4#inet#fw4#vpn_domains (Sing-box / Clash / Passwall / OpenConnect)")

o = s:option(Value, "vpn_set_ru", translate("Russian VPN Set"))
o.default = "4#inet#fw4#vpn_ru"
o.placeholder = "4#inet#fw4#vpn_ru"
o.description = translate("Used for ad-free token retrieval when you are located outside Russia.")

o = s:option(Value, "vpn_set_custom", translate("Custom VPN Set"))
o.default = "4#inet#fw4#vpn_custom"
o.placeholder = "4#inet#fw4#vpn_custom"

-- Compatibility Settings
s = m:section(NamedSection, "main", "main", translate("Compatibility Settings"))
s.anonymous = true

o = s:option(Flag, "ignore_coexistence_warnings", translate("Ignore Coexistence Warnings"))
o.default = "0"
o.rmempty = false
o.description = translate("Check this if twitch.tv is already placed in Exclude / Bypass list of Forkop, Podkop or PassWall.")

return m
