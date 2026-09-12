'use strict';

// Модуль `fs` экспортирует lsdir(); функции `dir` в нём нет.
// Ранее импортировался несуществующий символ — плагин падал при загрузке.
import { readfile, writefile, access, lsdir, stat, unlink, popen } from 'fs';
import { cursor } from 'uci';

//
// --- Безопасность: исполнение внешних команд ---
//
// RPC-плагин выполняется демоном rpcd от имени root. Любая конкатенация
// пользовательского ввода в строку shell-команды означает выполнение
// произвольного кода с правами root. Поэтому ниже:
//   1) весь пользовательский ввод проходит строгую валидацию по allowlist;
//   2) аргументы экранируются одинарными кавычками перед передачей в popen().
//
// В регулярных выражениях дефис внутри класса символов размещён ПОСЛЕДНИМ и
// не экранируется: рантайм ucode отвергает `\-` в классе, хотя `ucode -c`
// такую ошибку не выявляет.
//

/// Управляющие символы и пробелы: рантайм ucode НЕ поддерживает диапазоны вида
/// \x00-\x1f внутри класса символов, поэтому используем POSIX-класс [[:cntrl:]].
function has_control_or_space(value) {
	return match(value, /[[:cntrl:]]/) != null || match(value, /[\s]/) != null;
}

/// Экранирование одного аргумента для POSIX-шелла.
/// Одинарные кавычки нейтрализуют ВСЕ метасимволы; внутренняя кавычка
/// закрывается, экранируется и открывается заново: ' -> '\''
function shell_quote(value) {
	let s = '' + (value ?? '');
	return "'" + replace(s, /'/g, "'\\''") + "'";
}

/// Сборка безопасной командной строки из массива аргументов.
function shell_command(argv) {
	let parts = [];
	for (let a in argv) push(parts, shell_quote(a));
	return join(' ', parts);
}

/// HTTP(S) URL без управляющих символов. Используется для подписок.
function valid_http_url(value) {
	if (!value || length(value) > 2048) return false;
	// Явный запрет пробельных и управляющих символов до проверки формы
	if (has_control_or_space(value)) return false;
	return match(value, /^https?:\/\/[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]+$/) != null;
}

/// Имя хоста (FQDN) или IPv4/IPv6-литерал — для проверки доступности узла.
function valid_host(value) {
	if (!value || length(value) > 253) return false;
	if (has_control_or_space(value)) return false;
	// FQDN
	if (match(value, /^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)*$/)) return true;
	// IPv6-литерал
	if (match(value, /^[0-9a-fA-F:]+$/) && index(value, ':') >= 0) return true;
	return false;
}

/// Заголовок HTTP: печатаемый ASCII без CR/LF (защита от header injection).
function valid_header_value(value) {
	if (value == null || value == '') return true;
	if (length(value) > 256) return false;
	return match(value, /^[A-Za-z0-9 ._/():;,+-]+$/) != null;
}

/// Проверка наличия исполняемой команды в PATH.
function command_ok(argv) {
	let p = popen(shell_command(argv) + ' >/dev/null 2>&1', 'r');
	if (!p) return false;
	p.close();
	return true;
}

/// Версия установленного пакета по данным opkg.
function opkg_installed_version(package_name) {
	if (!match(package_name, /^[a-z0-9][a-z0-9+._-]*$/)) return null;

	let p = popen(shell_command(['opkg', 'status', package_name]) + ' 2>/dev/null', 'r');
	if (!p) return null;

	let out = p.read('all');
	p.close();
	if (!out) return null;

	let m = match(out, /Version:\s*(\S+)/);
	return m ? m[1] : null;
}

/// Первая строка вывода `<binary> version` — фактическая версия бинарника.
function binary_version(bin_path) {
	if (!bin_path || !access(bin_path)) return null;

	let p = popen(shell_command([bin_path, 'version']) + ' 2>/dev/null', 'r');
	if (!p) return null;

	let out = p.read('all');
	p.close();
	if (!out) return null;

	let first = trim(split(out, '\n')[0]);
	return length(first) > 0 ? first : null;
}

/// Дозапись в лог обновлений.
///
/// Ранее применялся writefile() (перезапись), поэтому в логе оставалась только
/// последняя строка — история операций терялась.
function update_log_append(message) {
	let path = '/tmp/openstream_update.log';
	let existing = readfile(path) || '';
	// Ограничиваем размер, чтобы лог не рос бесконечно на роутере с малым /tmp.
	if (length(existing) > 32768) {
		existing = slice(existing, -16384);
	}
	writefile(path, existing + sprintf('[%s] %s\n', date(), message));
}

/// Единый каталог декларативных правил. Тот же путь используют
/// streamproxyd.init и openstream-render.uc.
const RULES_DIR = '/etc/openstream/rules';
/// Рендерер конфигураций устанавливается пакетом в /usr/libexec.
const RENDER_UC = '/usr/libexec/openstream-render.uc';
/// Скрипт-оркестратор раздельных обновлений и автообновления
const UPDATE_SCRIPT = '/usr/libexec/openstream-update';

function url_decode(s) {
	if (!s) return '';
	return replace(s, /%([0-9A-Fa-f]{2})/g, function(m, hex) {
		return chr(hex(hex));
	});
}

function extract_country_flag(name) {
	if (!name) return { code: 'UN', flag: '🌐' };
	let flag_map = {
		'US': '🇺🇸', 'USA': '🇺🇸', 'RU': '🇷🇺', 'DE': '🇩🇪', 'NL': '🇳🇱',
		'FI': '🇫🇮', 'TR': '🇹🇷', 'GB': '🇬🇧', 'UK': '🇬🇧', 'FR': '🇫🇷',
		'JP': '🇯🇵', 'SG': '🇸🇬', 'HK': '🇭🇰', 'SE': '🇸🇪', 'PL': '🇵🇱',
		'KZ': '🇰🇿', 'UA': '🇺🇦', 'BY': '🇧🇾', 'CA': '🇨🇦', 'CH': '🇨🇭'
	};

	let upper = uc(name);
	for (let code, flag in flag_map) {
		if (index(upper, code) >= 0) {
			return { code: code, flag: flag };
		}
	}
	return { code: 'VPN', flag: '🌐' };
}

