#!/usr/bin/env lua

local http = require "luci.http"

module("luci.controller.openstream", package.seeall)

function index()
	-- Menu is defined declaratively via /usr/share/luci/menu.d/luci-app-openstream.json (LuCI 21.02+ / 24.10).
	-- Controller only handles API endpoints.
	entry(
		{ "admin", "services", "openstream", "api_status" },
		call("action_api_status")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_routing_get" },
		call("action_api_routing_get")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_routing_save" },
		call("action_api_routing_save")
	).leaf = true
	entry(
		{ "admin", "services", "openstream", "api_routing_test" },
		call("action_api_routing_test")
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

function action_api_routing_get()
	local nixio = require "nixio"
	local json = require "luci.jsonc"
	local util = require "luci.util"

	local zapret2_installed = nixio.fs.access("/usr/bin/nfqws2") or nixio.fs.access("/usr/bin/nfqws")
	local singbox_installed = nixio.fs.access("/usr/bin/sing-box")

	-- Сбор клиентов локальной сети из /tmp/dhcp.leases
	local clients = {}
	local leases_raw = nixio.fs.readfile("/tmp/dhcp.leases") or ""
	for line in leases_raw:gmatch("[^\r\n]+") do
		local ts, mac, ip, name = line:match("(%d+)%s+([%x:]+)%s+([%d%.]+)%s+([^%s]+)")
		if mac and ip then
			table.insert(clients, {
				mac = mac:upper(),
				ip = ip,
				hostname = (name ~= "*" and name) or "Unknown Device"
			})
		end
	end

	-- Сбор правил из /etc/openstream/rules/
	local rules = {}
	local rules_dir = "/etc/openstream/rules"
	local entries = nixio.fs.dir(rules_dir)
	if entries then
		for filename in entries do
			if filename:match("%.osrule%.ya?ml$") then
				local content = nixio.fs.readfile(rules_dir .. "/" .. filename)
				if content then
					local id = content:match("id:%s*[\"']?([%w%.%-_]+)") or filename
					local name = content:match("name:%s*[\"']?([^\"\r\n]+)") or id
					local enabled = not content:match("enabled:%s*false")
					local desc = content:match("description:%s*[\"']?([^\"\r\n]+)") or ""
					
					-- Определение активного действия
					local action = "direct"
					if content:match("zapret2") then
						action = "zapret2"
					elseif content:match("stream_proxy") or content:match("streamproxy") then
						action = "streamproxy"
					elseif content:match("proxy:") or content:match("action:%s*\"?proxy") then
						action = "vpn"
					elseif content:match("action:%s*\"?block") then
						action = "block"
					end

					table.insert(rules, {
						file = filename,
						id = id,
						name = name,
						enabled = enabled,
						description = desc,
						action = action,
						raw_yaml = content
					})
				end
			end
		end
	end

	-- Если каталог пуст, отдаем базовые встроенные шаблоны
	if #rules == 0 then
		table.insert(rules, {
			file = "youtube.osrule.yaml",
			id = "org.openstream.rules.youtube",
			name = "YouTube 4K & Googlevideo",
			enabled = true,
			description = "Десинхронизация DPI через Zapret2 (nfqws2) без тормозов 4K",
			action = "zapret2",
			preset = "youtube_4k"
		})
		table.insert(rules, {
			file = "discord.osrule.yaml",
			id = "org.openstream.rules.discord",
			name = "Discord RTC & Voice",
			enabled = true,
			description = "Обход блокировок голосовых каналов Discord через UDP-пресет Zapret2",
			action = "zapret2",
			preset = "discord_voice"
		})
		table.insert(rules, {
			file = "twitch.osrule.yaml",
			id = "org.openstream.rules.twitch",
			name = "Twitch AdBlock StreamProxy",
			enabled = true,
			description = "Локальная модификация плейлистов HLS без рекламы в 1080p60",
			action = "streamproxy"
		})
		table.insert(rules, {
			file = "bittorrent.osrule.yaml",
			id = "org.openstream.rules.p2p",
			name = "BitTorrent & P2P Bypass",
			enabled = true,
			description = "Исключение P2P-трафика из прокси и туннелей напрямую в WAN (решение #72)",
			action = "direct",
			bypass_p2p = true
		})
	end

	local data = {
		zapret2_installed = zapret2_installed,
		singbox_installed = singbox_installed,
		clients = clients,
		rules = rules,
		shadow_warnings = {}
	}

	http.prepare_content("application/json")
	http.write(json.stringify(data))
end

function action_api_routing_save()
	local json = require "luci.jsonc"
	local util = require "luci.util"
	local nixio = require "nixio"

	local body = http.content()
	local payload = json.parse(body or "{}")

	if not payload or not payload.rules then
		http.status(400, "Bad Request")
		http.prepare_content("application/json")
		http.write(json.stringify({ success = false, error = "Отсутствуют правила для сохранения" }))
		return
	end

	local rules_dir = "/etc/openstream/rules"
	nixio.fs.mkdir(rules_dir)

	-- Резервная копия перед валидацией (Rollback safety - правило #4)
	util.exec("rm -rf /tmp/openstream_rules_backup && cp -r " .. rules_dir .. " /tmp/openstream_rules_backup 2>/dev/null")

	-- Запись обновленных файлов правил
	for _, r in ipairs(payload.rules) do
		if r.file and r.raw_yaml then
			local filepath = rules_dir .. "/" .. r.file
			nixio.fs.writefile(filepath, r.raw_yaml)
		end
	end

	-- Атомарная валидация компилятором ядра
	local validate_res = util.exec("/usr/bin/streamproxyd --compile-rules --rules-dir " .. rules_dir .. " 2>&1")
	local ok = (nixio.fs.access("/tmp/dnsmasq.d/openstream-rules.conf") and true) or false

	if not ok and nixio.fs.access("/tmp/openstream_rules_backup") then
		-- Откат при ошибке
		util.exec("cp -r /tmp/openstream_rules_backup/* " .. rules_dir .. "/ 2>/dev/null")
		http.status(422, "Validation Failed")
		http.prepare_content("application/json")
		http.write(json.stringify({
			success = false,
			error = "Ошибка компиляции правил. Выполнен автоматический откат к рабочей версии.",
			details = validate_res
		}))
		return
	end

	-- Применение изменений без разрыва DNS (Правило #1, #4)
	util.exec("/etc/init.d/openstream reload 2>/dev/null")

	http.prepare_content("application/json")
	http.write(json.stringify({ success = true, message = "Маршруты успешно обновлены и проверены ядром." }))
end

function action_api_routing_test()
	local json = require "luci.jsonc"
	local util = require "luci.util"
	local http = require "luci.http"

	local domain = http.formvalue("domain") or ""
	domain = domain:gsub("[%s;%|%&]", "")

	if #domain == 0 then
		http.status(400, "Bad Request")
		http.prepare_content("application/json")
		http.write(json.stringify({ success = false, error = "Домен не указан" }))
		return
	end

	local nft_match = util.exec(string.format("nft list sets | grep -B 1 -A 5 '%s' 2>/dev/null", domain))
	local dns_check = util.exec(string.format("grep -i '%s' /tmp/dnsmasq.d/openstream-rules.conf 2>/dev/null", domain))

	local result = {
		domain = domain,
		route = "direct",
		engine = "Direct / WAN",
		details = "Трафик идет напрямую через сетевой шлюз провайдера"
	}

	if dns_check:match("zapret2_targets") then
		result.route = "zapret2"
		result.engine = "Zapret2 (nfqws2 NFQUEUE 1088)"
		result.details = "Обход блокировок ТСПУ десинхронизацией пакетов"
	elseif dns_check:match("streamproxy_targets") then
		result.route = "streamproxy"
		result.engine = "OpenStream StreamProxy (:8888)"
		result.details = "Локальное удаление рекламы и стриминг HLS"
	elseif dns_check:match("vpn_") then
		local vpn_tag = dns_check:match("vpn_([%w_]+)")
		result.route = "vpn"
		result.engine = "VPN Gateway (" .. (vpn_tag or "default") .. ")"
		result.details = "Маршрутизируется в зашифрованный туннель"
	elseif dns_check:match("0.0.0.0") then
		result.route = "block"
		result.engine = "DNS Sinkhole (Blocked)"
		result.details = "Заблокировано на уровне DNS"
	end

	http.prepare_content("application/json")
	http.write(json.stringify({ success = true, result = result }))
end

