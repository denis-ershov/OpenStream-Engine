local m, s, o

local function apply_daemon()
	luci.sys.call("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	luci.sys.call("/etc/init.d/streamproxyd reload >/dev/null 2>&1")
end

m = Map("openstream", translate("Streaming Services Hub"),
	translate("Manage streaming platform modules and global settings."))

m.on_after_commit = function()
	apply_daemon()
end

s = m:section(NamedSection, "main", "main", translate("Global Settings"))
s.anonymous = true

o = s:option(Flag, "enabled", translate("Master Switch (OpenStream Engine)"))
o.default = "1"
o.rmempty = false

o = s:option(Flag, "expert_mode", translate("Expert / Developer Mode"))
o.default = "0"
o.description = translate("Enables raw metric views and low-level diagnostic panels in navigation.")

s = m:section(NamedSection, "youtube", "module", translate("YouTube (Experimental)"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable YouTube Module"))
o.default = "0"
o.description = translate("Policy-based routing for YouTube CDN and googlevideo streams.")

s = m:section(NamedSection, "kick", "module", translate("Kick.com"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable Kick Module"))
o.default = "0"

s = m:section(NamedSection, "trovo", "module", translate("Trovo.live"))
s.anonymous = true
o = s:option(Flag, "enabled", translate("Enable Trovo Module"))
o.default = "0"

return m
