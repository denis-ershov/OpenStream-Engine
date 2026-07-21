#!/usr/bin/env lua

local http = require "luci.http"

module("luci.controller.openstream", package.seeall)

function index()
	if not nixio.fs.access("/etc/config/openstream") then
		return
	end

	local page = entry(
		{ "admin", "services", "openstream" },
		firstchild(),
		_("OpenStream Engine"),
		60
	)
	page.dependent = true
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
		{ "admin", "services", "openstream", "events" },
		template("openstream/events"),
		_("Events"),
		4
	)
	entry(
		{ "admin", "services", "openstream", "metrics" },
		template("openstream/metrics"),
		_("Metrics"),
		5
	)
	entry(
		{ "admin", "services", "openstream", "logs" },
		template("openstream/logs"),
		_("Logs"),
		6
	)
	entry(
		{ "admin", "services", "openstream", "coexistence" },
		template("openstream/coexistence"),
		_("Environment"),
		7
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
end

local function proxy_get(path)
	local util = require "luci.util"
	return util.exec("wget -qO- http://127.0.0.1:18080" .. path .. " 2>/dev/null")
end

function action_api_status()
	local body = proxy_get("/api/status")
	http.prepare_content("application/json")
	if body and #body > 0 then
		http.write(body)
	else
		http.write('{"error":"daemon unreachable"}')
	end
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
	local body = util.exec(
		"wget -qO- --post-data='' http://127.0.0.1:18080/api/reload 2>/dev/null"
	)
	if not body or #body == 0 then
		util.exec("/etc/init.d/streamproxyd reload >/dev/null 2>&1")
		body = '{"ok":true,"via":"init"}'
	end
	http.prepare_content("application/json")
	http.write(body)
end
