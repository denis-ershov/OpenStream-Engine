#!/usr/bin/env lua

local http = require "luci.http"

module("luci.controller.openstream", package.seeall)

function index()
	-- Menu primarily from /usr/share/luci/menu.d/luci-app-openstream.json (LuCI 24).
	-- Keep Lua entries as fallback for older luci-mod-admin without menu.d merge.
	if not nixio.fs.access("/etc/config/openstream") then
		return
	end

	local page = entry(
		{ "admin", "services", "openstream" },
		firstchild(),
		_("OpenStream Engine"),
		60
	)
	page.dependent = false
	page.acl_depends = { "luci-app-openstream" }

	entry(
		{ "admin", "services", "openstream", "status" },
		template("openstream/status"),
		_("Status"),
		1
	)
	entry(
		{ "admin", "services", "openstream", "twitch" },
		cbi("openstream/twitch"),
		_("Twitch"),
		2
	)
	entry(
		{ "admin", "services", "openstream", "services" },
		cbi("openstream/services"),
		_("Services"),
		3
	)
	entry(
		{ "admin", "services", "openstream", "logs" },
		template("openstream/logs"),
		_("Logs"),
		4
	)
	entry(
		{ "admin", "services", "openstream", "diagnostics" },
		template("openstream/diagnostics"),
		_("Diagnostics"),
		5
	)
	entry(
		{ "admin", "services", "openstream", "api_status" },
		call("action_api_status")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_events" },
		call("action_api_events")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_metrics" },
		call("action_api_metrics")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_reload" },
		call("action_api_reload")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_dns_test" },
		call("action_api_dns_test")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_routes" },
		call("action_api_routes")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_coexistence" },
		call("action_api_coexistence")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_toggle_ignore_warn" },
		call("action_api_toggle_ignore_warn")
	).leaf = true
end

function action_api_toggle_ignore_warn()
	local uci = require "luci.model.uci".cursor()
	local cur = uci:get("openstream", "main", "ignore_coexistence_warnings")
	local new_val = (cur == "1") and "0" or "1"
	uci:set("openstream", "main", "ignore_coexistence_warnings", new_val)
	uci:commit("openstream")
	http.prepare_content("application/json")
	http.write(string.format('{"ok":true,"ignored":%s}', (new_val == "1") and "true" or "false"))
end

function action_api_coexistence()
	local json = require "luci.jsonc"
	local nixio = require "nixio"
	local uci = require "luci.model.uci".cursor()

	local ignore_warn = (uci:get("openstream", "main", "ignore_coexistence_warnings") == "1")

	local function file_exists(p)
		return nixio.fs.access(p, "f") or nixio.fs.access(p, "x")
	end

	-- Возвращает короткое имя файла если домен найден, иначе nil
	local function twitch_file(p)
		if not file_exists(p) then return nil end
		local content = nixio.fs.readfile(p) or ""
		if content:find("twitch%.tv") then
			return p:match("([^/]+)$")
		end
		return nil
	end

	-- Проверяет список путей, возвращает { found=bool, files={"file1","file2",...} }
	local function check_files(...)
		local found = false
		local files = {}
		for _, p in ipairs({...}) do
			local name = twitch_file(p)
			if name then
				found = true
				table.insert(files, name)
			end
		end
		return { found = found, files = files }
	end

	local forkop_excl = check_files(
		"/etc/forkop/exclude.txt",
		"/etc/forkop/exclude_hosts.txt",
		"/etc/forkop/exclude_domains.txt",
		"/etc/forkop/direct_domains.txt",
		"/etc/forkop/whitelist.txt",
		"/etc/forkop/bypass.txt"
	)

	local podkop_excl = check_files(
		"/etc/podkop/exclude.txt",
		"/etc/podkop/exclude_domains.txt",
		"/etc/podkop/direct_hosts.txt",
		"/etc/podkop/custom_direct_domains.txt",
		"/etc/podkop/whitelist.txt"
	)

	local zapret_excl = check_files(
		"/opt/zapret/ipset/zapret-hosts-user-exclude.txt",
		"/etc/zapret/exclude.txt"
	)

	local forkop_found = check_files(
		"/etc/forkop/domains.txt",
		"/etc/forkop/hosts.txt",
		"/etc/forkop/user_domains.txt",
		"/etc/forkop/custom_domains.txt"
	)

	local podkop_found = check_files(
		"/etc/podkop/domains.txt",
		"/etc/podkop/hosts.txt",
		"/etc/podkop/user_domains.txt",
		"/etc/podkop/custom_domains.txt"
	)

	local zapret_found = check_files(
		"/opt/zapret/ipset/zapret-hosts-user.txt",
		"/etc/zapret/hosts.txt",
		"/etc/zapret/zapret-hosts-user.txt"
	)

	local neighbors = {
		podkop = {
			name = "Podkop",
			detected = file_exists("/etc/init.d/podkop") or file_exists("/etc/config/podkop"),
			has_twitch = podkop_found.found,
			twitch_files = podkop_found.files,
			is_excluded = podkop_excl.found,
			exclude_files = podkop_excl.files
		},
		forkop = {
			name = "Forkop",
			detected = file_exists("/etc/init.d/forkop") or file_exists("/etc/config/forkop"),
			has_twitch = forkop_found.found,
			twitch_files = forkop_found.files,
			is_excluded = forkop_excl.found,
			exclude_files = forkop_excl.files
		},
		netshift = {
			name = "NetShift",
			detected = file_exists("/etc/init.d/netshift") or file_exists("/etc/config/netshift"),
			has_twitch = (twitch_file("/etc/netshift/domains.txt") or twitch_file("/etc/netshift/hosts.txt")) ~= nil,
			twitch_files = {},
			is_excluded = false,
			exclude_files = {}
		},
		zapret = {
			name = "Zapret / Zapret2",
			detected = file_exists("/etc/init.d/zapret") or file_exists("/opt/zapret/init.d/sysv/zapret") or file_exists("/etc/config/zapret"),
			has_twitch = zapret_found.found,
			twitch_files = zapret_found.files,
			is_excluded = zapret_excl.found,
			exclude_files = zapret_excl.files
		},
		byedpi = {
			name = "ByeDPI",
			detected = file_exists("/etc/init.d/byedpi") or file_exists("/usr/bin/ciadpi"),
			has_twitch = false,
			twitch_files = {},
			is_excluded = false,
			exclude_files = {}
		},
		openclash = {
			name = "OpenClash / Mihomo",
			detected = file_exists("/etc/init.d/openclash") or file_exists("/etc/config/openclash"),
			has_twitch = false,
			twitch_files = {},
			is_excluded = false,
			exclude_files = {}
		},
		passwall = {
			name = "PassWall / HomeProxy",
			detected = file_exists("/etc/init.d/passwall") or file_exists("/etc/config/passwall"),
			has_twitch = false,
			twitch_files = {},
			is_excluded = false,
			exclude_files = {}
		}
	}

	http.prepare_content("application/json")
	http.write(json.stringify({
		neighbors = neighbors,
		ignore_warnings = ignore_warn
	}))
