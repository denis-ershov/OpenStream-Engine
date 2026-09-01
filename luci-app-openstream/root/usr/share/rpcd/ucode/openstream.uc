'use strict';

import { readfile, writefile, access, dir, stat } from 'fs';
import { cursor } from 'uci';

return {
	openstream: {
		status: {
			call: function(req) {
				let running = false;
				let zapret2_running = false;
				let zapret2_installed = access('/usr/bin/nfqws2') || access('/usr/bin/nfqws');
				let singbox_installed = access('/usr/bin/sing-box');

				// Проверка работы основного демона streamproxyd
				let pid_str = readfile('/var/run/streamproxyd.pid');
				if (pid_str && access('/proc/' + trim(pid_str))) {
					running = true;
				}

				// Проверка работы zapret2 (nfqws / nfqws2)
				let nfqws_pids = readfile('/var/run/nfqws.pid') || readfile('/var/run/nfqws2.pid');
				if (nfqws_pids && access('/proc/' + trim(nfqws_pids))) {
					zapret2_running = true;
				}

				return {
					running: running,
					zapret2_installed: zapret2_installed,
					zapret2_running: zapret2_running,
					singbox_installed: singbox_installed,
					version: "2.1.0"
				};
			}
		},

		get_routing: {
			call: function(req) {
				let clients = [];
				let leases_raw = readfile('/tmp/dhcp.leases') || '';
				for (let line in split(leases_raw, '\n')) {
					let parts = split(trim(line), ' ');
					if (length(parts) >= 4) {
						push(clients, {
							mac: uc(parts[1]),
							ip: parts[2],
							hostname: (parts[3] != '*') ? parts[3] : 'Unknown Device'
						});
					}
				}

				let rules = [];
				let rules_dir = '/etc/openstream/rules';
				let entries = dir(rules_dir);
				if (entries) {
					for (let filename in entries) {
						if (match(filename, /\.osrule\.ya?ml$/)) {
							let content = readfile(rules_dir + '/' + filename);
							if (content) {
								let id_m = match(content, /id:\s*["']?([a-zA-Z0-9_\-\.]+)/);
								let name_m = match(content, /name:\s*["']?([^"\r\n]+)/);
								let desc_m = match(content, /description:\s*["']?([^"\r\n]+)/);
								let enabled = !match(content, /enabled:\s*false/);

								let action = 'direct';
								if (match(content, /zapret2/)) {
									action = 'zapret2';
								} else if (match(content, /streamproxy/) || match(content, /stream_proxy/)) {
									action = 'streamproxy';
								} else if (match(content, /proxy:/) || match(content, /action:\s*"?proxy/)) {
									action = 'vpn';
								} else if (match(content, /action:\s*"?block/)) {
									action = 'block';
								}

								let preset_m = match(content, /preset:\s*["']?([a-zA-Z0-9_]+)/);

								push(rules, {
									file: filename,
									id: id_m ? id_m[1] : filename,
									name: name_m ? name_m[1] : filename,
									enabled: enabled,
									description: desc_m ? desc_m[1] : '',
									action: action,
									preset: preset_m ? preset_m[1] : 'youtube_4k',
									raw_yaml: content
								});
							}
						}
					}
				}

				// Fallback если каталог пуст
				if (!length(rules)) {
					push(rules, {
						file: 'youtube.osrule.yaml',
						id: 'org.openstream.rules.youtube',
						name: 'YouTube 4K & Googlevideo',
						enabled: true,
						description: 'Десинхронизация DPI через Zapret2 (nfqws2) без тормозов 4K',
						action: 'zapret2',
						preset: 'youtube_4k',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.youtube\"\nname: \"YouTube 4K\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"*.googlevideo.com\", \"*.ytimg.com\"]\n    action:\n      zapret2:\n        preset: \"youtube_4k\"\n"
					});
					push(rules, {
						file: 'discord.osrule.yaml',
						id: 'org.openstream.rules.discord',
						name: 'Discord RTC & Voice',
						enabled: true,
						description: 'Обход блокировок голосовых каналов Discord через UDP-пресет Zapret2',
						action: 'zapret2',
						preset: 'discord_voice',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.discord\"\nname: \"Discord RTC & Voice\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"*.discord.gg\", \"*.discord.media\"]\n    action:\n      zapret2:\n        preset: \"discord_voice\"\n"
					});
					push(rules, {
						file: 'twitch.osrule.yaml',
						id: 'org.openstream.rules.twitch',
						name: 'Twitch AdBlock StreamProxy',
						enabled: true,
						description: 'Локальная модификация плейлистов HLS без рекламы в 1080p60',
						action: 'streamproxy',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.twitch\"\nname: \"Twitch Optimizer\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"gql.twitch.tv\"]\n    action:\n      stream_proxy:\n        mode: \"adfree\"\n"
					});
					push(rules, {
						file: 'bittorrent.osrule.yaml',
						id: 'org.openstream.rules.p2p',
						name: 'BitTorrent & P2P Bypass',
						enabled: true,
						description: 'Исключение P2P-трафика из прокси и туннелей напрямую в WAN (#72)',
						action: 'direct',
						bypass_p2p: true,
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.p2p\"\nname: \"BitTorrent Bypass\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"*.tracker.torrent.to\"]\n    bypass:\n      bypass_p2p: true\n    action: \"direct\"\n"
					});
				}

				return {
					zapret2_installed: access('/usr/bin/nfqws2') || access('/usr/bin/nfqws'),
					singbox_installed: access('/usr/bin/sing-box'),
					clients: clients,
					rules: rules
				};
			}
		},

		save_routing: {
			args: { rules: [] },
			call: function(req) {
				let rules = req.args.rules;
				if (!rules || !length(rules)) {
					return { success: false, error: 'Отсутствуют правила для сохранения' };
				}

				let rules_dir = '/etc/openstream/rules';
				// Атомарный бэкап перед валидацией (Rollback safety - Правило #4)
				system('rm -rf /tmp/openstream_rules_bak && cp -r ' + rules_dir + ' /tmp/openstream_rules_bak 2>/dev/null');

				for (let r in rules) {
					if (r.file && r.raw_yaml) {
						// Защита от Path Traversal: извлекаем только базовое имя и проверяем суффикс
						let safe_name = replace(r.file, /^.*[\/\\]/, '');
						if (!match(safe_name, /^[a-zA-Z0-9_\-]+\.osrule\.ya?ml$/)) {
							continue;
						}
						writefile(rules_dir + '/' + safe_name, r.raw_yaml);
					}
				}

				// Атомарная валидация компилятором ядра
				let compile_res = system('/usr/bin/streamproxyd --compile-rules --rules-dir ' + rules_dir + ' 2>&1');
				if (compile_res != 0) {
					// Автоматический откат при ошибке синтаксиса
					system('cp -r /tmp/openstream_rules_bak/* ' + rules_dir + '/ 2>/dev/null');
					return {
						success: false,
						error: 'Ошибка компиляции правил. Выполнен автоматический откат к рабочей версии.'
					};
				}

				system('/etc/init.d/streamproxyd reload 2>/dev/null || /etc/init.d/openstream reload 2>/dev/null');
				return { success: true, message: 'Маршруты успешно скомпилированы и применены в ucode.' };
			}
		},

		test_route: {
			args: { domain: "example.com" },
			call: function(req) {
				let domain = req.args.domain;
				if (!domain) return { success: false, error: 'Домен не указан' };

				// Защита от инъекций спецсимволов: строгая фильтрация hostname
				domain = trim(domain);
				if (!match(domain, /^[a-zA-Z0-9\.\-]+$/) || length(domain) > 253) {
					return { success: false, error: 'Недопустимый формат доменного имени' };
				}

				let dns_cfg = readfile('/tmp/dnsmasq.d/openstream-rules.conf') || '';
				let res = {
					domain: domain,
					route: 'direct',
					engine: 'Direct / WAN',
					details: 'Трафик идет напрямую через сетевой шлюз провайдера'
				};

				let needle = '/' + domain + '/';
				if (index(dns_cfg, needle) >= 0) {
					for (let line in split(dns_cfg, '\n')) {
						if (index(line, needle) >= 0) {
							if (index(line, 'zapret2_targets') >= 0) {
								res.route = 'zapret2';
								res.engine = 'Zapret2 (nfqws2 NFQUEUE 1088)';
								res.details = 'Обход блокировок ТСПУ десинхронизацией пакетов';
							} else if (index(line, 'streamproxy_targets') >= 0) {
								res.route = 'streamproxy';
								res.engine = 'OpenStream StreamProxy (:8888)';
								res.details = 'Локальное удаление рекламы и стриминг HLS';
							} else if (index(line, 'vpn_') >= 0) {
								res.route = 'vpn';
								res.engine = 'VPN Gateway';
								res.details = 'Маршрутизируется в зашифрованный туннель';
							} else if (index(line, '0.0.0.0') >= 0) {
								res.route = 'block';
								res.engine = 'DNS Sinkhole (Blocked)';
								res.details = 'Заблокировано на уровне DNS';
							}
							break;
						}
					}
				}

				return { success: true, result: res };
			}
		}
	}
};
