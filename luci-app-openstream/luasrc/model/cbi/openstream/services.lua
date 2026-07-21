local m, s, o

local function apply_daemon()
	luci.sys.call("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	luci.sys.call(
		"wget -qO- --post-data='' http://127.0.0.1:18080/api/reload >/dev/null 2>&1 " ..
		"|| /etc/init.d/streamproxyd reload >/dev/null 2>&1"
	)
end

m = Map("openstream", translate("Services & observability"),
	translate("Enable additional HLS/DASH plugins and metrics endpoints."))

m.on_after_commit = function()
	apply_daemon()
end

s = m:section(NamedSection, "kick", "kick", translate("Kick"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable"))
o.default = "0"
o = s:option(Flag, "debug", translate("Debug"))
o.default = "0"

s = m:section(NamedSection, "trovo", "trovo", translate("Trovo"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable"))
o.default = "0"
o = s:option(Flag, "debug", translate("Debug"))
o.default = "0"

s = m:section(NamedSection, "youtube", "youtube", translate("YouTube Live"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable"))
o.default = "0"
o = s:option(Flag, "debug", translate("Debug"))
o.default = "0"

s = m:section(NamedSection, "dash", "dash", translate("DASH"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable"))
o.default = "1"
o = s:option(Flag, "debug", translate("Debug"))
o.default = "0"

s = m:section(NamedSection, "observability", "observability", translate("Observability"))
s.anonymous = true
o = s:option(Flag, "metrics", translate("OpenMetrics /metrics"))
o.default = "1"
o = s:option(Flag, "events", translate("Events API /api/events"))
o.default = "1"
o = s:option(Value, "event_capacity", translate("Event ring capacity"))
o.datatype = "uinteger"
o.default = "128"

return m