end

local function proxy_get(path)
	local util = require "luci.util"
	return util.exec("wget -qO- http://127.0.0.1:18080" .. path .. " 2>/dev/null")
end

function action_api_status()
	local uci = require "luci.model.uci".cursor()
	local json = require "luci.jsonc"
	local body = proxy_get("/api/status")
	local data = {}
	if body and #body > 0 then
		data = json.parse(body) or {}
	end
	data.twitch_enabled = (uci:get("openstream", "twitch", "enabled") ~= "0")
	data.twitch_preset = uci:get("openstream", "twitch", "preset") or "ru_smartdns_noads_quality"
	data.route_token = uci:get("openstream", "twitch", "route_token") or "dns_yandex"
	data.route_master = uci:get("openstream", "twitch", "route_master") or "smartdns_comss"
	data.route_media = uci:get("openstream", "twitch", "route_media") or "direct"
	data.route_segments = uci:get("openstream", "twitch", "route_segments") or "direct"
	data.route_ads = uci:get("openstream", "twitch", "route_ads") or "block"
	http.prepare_content("application/json")
	http.write(json.stringify(data))
end

function action_api_events()
	local body = proxy_get("/api/events")
	http.prepare_content("application/json")
	if body and #body > 0 then
		http.write(body)
	else
		http.write("[]")
	end
end

function action_api_metrics()
	local body = proxy_get("/metrics")
	http.prepare_content("text/plain; charset=utf-8")
	if body and #body > 0 then
		http.write(body)
	else
		http.write("# daemon unreachable\n")
	end
end

function action_api_reload()
	local util = require "luci.util"
	util.exec("/usr/libexec/openstream-uci2yaml >/dev/null 2>&1")
	util.exec("/etc/init.d/streamproxyd reload >/dev/null 2>&1")
	http.prepare_content("application/json")
	http.write('{"ok":true}')
end

function action_api_routes()
	local nixio = require "nixio"
	local content = nixio.fs.readfile("/tmp/dnsmasq.d/openstream.conf") or
	                nixio.fs.readfile("/etc/dnsmasq.d/openstream.conf") or ""
	http.prepare_content("text/plain; charset=utf-8")
	http.write(content)
end

function action_api_dns_test()
	local nixio = require "nixio"
	local util = require "luci.util"
	local json = require "luci.jsonc"
	local domains = {
		"gql.twitch.tv",
		"usher.ttvnw.net",
		"edge.ads.twitch.tv",
		"countess.twitch.tv",
		"video-weaver.fra02.hls.ttvnw.net"
	}

	local results = {}
	for _, domain in ipairs(domains) do
		local ips = {}
		local seen = {}

		-- 1. Системный getaddrinfo (нативно считывает /etc/hosts и системный DNS)
		local addrs = nixio.getaddrinfo(domain, "inet")
		if addrs then
			for _, a in ipairs(addrs) do
				if a.address and not seen[a.address] then
					seen[a.address] = true
					table.insert(ips, a.address)
				end
			end
		end

		-- 2. Fallback на nslookup если getaddrinfo пуст
		if #ips == 0 then
			local out = util.exec(string.format("nslookup %s 2>/dev/null", domain))
			if out and #out > 0 then
				for ip in out:gmatch("Address[%s%d]*:%s*([%d%.]+)") do
					if ip ~= "127.0.0.1" and ip ~= "127.0.0.42" and not seen[ip] then
						seen[ip] = true
						table.insert(ips, ip)
					end
				end
			end
		end

		table.insert(results, {
			domain = domain,
			resolved = (#ips > 0),
			ips = ips
		})
	end
	http.prepare_content("application/json")
	http.write(json.stringify(results))
end