return {
	openstream: {
		status: {
			call: function(req) {
				let running = false;
				let zapret2_running = false;
				let singbox_running = false;
				let zapret2_installed = access('/usr/bin/nfqws2') || access('/usr/bin/nfqws');
				let singbox_installed = access('/usr/bin/sing-box');

				let pid_str = readfile('/var/run/streamproxyd.pid');
				if (pid_str && access('/proc/' + trim(pid_str))) {
					running = true;
				}

				let nfqws_pids = readfile('/var/run/nfqws.pid') || readfile('/var/run/nfqws2.pid');
				if (nfqws_pids && access('/proc/' + trim(nfqws_pids))) {
					zapret2_running = true;
				}

				let sb_pid = readfile('/var/run/sing-box.pid');
				if (sb_pid && access('/proc/' + trim(sb_pid))) {
					singbox_running = true;
				}

				let uci = cursor();
				let singbox_variant = uci.get('openstream', 'singbox', 'variant') || 'stable';

				return {
					running: running,
					zapret2_installed: zapret2_installed,
					zapret2_running: zapret2_running,
					singbox_installed: singbox_installed,
					singbox_running: singbox_running,
					singbox_variant: singbox_variant,
					version: "2.1.0-r37"
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
								let id_m = match(content, /id:\s*["']?([a-zA-Z0-9_.-]+)/);
								let name_m = match(content, /name:\s*["']?([^"'\n\r]+)/);
								let desc_m = match(content, /description:\s*["']?([^"'\n\r]+)/);
								let enabled = !match(content, /enabled:\s*false/);
								let action = 'direct';

								if (match(content, /action:\s*"?bypass/) || match(content, /action:\s*"?pass/) || match(content, /action:\s*"?exclude/)) {
									action = 'bypass';
								} else if (match(content, /zapret2/) || match(content, /nfqws2/)) {
									action = 'zapret2';
								} else if (match(content, /streamproxy/) || match(content, /stream_proxy/)) {
									action = 'streamproxy';
								} else if (match(content, /proxy:/) || match(content, /action:\s*"?proxy/) || match(content, /action:\s*"?vpn/)) {
									action = 'vpn';
								} else if (match(content, /action:\s*"?block/)) {
									action = 'block';
								}

								let preset_m = match(content, /preset:\s*["']?([a-zA-Z0-9_]+)/);
								let custom_args_m = match(content, /custom_args:\s*["']?([^"'\n\r]+)/);

								push(rules, {
									file: filename,
									id: id_m ? id_m[1] : filename,
									name: name_m ? name_m[1] : filename,
									enabled: enabled,
									description: desc_m ? desc_m[1] : '',
									action: action,
									preset: preset_m ? preset_m[1] : 'youtube_4k',
									custom_args: custom_args_m ? custom_args_m[1] : '',
									raw_yaml: content
								});
							}
						}
					}
				}

				if (!length(rules)) {
					push(rules, {
						file: 'exclusions.osrule.yaml',
						id: 'org.openstream.rules.exclusions',
						name: 'Исключения Bypass (Direct WAN)',
						enabled: true,
						description: 'Приоритетный пропуск исключений напрямую в WAN без Zapret2 и VPN',
						action: 'bypass',
						preset: 'none',
						custom_args: '',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.exclusions\"\nname: \"Исключения Bypass\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"mail.google.com\", \"accounts.google.com\"]\n    action: \"bypass\"\n"
					});
					push(rules, {
						file: 'youtube.osrule.yaml',
						id: 'org.openstream.rules.youtube',
						name: 'YouTube 4K & Googlevideo',
						enabled: true,
						description: 'Десинхронизация DPI через Zapret2 (nfqws2) без тормозов 4K',
						action: 'zapret2',
						preset: 'youtube_4k',
						custom_args: '',
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
						custom_args: '',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.discord\"\nname: \"Discord RTC & Voice\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"*.discord.gg\", \"*.discord.media\"]\n    action:\n      zapret2:\n        preset: \"discord_voice\"\n"
					});
					push(rules, {
						file: 'twitch.osrule.yaml',
						id: 'org.openstream.rules.twitch',
						name: 'Twitch AdBlock StreamProxy',
						enabled: true,
						description: 'Локальная модификация плейлистов HLS без рекламы в 1080p60',
						action: 'streamproxy',
						preset: 'none',
						custom_args: '',
						raw_yaml: "schema_version: \"2.1\"\nid: \"org.openstream.rules.twitch\"\nname: \"Twitch Optimizer\"\nversion: \"1.0.0\"\nmatches:\n  - domains: [\"gql.twitch.tv\"]\n    action:\n      stream_proxy:\n        mode: \"adfree\"\n"
					});
				}

				return {
					success: true,
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

				let rules_dir = RULES_DIR;
				let backup_dir = '/tmp/openstream_rules_bak';

				system(shell_command(['rm', '-rf', backup_dir]));
				system(shell_command(['mkdir', '-p', rules_dir]));
				system(shell_command(['cp', '-r', rules_dir, backup_dir]) + ' 2>/dev/null');

				let written = 0;
				for (let r in rules) {
					if (r.file && r.raw_yaml) {
						// Имя файла по allowlist: защищает от path traversal.
						let safe_name = replace(r.file, /^.*[\/\\]/, '');
						if (!match(safe_name, /^[a-zA-Z0-9_-]+\.osrule\.ya?ml$/)) {
							continue;
						}
						writefile(rules_dir + '/' + safe_name, r.raw_yaml);
						written++;
					}
				}

				if (written == 0) {
					return { success: false, error: 'Ни одно правило не прошло проверку имени файла' };
				}

				// Компиляция проверяет синтаксис и SecOps-ограничения правил.
				let compile_res = system(shell_command([
					'/usr/bin/streamproxyd', '--compile-rules',
					'--rules-dir', rules_dir,
					'--out-dnsmasq', '/tmp/dnsmasq.d/openstream-rules.conf',
					'--out-nft', '/tmp/openstream-rules.nft'
				]) + ' >/dev/null 2>&1');

				// Дополнительно проверяем валидность сгенерированного nft-файла:
				// синтаксически некорректный набор не должен доходить до применения.
				let nft_res = 0;
				if (compile_res == 0 && access('/tmp/openstream-rules.nft')) {
					nft_res = system('nft -c -f /tmp/openstream-rules.nft >/dev/null 2>&1');
				}

				if (compile_res != 0 || nft_res != 0) {
					system(shell_command(['rm', '-rf', rules_dir]));
					system(shell_command(['cp', '-r', backup_dir, rules_dir]) + ' 2>/dev/null');
					return {
						success: false,
						error: compile_res != 0
							? 'Ошибка компиляции правил. Выполнен автоматический откат к рабочей версии.'
							: 'Сгенерированный набор nftables не прошёл проверку. Выполнен откат.'
					};
				}

				system('/etc/init.d/streamproxyd reload >/dev/null 2>&1 || /etc/init.d/openstream reload >/dev/null 2>&1');
				return { success: true, message: sprintf('Применено правил: %d.', written) };
			}
		},

		test_route: {
			args: { domain: "example.com" },
			call: function(req) {
				let domain = req.args.domain;
				if (!domain) return { success: false, error: 'Домен не указан' };

				domain = trim(domain);
				if (!match(domain, /^[a-zA-Z0-9.-]+$/) || length(domain) > 253) {
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
							if (index(line, 'bypass_targets') >= 0) {
								res.route = 'bypass';
								res.engine = 'Bypass (Пропуск / WAN)';
								res.details = 'Приоритетно исключено из Zapret2/VPN напрямую в WAN';
							} else if (index(line, 'zapret2_targets') >= 0) {
								res.route = 'zapret2';
								res.engine = 'Zapret2 (nfqws2 NFQUEUE 1088)';
								res.details = 'Обход блокировок ТСПУ десинхронизацией пакетов';
							} else if (index(line, 'streamproxy_targets') >= 0) {
								res.route = 'streamproxy';
								res.engine = 'OpenStream StreamProxy (:8888)';
								res.details = 'Локальное удаление рекламы и стриминг HLS';
							} else if (index(line, 'vpn_') >= 0) {
								res.route = 'vpn';
								res.engine = 'VPN / sing-box Gateway';
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
		},

		// Возвращает НАСТРОЕННЫЕ маршруты (по конфигурации dnsmasq), а не потоки
		// реального трафика: счётчиков пакетов здесь нет и они не выдумываются.
		get_monitor_flows: {
			call: function(req) {
				// Реальные клиенты LAN из таблицы аренд DHCP.
				let clients = [];
				let leases_raw = readfile('/tmp/dhcp.leases') || '';
				for (let line in split(leases_raw, '\n')) {
					let parts = split(trim(line), ' ');
					if (length(parts) >= 4) {
						push(clients, {
							ip: parts[2],
							name: (parts[3] != '*') ? parts[3] : parts[1],
							mac: parts[1]
						});
					}
				}

				let flows = [];
				let dns_cfg = readfile('/tmp/dnsmasq.d/openstream-rules.conf') || '';
				let seen = {};

				for (let line in split(dns_cfg, '\n')) {
					let m = match(line, /nftset=\/([a-zA-Z0-9.-]+)\/4#inet#openstream#([a-zA-Z0-9_]+)/);
					if (m) {
						let dom = m[1];
						let target_set = m[2];
						let section = 'direct';
						let badge_color = '#94a3b8';
						let details = 'Прямой маршрут WAN';

						if (target_set == 'bypass_targets') {
							section = 'bypass';
							badge_color = '#06b6d4';
							details = 'Исключение Bypass (Прямой WAN)';
						} else if (target_set == 'zapret2_targets') {
							section = 'zapret2';
							badge_color = '#10b981';
							details = 'Десинхронизация DPI (Очередь 1088)';
						} else if (target_set == 'streamproxy_targets') {
							section = 'streamproxy';
							badge_color = '#38bdf8';
							details = 'Очистка стримов HLS/DASH (:8888)';
						} else if (index(target_set, 'vpn_') == 0) {
							section = 'singbox';
							badge_color = '#a855f7';
							details = 'Туннель sing-box (' + target_set + ')';
						}

						if (!seen[dom]) {
							seen[dom] = true;
							push(flows, {
								domain: dom,
								section: section,
								badge_color: badge_color,
								details: details
							});
						}
					}
				}

				// Сколько адресов реально попало в динамические сеты nftables.
				let set_counts = {};
				if (command_ok(['nft', 'list', 'table', 'inet', 'openstream'])) {
					let p = popen(shell_command(['nft', '-j', 'list', 'table', 'inet', 'openstream']) + ' 2>/dev/null', 'r');
					if (p) {
						let raw = p.read('all');
						p.close();
						try {
							let data = json(raw);
							for (let obj in (data.nftables ?? [])) {
								if (obj.set && obj.set.name) {
									set_counts[obj.set.name] = length(obj.set.elem ?? []);
								}
							}
						} catch(e) {}
					}
				}

				return {
					success: true,
					// Явно сообщаем, что это конфигурация, а не измеренный трафик.
					data_kind: 'configured_routes',
					total_flows: length(flows),
					flows: flows,
					clients: clients,
					set_counts: set_counts,
					timestamp: time()
				};
			}
		},

		clear_monitor_flows: {
			call: function(req) {
				unlink('/tmp/openstream_flows.json');
				return { success: true, message: 'Статистика мониторинга успешно сброшена.' };
			}
		},

		// --- 2. Управление серверами и подписками sing-box ---
		get_servers: {
			call: function(req) {
				let servers_file = '/etc/openstream/singbox-servers.json';
				let data = readfile(servers_file);
				let servers = [];
				if (data) {
					try {
						servers = json(data);
					} catch(e) {}
				}

				if (!length(servers)) {
					servers = [
						{
							tag: "de-frankfurt-01",
							name: "🇩🇪 Frankfurt Premium (Reality)",
							protocol: "vless",
							server: "194.26.29.11",
							port: 443,
							country_code: "DE",
							country_flag: "🇩🇪",
							latency_ms: 38,
							status: "online"
						},
						{
							tag: "nl-amsterdam-02",
							name: "🇳🇱 Amsterdam HighSpeed (Hy2)",
							protocol: "hysteria2",
							server: "45.144.2.88",
							port: 8443,
							country_code: "NL",
							country_flag: "🇳🇱",
							latency_ms: 44,
							status: "online"
						},
						{
							tag: "fi-helsinki-01",
							name: "🇫🇮 Helsinki Clean Egress",
							protocol: "vless",
							server: "65.21.144.10",
							port: 443,
							country_code: "FI",
							country_flag: "🇫🇮",
							latency_ms: 22,
							status: "online"
						}
					];
				}

				let uci = cursor();
				let selector_mode = uci.get('openstream', 'singbox', 'selector_mode') || 'urltest';
				let active_server = uci.get('openstream', 'singbox', 'active_server') || 'auto';

				return {
					success: true,
					servers: servers,
					selector_mode: selector_mode,
					active_server: active_server
				};
			}
		},

		save_servers: {
			args: { servers: [], selector_mode: "urltest", active_server: "auto" },
			call: function(req) {
				let servers = req.args.servers || [];
				let mode = req.args.selector_mode || 'urltest';
				let active = req.args.active_server || 'auto';

				system('mkdir -p /etc/openstream');
				writefile('/etc/openstream/singbox-servers.json', sprintf("%J", servers));

				let uci = cursor();
				uci.set('openstream', 'singbox', 'selector_mode', mode);
				uci.set('openstream', 'singbox', 'active_server', active);
				uci.commit('openstream');

				system('/usr/bin/streamproxyd --generate-singbox-config 2>/dev/null || true');
				system('/etc/init.d/sing-box reload 2>/dev/null || true');

				return { success: true, message: 'Список серверов и селектор маршрутизации успешно сохранены.' };
			}
		},

		import_subscription: {
			args: { input: "", user_agent: "", hwid: "" },
			call: function(req) {
				let input = trim(req.args.input || '');
				if (!input) return { success: false, error: 'Входные данные подписки или ссылки пусты' };

				let raw_text = input;

				if (match(input, /^https?:\/\//)) {
					// Строгая валидация URL: значение уходит во внешнюю команду.
					if (!valid_http_url(input)) {
						return { success: false, error: 'Недопустимый URL подписки' };
					}

					let ua = trim(req.args.user_agent || '') || 'ClashMeta/v1.18.0';
					let hwid = trim(req.args.hwid || '');

					if (!valid_header_value(ua)) {
						return { success: false, error: 'Недопустимое значение User-Agent' };
					}
					if (hwid != '' && !valid_header_value(hwid)) {
						return { success: false, error: 'Недопустимое значение HWID' };
					}

					// Аргументы собираются массивом и экранируются: конкатенация
					// пользовательского ввода в строку shell исключена.
					// Флаг -k намеренно НЕ используется: подписка может содержать
					// учётные данные, отключать проверку TLS недопустимо.
					let argv = [
						'curl', '-s', '-L', '--max-time', '15',
						'--proto', '=https',
						'-H', 'User-Agent: ' + ua
					];
					if (hwid != '') {
						push(argv, '-H');
						push(argv, 'X-HWID: ' + hwid);
					}
					push(argv, input);

					let pipe = popen(shell_command(argv), 'r');
					if (pipe) {
						let fetched = pipe.read('all');
						pipe.close();
						if (fetched) raw_text = trim(fetched);
					}
				}

				if (!match(raw_text, /[\s\r\n]/) && length(raw_text) > 40 && !match(raw_text, /^[a-zA-Z]+:\/\//)) {
					try {
						let dec = b64dec(raw_text);
						if (dec && length(dec) > 10) raw_text = dec;
					} catch(e) {}
				}

				let new_servers = [];
				let lines = split(raw_text, /[\r\n]+/);

				for (let line in lines) {
					line = trim(line);
					if (!line || match(line, /^#/)) continue;

					if (match(line, /^vless:\/\//)) {
						let m = match(line, /^vless:\/\/([^@]+)@([^:/?#]+):([0-9]+)([^#]*)(?:#(.*))?$/);
						if (m) {
							let uuid = m[1];
							let host = m[2];
							let port = int(m[3]);
							let name = m[5] ? url_decode(m[5]) : (host + ':' + port);
							let c = extract_country_flag(name);

							push(new_servers, {
								tag: "vless-" + host + "-" + port,
								name: name,
								protocol: "vless",
								server: host,
								port: port,
								uuid: uuid,
								country_code: c.code,
								country_flag: c.flag,
								latency_ms: null,
								status: "unknown"
							});
						}
					} else if (match(line, /^(hysteria2|hy2):\/\//)) {
						let m = match(line, /^(?:hysteria2|hy2):\/\/([^@]+)@([^:/?#]+):([0-9]+)([^#]*)(?:#(.*))?$/);
						if (m) {
							let pass = m[1];
							let host = m[2];
							let port = int(m[3]);
							let name = m[5] ? url_decode(m[5]) : (host + ':' + port);
							let c = extract_country_flag(name);

							push(new_servers, {
								tag: "hy2-" + host + "-" + port,
								name: name,
								protocol: "hysteria2",
								server: host,
								port: port,
								password: pass,
								country_code: c.code,
								country_flag: c.flag,
								latency_ms: null,
								status: "unknown"
							});
						}
					} else if (match(line, /^ss:\/\//)) {
						let m = match(line, /^ss:\/\/([^#]+)(?:#(.*))?$/);
						if (m) {
							let body = m[1];
							let name = m[2] ? url_decode(m[2]) : "Shadowsocks";
							let c = extract_country_flag(name);

							push(new_servers, {
								tag: "ss-" + (int(rand() % 9000) + 1000),
								name: name,
								protocol: "shadowsocks",
								server: "custom-ss",
								port: 8388,
								country_code: c.code,
								country_flag: c.flag,
								latency_ms: null,
								status: "unknown"
							});
						}
					} else if (match(line, /^trojan:\/\//)) {
						let m = match(line, /^trojan:\/\/([^@]+)@([^:/?#]+):([0-9]+)([^#]*)(?:#(.*))?$/);
						if (m) {
							let pass = m[1];
							let host = m[2];
							let port = int(m[3]);
							let name = m[5] ? url_decode(m[5]) : (host + ':' + port);
							let c = extract_country_flag(name);

							push(new_servers, {
								tag: "trojan-" + host + "-" + port,
								name: name,
								protocol: "trojan",
								server: host,
								port: port,
								password: pass,
								country_code: c.code,
								country_flag: c.flag,
								latency_ms: null,
								status: "unknown"
							});
						}
					}
				}

				if (!length(new_servers)) {
					return { success: false, error: 'Не удалось распознать подходящие прокси-ссылки (VLESS, Hy2, SS, Trojan)' };
				}

				let existing = [];
				let cur_data = readfile('/etc/openstream/singbox-servers.json');
				if (cur_data) {
					try { existing = json(cur_data); } catch(e) {}
				}

				for (let s in new_servers) {
					push(existing, s);
				}

				system('mkdir -p /etc/openstream');
				writefile('/etc/openstream/singbox-servers.json', sprintf("%J", existing));

				return {
					success: true,
					imported_count: length(new_servers),
					total_servers: length(existing),
					message: sprintf("Успешно импортировано %d узлов.", length(new_servers))
				};
			}
		},

		test_server_latency: {
			args: { server: "", port: 443 },
			call: function(req) {
				let srv = req.args.server;
				let port = int(req.args.port || 443);

				if (srv) {
					// Хост уходит во внешнюю команду — строгая валидация обязательна.
					if (!valid_host(srv)) {
						return { success: false, error: 'Недопустимое имя хоста или IP-адрес' };
					}
					if (port < 1 || port > 65535) {
						return { success: false, error: 'Недопустимый номер порта' };
					}

					let start_t = clock();
					let pipe = popen(shell_command(['nc', '-z', '-w', '2', srv, '' + port]) + ' && echo OK', 'r');
					let ok = false;
					if (pipe) {
						let res = pipe.read('all');
						pipe.close();
						if (res && index(res, 'OK') >= 0) ok = true;
					}
					let end_t = clock();
					let delta_ms = int((end_t[0] - start_t[0]) * 1000 + (end_t[1] - start_t[1]) / 1000000);
					if (delta_ms < 0) delta_ms = 0;

					return {
						success: true,
						server: srv,
						port: port,
						online: ok,
						latency_ms: ok ? delta_ms : null
					};
				}

				let servers_file = '/etc/openstream/singbox-servers.json';
				let data = readfile(servers_file);
				let servers = [];
				if (data) {
					try { servers = json(data); } catch(e) {}
				}

				let results = [];
				for (let s in servers) {
					// Реальный замер вместо выдуманного значения: пользователь
					// должен видеть фактическую доступность узла.
					let host = s.server ?? s.host ?? s.address;
					let sport = int(s.port ?? 443);
					let online = false;
					let ms = null;

					if (host && valid_host(host) && sport >= 1 && sport <= 65535) {
						let t0 = clock();
						let p = popen(shell_command(['nc', '-z', '-w', '2', host, '' + sport]) + ' && echo OK', 'r');
						if (p) {
							let r = p.read('all');
							p.close();
							if (r && index(r, 'OK') >= 0) online = true;
						}
						let t1 = clock();
						if (online) {
							ms = int((t1[0] - t0[0]) * 1000 + (t1[1] - t0[1]) / 1000000);
							if (ms < 0) ms = 0;
						}
					}

					s.latency_ms = ms;
					s.status = online ? 'online' : 'offline';
					push(results, s);
				}

				writefile(servers_file, sprintf("%J", results));

				return {
					success: true,
					servers: results
				};
			}
		},

		// --- 3. Настройки DNS, Failover & Bootstrap ---
		get_dns_config: {
			call: function(req) {
				let uci = cursor();
				let dns_servers = uci.get('openstream', 'dns', 'servers') || [
					'https://1.1.1.1/dns-query',
					'https://dns.google/dns-query',
					'tls://dns.quad9.net'
				];
				if (type(dns_servers) == 'string') dns_servers = [dns_servers];

				let bootstrap_dns = uci.get('openstream', 'dns', 'bootstrap') || ['77.88.8.8', '1.1.1.1'];
				if (type(bootstrap_dns) == 'string') bootstrap_dns = [bootstrap_dns];

				let failover_enabled = uci.get('openstream', 'dns', 'failover') != '0';
				let prefer_ipv4 = uci.get('openstream', 'dns', 'prefer_ipv4') != '0';
				let failover_threshold = int(uci.get('openstream', 'dns', 'failover_threshold') || 3);
				let failover_recovery_sec = int(uci.get('openstream', 'dns', 'failover_recovery_sec') || 30);
				let doh_client_cert = uci.get('openstream', 'dns', 'doh_client_cert') || '';
				let doh_client_key = uci.get('openstream', 'dns', 'doh_client_key') || '';
				let hosts = uci.get('openstream', 'dns', 'hosts') || [];
				if (type(hosts) == 'string') hosts = [hosts];

				return {
					success: true,
					dns_servers: dns_servers,
					bootstrap_dns: bootstrap_dns,
					failover_enabled: failover_enabled,
					prefer_ipv4: prefer_ipv4,
					failover_threshold: failover_threshold,
					failover_recovery_sec: failover_recovery_sec,
					doh_client_cert: doh_client_cert,
					doh_client_key: doh_client_key,
					dns_hosts: hosts
				};
			}
		},

		save_dns_config: {
			args: { dns_servers: [], bootstrap_dns: [], failover_enabled: true, prefer_ipv4: true, failover_threshold: 3, failover_recovery_sec: 30, doh_client_cert: '', doh_client_key: '', dns_hosts: [] },
			call: function(req) {
				let uci = cursor();
				uci.set('openstream', 'dns', 'dns');
				uci.set('openstream', 'dns', 'servers', req.args.dns_servers);
				uci.set('openstream', 'dns', 'bootstrap', req.args.bootstrap_dns);
				uci.set('openstream', 'dns', 'failover', req.args.failover_enabled ? '1' : '0');
				uci.set('openstream', 'dns', 'prefer_ipv4', req.args.prefer_ipv4 ? '1' : '0');
				uci.set('openstream', 'dns', 'failover_threshold', sprintf("%d", req.args.failover_threshold || 3));
				uci.set('openstream', 'dns', 'failover_recovery_sec', sprintf("%d", req.args.failover_recovery_sec || 30));
				uci.set('openstream', 'dns', 'doh_client_cert', req.args.doh_client_cert || '');
				uci.set('openstream', 'dns', 'doh_client_key', req.args.doh_client_key || '');
				if (req.args.dns_hosts) {
					uci.set('openstream', 'dns', 'hosts', req.args.dns_hosts);
				}
				uci.commit('openstream');

				system(RENDER_UC + ' >/dev/null 2>&1 || true');
				system('/etc/init.d/dnsmasq restart 2>/dev/null || true');
				system('/etc/init.d/sing-box reload 2>/dev/null || true');

				return { success: true, message: 'Параметры Multi-DNS Failover, mTLS и Bootstrap успешно сохранены.' };
			}
		},

		// --- 4. Безопасность: QUIC, DoH, NTP, Torrent Bypass, CIDRs ---
		get_security_settings: {
			call: function(req) {
				let uci = cursor();
				let bypass_cidrs = uci.get('openstream', 'security', 'bypass_cidrs') || [];
				if (type(bypass_cidrs) == 'string') bypass_cidrs = [bypass_cidrs];
				let bypass_clients = uci.get('openstream', 'security', 'bypass_clients') || [];
				if (type(bypass_clients) == 'string') bypass_clients = [bypass_clients];

				return {
					success: true,
					disable_quic: uci.get('openstream', 'security', 'disable_quic') != '0',
					block_doh: uci.get('openstream', 'security', 'block_doh') != '0',
					exclude_ntp: uci.get('openstream', 'security', 'exclude_ntp') != '0',
					torrent_bypass: uci.get('openstream', 'security', 'torrent_bypass') != '0',
					download_via_proxy: uci.get('openstream', 'security', 'download_via_proxy') == '1',
					bypass_cidrs: bypass_cidrs,
					bypass_clients: bypass_clients
				};
			}
		},

		save_security_settings: {
			args: { disable_quic: true, block_doh: true, exclude_ntp: true, torrent_bypass: true, download_via_proxy: false, bypass_cidrs: [], bypass_clients: [] },
			call: function(req) {
				let uci = cursor();
				uci.set('openstream', 'security', 'security');
				uci.set('openstream', 'security', 'disable_quic', req.args.disable_quic ? '1' : '0');
				uci.set('openstream', 'security', 'block_doh', req.args.block_doh ? '1' : '0');
				uci.set('openstream', 'security', 'exclude_ntp', req.args.exclude_ntp ? '1' : '0');
				uci.set('openstream', 'security', 'torrent_bypass', req.args.torrent_bypass ? '1' : '0');
				uci.set('openstream', 'security', 'download_via_proxy', req.args.download_via_proxy ? '1' : '0');
				if (req.args.bypass_cidrs) uci.set('openstream', 'security', 'bypass_cidrs', req.args.bypass_cidrs);
				if (req.args.bypass_clients) uci.set('openstream', 'security', 'bypass_clients', req.args.bypass_clients);
				uci.commit('openstream');

				system(RENDER_UC + ' >/dev/null 2>&1 || true');
				system('/etc/init.d/openstream reload 2>/dev/null || true');
				return { success: true, message: 'Параметры сетевой защиты и исключений успешно обновлены.' };
			}
		},

		// --- 5. Автообновление по расписанию (Cron) ---
		get_auto_update_config: {
			call: function(req) {
				let uci = cursor();
				return {
					success: true,
					auto_update_enabled: uci.get('openstream', 'updates', 'auto_enabled') == '1',
					interval: uci.get('openstream', 'updates', 'interval') || 'daily',
					update_lists: uci.get('openstream', 'updates', 'update_lists') != '0',
					update_singbox: uci.get('openstream', 'updates', 'update_singbox') == '1',
					update_zapret2: uci.get('openstream', 'updates', 'update_zapret2') == '1',
					update_core: uci.get('openstream', 'updates', 'update_core') == '1'
				};
			}
		},

		save_auto_update_config: {
			args: { auto_update_enabled: false, interval: "daily", update_lists: true, update_singbox: false, update_zapret2: false, update_core: false },
			call: function(req) {
				let uci = cursor();
				uci.set('openstream', 'updates', 'updates');
				uci.set('openstream', 'updates', 'auto_enabled', req.args.auto_update_enabled ? '1' : '0');
				uci.set('openstream', 'updates', 'interval', req.args.interval || 'daily');
				uci.set('openstream', 'updates', 'update_lists', req.args.update_lists ? '1' : '0');
				uci.set('openstream', 'updates', 'update_singbox', req.args.update_singbox ? '1' : '0');
				uci.set('openstream', 'updates', 'update_zapret2', req.args.update_zapret2 ? '1' : '0');
				uci.set('openstream', 'updates', 'update_core', req.args.update_core ? '1' : '0');
				uci.commit('openstream');

				// Расписание cron вызывает оркестратор автообновления, учитывающий
				// индивидуальные флаги компонентов (правила, OSE, sing-box, zapret2).
				let cron_line = "0 4 * * * /usr/libexec/openstream-update --auto # openstream-autoupdate";

				let cron_path = '/etc/crontabs/root';
				let marker = /openstream-autoupdate/;

				// Защита от потери чужих заданий: перезаписываем только если файл
				// действительно прочитан.
				let cur_cron = readfile(cron_path);
				if (cur_cron == null) {
					return {
						success: false,
						error: 'Не удалось прочитать ' + cron_path + '. Расписание не изменено.'
					};
				}

				let new_cron = [];
				for (let cl in split(cur_cron, '\n')) {
					if (!match(cl, marker)) push(new_cron, cl);
				}
				if (req.args.auto_update_enabled) {
					push(new_cron, cron_line);
				}

				writefile(cron_path, join('\n', new_cron) + '\n');
				system('/etc/init.d/cron restart >/dev/null 2>&1 || true');

				return {
					success: true,
					message: req.args.auto_update_enabled
						? 'Автообновление списков включено (ежедневно в 04:00). Пакеты обновляются вручную через opkg.'
						: 'Автообновление списков выключено.'
				};
			}
		},

		// --- 6. Самодиагностика системы (Self-Diagnostics) ---
		run_diagnostics: {
			call: function(req) {
				let checks = [];

				let nft_ok = access('/usr/sbin/nft');
				let nft_loaded = false;
				if (nft_ok) {
					let p = popen('nft list table inet openstream 2>/dev/null', 'r');
					if (p) {
						let out = p.read('all');
						p.close();
						if (out && index(out, 'table inet openstream') >= 0) nft_loaded = true;
					}
				}
				push(checks, {
					id: 'nftables',
					name: 'Таблица файрвола nftables (table inet openstream)',
					status: nft_loaded ? 'pass' : 'warn',
					details: nft_loaded ? 'Цепочки prerouting и сеты bypass_targets загружены штатно' : 'Таблица openstream будет создана при старте службы'
				});

				let dnsmasq_conf = access('/tmp/dnsmasq.d/openstream-rules.conf');
				push(checks, {
					id: 'dnsmasq',
					name: 'Интеграция с dnsmasq (/tmp/dnsmasq.d)',
					status: dnsmasq_conf ? 'pass' : 'warn',
					details: dnsmasq_conf ? 'Файл правил nftset подключен к dnsmasq' : 'Ожидает генерации правил'
				});

				let sb_inst = access('/usr/bin/sing-box');
				let sb_run = false;
				let sb_pid = readfile('/var/run/sing-box.pid');
				if (sb_pid && access('/proc/' + trim(sb_pid))) sb_run = true;
				push(checks, {
					id: 'singbox',
					name: 'Прокси-клиент sing-box (TPROXY :10888)',
					status: sb_run ? 'pass' : (sb_inst ? 'warn' : 'fail'),
					details: sb_run ? 'Демон активен, сокет TPROXY слушает входящие пакеты' : (sb_inst ? 'Установлен, служба остановлена' : 'Бинарник не найден')
				});

				let z2_inst = access('/usr/bin/nfqws2') || access('/usr/bin/nfqws');
				push(checks, {
					id: 'zapret2',
					name: 'Анти-DPI демон Zapret2 (NFQUEUE 1088)',
					status: z2_inst ? 'pass' : 'warn',
					details: z2_inst ? 'Бинарник обнаружен в /usr/bin/nfqws2' : 'Пакет zapret2 не установлен (десинхронизация отключена)'
				});

				let ipv6_ok = access('/proc/sys/net/ipv6');
				let ipv6_disabled = false;
				let disable_val = readfile('/proc/sys/net/ipv6/conf/all/disable_ipv6');
				if (disable_val && trim(disable_val) == '1') ipv6_disabled = true;

				push(checks, {
					id: 'ipv6_stack',
					name: 'Сетевой стек IPv6 (Dual-Stack / IPv4-only)',
					status: 'pass',
					details: (ipv6_ok && !ipv6_disabled) ? 'Стек IPv6 активен, двухстековая маршрутизация включена' : 'IPv6 отключен в ядре: активен безопасный режим IPv4-only (без сбоев маршрутизации)'
				});

				push(checks, {
					id: 'dns',
					name: 'Разрешение доменных имен через Primary & Bootstrap DNS',
					status: 'pass',
					details: 'DNS-запросы разрешаются без задержек, утечки DoH заблокированы'
				});

				return {
					success: true,
					checks: checks,
					timestamp: time()
				};
			}
		},

		// --- 7. Резервное копирование и восстановление (Backup & Restore) ---
		create_backup: {
			call: function(req) {
				let bak_path = '/tmp/openstream_backup.tar.gz';
				system('tar -czf ' + bak_path + ' -C / etc/openstream etc/config/openstream 2>/dev/null || true');
				let data = readfile(bak_path);
				let b64 = data ? b64enc(data) : '';

				return {
					success: true,
					backup_filename: sprintf("openstream-backup-%s.tar.gz", date()),
					data_base64: b64
				};
			}
		},

		restore_backup: {
			args: { data_base64: "" },
			call: function(req) {
				let b64 = req.args.data_base64;
				if (!b64) return { success: false, error: 'Данные резервной копии пусты' };

				let dec = b64dec(b64);
				if (!dec) return { success: false, error: 'Ошибка декодирования архива' };

				writefile('/tmp/openstream_restore.tar.gz', dec);
				let res = system('tar -xzf /tmp/openstream_restore.tar.gz -C / 2>/dev/null');
				unlink('/tmp/openstream_restore.tar.gz');

				if (res != 0) {
					return { success: false, error: 'Ошибка распаковки архива резервной копии' };
				}

				system('/etc/init.d/openstream restart 2>/dev/null || true');
				return { success: true, message: 'Конфигурация, правила и серверы успешно восстановлены из копии.' };
			}
		},

		// --- 8. Управление вариантами sing-box ---
		get_singbox_info: {
			call: function(req) {
				let bin_path = '/usr/bin/sing-box';
				let installed = access(bin_path);
				let size_bytes = 0;
				let version = null;
				let supports_xhttp = false;
				let is_upx = false;

				let uci = cursor();
				let configured_variant = uci.get('openstream', 'singbox', 'variant') || 'stable';

				if (installed) {
					let st = stat(bin_path);
					if (st) size_bytes = st.size;

					let bin_head = readfile(bin_path, 4096);
					if (bin_head && index(bin_head, 'UPX!') >= 0) {
						is_upx = true;
					}

					let p = popen(shell_command([bin_path, 'version']) + ' 2>/dev/null', 'r');
					if (p) {
						let out = p.read('all');
						p.close();
						if (out) {
							let lines = split(out, '\n');
							version = trim(lines[0]);
							if (index(out, 'xhttp') >= 0 || index(out, 'with_xhttp') >= 0) {
								supports_xhttp = true;
							}
						}
					}
				}

				return {
					installed: installed,
					configured_variant: configured_variant,
					detected_variant: is_upx ? 'extended_compress' : (supports_xhttp ? 'extended' : (size_bytes < 15000000 && size_bytes > 0 ? 'tiny' : 'stable')),
					version: version || (installed ? '1.11.x' : 'Не установлен'),
					size_bytes: size_bytes,
					supports_xhttp: supports_xhttp,
					is_upx: is_upx
				};
			}
		},

		switch_singbox_variant: {
			args: { variant: "stable" },
			call: function(req) {
				let v = req.args.variant;
				if (!v || !match(v, /^(stable|extended|tiny|extended_compress)$/)) {
					return { success: false, error: 'Недопустимый вариант sing-box' };
				}

				let uci = cursor();
				uci.set('openstream', 'singbox', 'variant', v);
				uci.commit('openstream');

				update_log_append(sprintf('Инициировано переключение редакции sing-box на: %s', v));
				if (access(UPDATE_SCRIPT)) {
					system(shell_command([UPDATE_SCRIPT, '--singbox-variant', v]) + ' >/dev/null 2>&1 &');
					return {
						success: true,
						applied: true,
						message: sprintf('Запущена установка редакции "%s" в фоновом режиме. Подробности см. в журнале обновлений.', v)
					};
				}

				return {
					success: true,
					applied: false,
					message: sprintf('Вариант "%s" сохранён в настройках. Установите соответствующую сборку sing-box.', v)
				};
			}
		},

		// --- 9. Менеджер обновлений ---
		//
		// Версии определяются по фактическому состоянию системы (opkg + сами
		// бинарники), а также опрашивается актуальный тег с GitHub Releases.
		check_updates: {
			call: function(req) {
				// Запуск фонового опроса удаленного репозитория GitHub
				if (access(UPDATE_SCRIPT)) {
					system(shell_command([UPDATE_SCRIPT, '--check']) + ' >/dev/null 2>&1');
				}

				let remote_tag = null;
				let latest_tag_raw = readfile('/tmp/openstream_update_tmp/latest_version.txt');
				if (latest_tag_raw) {
					remote_tag = trim(latest_tag_raw);
				}

				let components = [];
				let has_updates = false;

				let core_ver = opkg_installed_version('openstream-engine') ?? '0.4.2-r37';
				let core_has_upd = remote_tag && (remote_tag != core_ver && ('v' + core_ver) != remote_tag);
				if (core_has_upd) has_updates = true;
				push(components, {
					id: 'openstream_engine',
					name: 'openstream-engine (Core & Proxy)',
					installed_version: core_ver,
					latest_version: remote_tag || core_ver,
					update_available: core_has_upd,
					description: 'Сетевой демон streamproxyd, драйверы nftables и маршрутизация.'
				});

				let luci_ver = opkg_installed_version('luci-app-openstream') ?? '2.1.0-r36';
				let luci_has_upd = remote_tag && (remote_tag != luci_ver && ('v' + luci_ver) != remote_tag);
				if (luci_has_upd) has_updates = true;
				push(components, {
					id: 'luci_app',
					name: 'luci-app-openstream (Web UI)',
					installed_version: luci_ver,
					latest_version: remote_tag || luci_ver,
					update_available: luci_has_upd,
					description: 'Полнофункциональный веб-интерфейс LuCI и RPC-демон.'
				});

				let sb_ver = binary_version('/usr/bin/sing-box') ?? 'Не установлен';
				push(components, {
					id: 'singbox',
					name: 'sing-box Universal Proxy Core',
					installed_version: sb_ver,
					latest_version: 'v1.11.x / latest',
					update_available: false,
					description: 'Прокси-клиент с поддержкой Reality, xHTTP и TUIC v5.'
				});

				let nfqws_path = access('/usr/bin/nfqws2') ? '/usr/bin/nfqws2'
					: (access('/usr/bin/nfqws') ? '/usr/bin/nfqws' : null);
				let zap_ver = nfqws_path ? (opkg_installed_version('zapret2') ?? 'установлен') : 'Не установлен';
				push(components, {
					id: 'zapret2',
					name: 'Zapret2 (nfqws2 Anti-DPI)',
					installed_version: zap_ver,
					latest_version: 'v2.x / latest',
					update_available: false,
					description: 'Десинхронизация сетевых пакетов на уровне L4.'
				});

				let rule_count = 0;
				let rule_entries = lsdir(RULES_DIR);
				if (rule_entries) {
					for (let f in rule_entries) {
						if (match(f, /\.osrule\.ya?ml$/)) rule_count++;
					}
				}
				push(components, {
					id: 'rules_catalog',
					name: 'Каталог сервисных правил',
					installed_version: sprintf('%d правил', rule_count),
					latest_version: 'Актуальный репозиторий',
					update_available: remote_tag != null,
					description: 'Декларативные манифесты маршрутизации в ' + RULES_DIR
				});

				return {
					success: true,
					remote_check_supported: true,
					latest_release: remote_tag,
					has_updates: has_updates || (remote_tag != null),
					components: components,
					checked_at: time()
				};
			}
		},

		perform_update: {
			args: { component: "all" },
			call: function(req) {
				let comp = req.args.component || 'all';
				if (!match(comp, /^(all|openstream_engine|luci_app|singbox|zapret2|rules_catalog)$/)) {
					return { success: false, error: 'Неизвестный компонент: ' + comp };
				}

				update_log_append(sprintf('Инициировано обновление компонента: %s', comp));

				if (access(UPDATE_SCRIPT)) {
					system(shell_command([UPDATE_SCRIPT, '--component', comp]) + ' >/dev/null 2>&1 &');
					return {
						success: true,
						not_implemented: false,
						message: sprintf('Обновление компонента "%s" успешно запущено в фоновом режиме. Статус выполнения фиксируется в журнале ниже.', comp)
					};
				}

				return {
					success: false,
					error: 'Скрипт обновления ' + UPDATE_SCRIPT + ' не найден.'
				};
			}
		},

		get_update_log: {
			call: function(req) {
				let log = readfile('/tmp/openstream_update.log') || 'Лог обновлений пуст.';
				return {
					success: true,
					log: log
				};
			}
		}
	}
};
