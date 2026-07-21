local m, s, o

local function apply_daemon()
	luci.sys.call("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	luci.sys.call(
		"wget -qO- --post-data='' http://127.0.0.1:18080/api/reload >/dev/null 2>&1 " ..
		"|| /etc/init.d/streamproxyd reload >/dev/null 2>&1"
	)
end

m = Map("openstream", translate("Twitch plugin"),
	translate("Segment Stripping for Twitch HLS. Client waits during ad breaks."))

m.on_after_commit = function()
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

s = m:section(NamedSection, "proxy", "proxy", translate("Proxy"))
s.anonymous = true

o = s:option(ListValue, "mode", translate("Mode"))
o:value("explicit", translate("Explicit (recommended)"))
o:value("redirect_whitelist", translate("Redirect whitelist (opt-in)"))
o:value("off", translate("Off"))
o.default = "explicit"

o = s:option(Value, "listen", translate("Listen"))
o.default = "0.0.0.0:18080"

o = s:option(Value, "proxy_public_url", translate("Public proxy URL"))
o.placeholder = "http://192.168.1.1:18080"
o.description = translate("Used for master playlist rewrite.")

o = s:option(Flag, "mitm", translate("MITM for HLS/DASH hosts"))
o.default = "1"

return m
