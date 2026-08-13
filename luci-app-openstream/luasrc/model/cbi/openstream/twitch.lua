local m, s, o

local function apply_daemon()
	luci.sys.call("/usr/libexec/openstream-compose-hostlist >/dev/null 2>&1")
	luci.sys.call("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	luci.sys.call(
		"wget -qO- --post-data='' http://127.0.0.1:18080/api/reload >/dev/null 2>&1 " ..
		"|| /etc/init.d/streamproxyd reload >/dev/null 2>&1"
	)
end

m = Map("openstream", translate("Twitch plugin"),
	translate("Segment Stripping. Default mode is Playlist Edge (no client CA)."))

m.on_after_commit = function()
	local c = luci.model.uci.cursor()
	if c:get("openstream", "twitch", "enabled") == "1" then
		luci.sys.call(
			"uci -q del_list openstream.proxy.hostlist_services='twitch'; " ..
			"uci -q add_list openstream.proxy.hostlist_services='twitch'; " ..
			"uci -q commit openstream"
		)
	end
	apply_daemon()
end

s = m:section(NamedSection, "twitch", "twitch", translate("Detection"))
s.anonymous = true

o = s:option(Flag, "enabled", translate("Enable"))
o.default = "1"

o = s:option(Flag, "detect_stitched", translate("Detect stitched"))
o.default = "1"

o = s:option(Flag, "detect_daterange", translate("Detect EXT-X-DATERANGE"))
o.default = "1"

o = s:option(Flag, "detect_regex", translate("Regex heuristics"))
o.default = "0"

o = s:option(Value, "max_wait_secs", translate("Maximum wait (sec)"))
o.datatype = "uinteger"
o.default = "30"

o = s:option(Flag, "debug", translate("Debug"))
o.default = "0"

o = s:option(Flag, "backup_seamless", translate("Backup seamless (opt-in scaffold)"))
o.default = "0"
o.description = translate("Does not enable GraphQL/token by default. Strip remains the primary mode.")

s = m:section(NamedSection, "proxy", "proxy", translate("Proxy / Routing"))
s.anonymous = true

o = s:option(ListValue, "mode", translate("Mode"))
o:value("edge", translate("Playlist Edge (no client CA)"))
o:value("geo_split", translate("Smart Geo-Split (R3, Recommended, no CA)"))
o:value("off", translate("Off"))
o.default = "edge"
o.description = translate("Smart Geo-Split: routes only usher.ttvnw.net to VPN, bypassing ads and keeping quality without CA. Edge: local HLS rewrite on the router.")

o = s:option(Value, "geo_split_vpn_set", translate("Smart Geo-Split VPN Set"))
o:depends("mode", "geo_split")
o.default = "4#inet#fw4#vpn_domains"
o.placeholder = "4#inet#fw4#vpn_domains"
o.description = translate("Name of the nftset/ipset for VPN policy routing (e.g. 4#inet#fw4#vpn_domains or vpn_domains).")

o = s:option(Value, "listen", translate("Listen"))
o.default = "0.0.0.0:18080"
o:depends("mode", "edge")

o = s:option(Value, "proxy_public_url", translate("Public Edge URL"))
o.placeholder = "http://192.168.8.1:18080"
o.description = translate("Required for ad strip: master variants rewrite to nested /https://… on the router. If empty, Edge uses the request Host (LAN IP). Set explicitly if you open Edge via 127.0.0.1.")
o:depends("mode", "edge")

o = s:option(Flag, "mitm", translate("MITM for HLS hosts (transparent/explicit only)"))
o.default = "0"
o:depends("mode", "edge")

s = m:section(NamedSection, "proxy", "proxy", translate("Hostlists"))
s.anonymous = true

o = s:option(MultiValue, "hostlist_services", translate("Service lists"))
o:value("twitch", "Twitch")
o:value("kick", "Kick")
o:value("trovo", "Trovo")
o:value("youtube", "YouTube")
o.widget = "select"
o.size = 4
o.description = translate("Which shipped/remote hostlists to compose. Enabling a plugin also adds its list.")

o = s:option(DynamicList, "custom_domain", translate("Custom domains"))
o.placeholder = "cdn.example.net"
o.description = translate("One domain or IPv4 per line. Merged into the effective hostlist.")

o = s:option(Flag, "hostlist_remote", translate("Update lists from GitHub"))
o.default = "0"
o.description = translate("Fetch hostlists from remote base every N hours (optional).")

o = s:option(Value, "hostlist_remote_base", translate("Remote base URL"))
o.default = "https://raw.githubusercontent.com/denis-ershov/OpenStream-Engine/main/package/openwrt/files/hostlists"

o = s:option(Value, "hostlist_remote_interval_hours", translate("Remote interval (hours)"))
o.datatype = "uinteger"
o.default = "12"

o = s:option(Value, "hostlist_file", translate("Effective hostlist file"))
o.default = "/var/run/openstream/hostlist-effective.txt"
o.description = translate("Written by openstream-compose-hostlist. Used for nft divert when mode=transparent.")

o = s:option(Value, "hostlist_refresh_secs", translate("Hostlist DNS refresh (sec)"))
o.datatype = "uinteger"
o.default = "300"

return m
