'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Services, Multi-DNS Failover & Security Hub
 * Mobile-First, OLED Dark Card Layout, No HTML Tables (Rules #8, #10, #11)
 */

var callGetDnsConfig = rpc.declare({
	object: 'openstream',
	method: 'get_dns_config',
	expect: { success: true }
});

var callSaveDnsConfig = rpc.declare({
	object: 'openstream',
	method: 'save_dns_config',
	params: [ 'dns_servers', 'bootstrap_dns', 'failover_enabled', 'prefer_ipv4' ],
	expect: { success: true }
});

var callGetSecurity = rpc.declare({
	object: 'openstream',
	method: 'get_security_settings',
	expect: { success: true }
});

var callSaveSecurity = rpc.declare({
	object: 'openstream',
	method: 'save_security_settings',
	params: [ 'disable_quic', 'block_doh', 'exclude_ntp', 'download_via_proxy' ],
	expect: { success: true }
});

var callCreateBackup = rpc.declare({
	object: 'openstream',
	method: 'create_backup',
	expect: { success: true }
});

var callRestoreBackup = rpc.declare({
	object: 'openstream',
	method: 'restore_backup',
	params: [ 'data_base64' ],
	expect: { success: true }
});

return view.extend({
	load: function() {
		return Promise.all([
			callGetDnsConfig().catch(function() { return {}; }),
			callGetSecurity().catch(function() { return {}; })
		]);
	},

	render: function(results) {
		var dnsCfg = results[0] || {};
		var secCfg = results[1] || {};

		var viewRoot = E('div', { 'class': 'cbi-map' });

		var styleNode = E('style', {}, `
			:root {
				--os-bg: #020617;
				--os-card: #0b1329;
				--os-border: #1e293b;
				--os-blue: #3b82f6;
				--os-emerald: #10b981;
				--os-rose: #f43f5e;
				--os-amber: #f59e0b;
				--os-purple: #a855f7;
				--os-cyan: #06b6d4;
				--os-muted: #64748b;
				--os-radius: 12px;
			}
			.os-card {
				background: var(--os-card);
				border: 1px solid var(--os-border);
				border-radius: var(--os-radius);
				padding: 20px;
				margin-bottom: 16px;
				box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
				transition: border-color 0.2s;
			}
			.os-card:hover { border-color: #475569; }
			.os-btn {
				display: inline-flex;
				align-items: center;
				gap: 8px;
				padding: 9px 18px;
				font-size: 13px;
				font-weight: 600;
				border-radius: 8px;
				cursor: pointer;
				border: none;
				transition: all 0.2s;
			}
			.os-btn-primary { background: #3b82f6; color: #fff; }
			.os-btn-primary:hover { background: #2563eb; }
			.os-btn-success { background: #10b981; color: #fff; }
			.os-btn-success:hover { background: #059669; }
			.os-btn-dark { background: #1e293b; color: #f8fafc; border: 1px solid var(--os-border); }
			.os-btn-dark:hover { background: #334155; }
			.os-input, .os-textarea {
				background: #020617;
				border: 1px solid var(--os-border);
				border-radius: 8px;
				padding: 9px 12px;
				color: #fff;
				font-size: 13px;
				outline: none;
				width: 100%;
				box-sizing: border-box;
			}
			.os-input:focus, .os-textarea:focus { border-color: var(--os-blue); }
			.os-switch {
				position: relative;
				display: inline-block;
				width: 44px;
				height: 24px;
			}
			.os-switch input { opacity: 0; width: 0; height: 0; }
			.os-slider {
				position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0;
				background-color: #334155;
				transition: .2s;
				border-radius: 24px;
			}
			.os-slider:before {
				position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px;
				background-color: white;
				transition: .2s;
				border-radius: 50%;
			}
			input:checked + .os-slider { background-color: var(--os-emerald); }
			input:checked + .os-slider:before { transform: translateX(20px); }
		`);
		viewRoot.appendChild(styleNode);

		// Header
		var headerNode = E('div', { 'class': 'os-card' }, [
			E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;' }, [
				E('div', {}, [
					E('h2', { 'style': 'margin: 0 0 6px 0; font-size: 22px; font-weight: 800; color: #fff;' }, '⚙️ Сетевые сервисы, Multi-DNS и Безопасность'),
					E('p', { 'style': 'margin: 0; color: var(--os-muted); font-size: 13px;' },
						'Управление каскадным DNS с Bootstrap-резолвером, блокировкой QUIC/DoH, защитой NTP и резервными копиями.'
					)
				]),
				E('button', {
					'class': 'os-btn os-btn-success',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = 'Применение настроек...';

						var dnsRaw = document.getElementById('os-dns-servers') ? document.getElementById('os-dns-servers').value.trim() : '';
						var bootRaw = document.getElementById('os-dns-bootstrap') ? document.getElementById('os-dns-bootstrap').value.trim() : '';
						var dnsList = dnsRaw ? dnsRaw.split(/[\r\n]+/) : [];
						var bootList = bootRaw ? bootRaw.split(/[\r\n]+/) : [];

						var failover = document.getElementById('os-dns-failover') ? document.getElementById('os-dns-failover').checked : true;
						var preferIpv4 = document.getElementById('os-dns-ipv4') ? document.getElementById('os-dns-ipv4').checked : true;

						var disQuic = document.getElementById('os-sec-quic') ? document.getElementById('os-sec-quic').checked : true;
						var blkDoh = document.getElementById('os-sec-doh') ? document.getElementById('os-sec-doh').checked : true;
						var exNtp = document.getElementById('os-sec-ntp') ? document.getElementById('os-sec-ntp').checked : true;
						var dlProxy = document.getElementById('os-sec-dlproxy') ? document.getElementById('os-sec-dlproxy').checked : false;

						Promise.all([
							callSaveDnsConfig(dnsList, bootList, failover, preferIpv4),
							callSaveSecurity(disQuic, blkDoh, exNtp, dlProxy)
						]).then(function() {
							btn.disabled = false;
							btn.innerText = '💾 Сохранить все параметры';
							ui.addNotification(null, E('p', {}, '✓ Параметры DNS и сетевой безопасности успешно сохранены!'), 'info');
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 Сохранить все параметры';
							ui.addNotification(null, E('p', {}, '❌ Ошибка RPC: ' + err), 'error');
						});
					}
				}, [ '💾 Сохранить все параметры' ])
			])
		]);
		viewRoot.appendChild(headerNode);

		// Section 1: Multi-DNS Failover & Bootstrap DNS (forkop reference)
		var dnsServersStr = (dnsCfg.dns_servers || [
			'https://1.1.1.1/dns-query',
			'https://dns.google/dns-query',
			'tls://dns.quad9.net'
		]).join('\n');

		var bootstrapStr = (dnsCfg.bootstrap_dns || [
			'77.88.8.8',
			'1.1.1.1'
		]).join('\n');

		var dnsCard = E('div', { 'class': 'os-card' }, [
			E('h3', { 'style': 'margin: 0 0 6px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '🛡️ Multi-DNS Failover & Bootstrap Resolver'),
			E('p', { 'style': 'margin: 0 0 16px 0; color: var(--os-muted); font-size: 13px;' },
				'Если первый DNS-сервер не отвечает, запросы автоматически переключаются на резервный (Failover). Bootstrap DNS исключает дедлоки резолва DoH.'
			),
			E('div', { 'style': 'display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 16px;' }, [
				E('div', {}, [
					E('label', { 'style': 'display: block; font-size: 12px; font-weight: 600; color: var(--os-muted); margin-bottom: 6px;' }, 'ОСНОВНЫЕ DNS СЕРВЕРЫ (DoH / DoT / UDP - по одному на строку)'),
					E('textarea', {
						'class': 'os-textarea',
						'id': 'os-dns-servers',
						'rows': 4
					}, dnsServersStr)
				]),
				E('div', {}, [
					E('label', { 'style': 'display: block; font-size: 12px; font-weight: 600; color: var(--os-muted); margin-bottom: 6px;' }, 'BOOTSTRAP DNS (Статические IP без шифрования для DoH)'),
					E('textarea', {
						'class': 'os-textarea',
						'id': 'os-dns-bootstrap',
						'rows': 4
					}, bootstrapStr)
				])
			]),
			E('div', { 'style': 'display: flex; gap: 20px; flex-wrap: wrap; margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--os-border);' }, [
				E('label', { 'style': 'display: flex; align-items: center; gap: 10px; cursor: pointer;' }, [
					E('label', { 'class': 'os-switch' }, [
						E('input', { 'type': 'checkbox', 'id': 'os-dns-failover', 'checked': dnsCfg.failover_enabled !== false }),
						E('span', { 'class': 'os-slider' })
					]),
					E('span', { 'style': 'font-size: 13px; font-weight: 600; color: #fff;' }, 'Отказоустойчивость DNS (Auto-Failover)')
				]),
				E('label', { 'style': 'display: flex; align-items: center; gap: 10px; cursor: pointer;' }, [
					E('label', { 'class': 'os-switch' }, [
						E('input', { 'type': 'checkbox', 'id': 'os-dns-ipv4', 'checked': dnsCfg.prefer_ipv4 !== false }),
						E('span', { 'class': 'os-slider' })
					]),
					E('span', { 'style': 'font-size: 13px; font-weight: 600; color: #fff;' }, 'Приоритет IPv4 (Prefer IPv4)')
				])
			])
		]);
		viewRoot.appendChild(dnsCard);

		// Section 2: Сетевая безопасность и перехват
		var secCard = E('div', { 'class': 'os-card' }, [
			E('h3', { 'style': 'margin: 0 0 6px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '🔒 Сетевая безопасность & Фильтрация протоколов'),
			E('p', { 'style': 'margin: 0 0 16px 0; color: var(--os-muted); font-size: 13px;' },
				'Оптимизация сетевого стека nftables: сброс QUIC для надёжного YouTube 4K в Zapret2, изоляция DoH и пропуск NTP.'
			),
			E('div', { 'style': 'display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px;' }, [
				E('div', { 'style': 'background: #020617; border: 1px solid var(--os-border); border-radius: 8px; padding: 14px;' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'style': 'font-weight: 700; font-size: 14px; color: #fff;' }, 'Блокировать QUIC (UDP 443)'),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-quic', 'checked': secCfg.disable_quic !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'style': 'font-size: 12px; color: var(--os-muted); margin-top: 6px;' },
						'Форсирует TCP TLS 1.3 в браузерах. Критически важно для работы обхода замедления YouTube в Zapret2.'
					)
				]),
				E('div', { 'style': 'background: #020617; border: 1px solid var(--os-border); border-radius: 8px; padding: 14px;' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'style': 'font-weight: 700; font-size: 14px; color: #fff;' }, 'Блокировать DoH/DoT (TCP 853)'),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-doh', 'checked': secCfg.block_doh !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'style': 'font-size: 12px; color: var(--os-muted); margin-top: 6px;' },
						'Предотвращает утечки DNS через сторонние браузерные резолверы мимо таблиц правил dnsmasq.'
					)
				]),
				E('div', { 'style': 'background: #020617; border: 1px solid var(--os-border); border-radius: 8px; padding: 14px;' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'style': 'font-weight: 700; font-size: 14px; color: #fff;' }, 'Исключить NTP (UDP 123)'),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-ntp', 'checked': secCfg.exclude_ntp !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'style': 'font-size: 12px; color: var(--os-muted); margin-top: 6px;' },
						'Прямой пропуск пакетов синхронизации времени без влияния прокси и туннелей.'
					)
				]),
				E('div', { 'style': 'background: #020617; border: 1px solid var(--os-border); border-radius: 8px; padding: 14px;' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'style': 'font-weight: 700; font-size: 14px; color: #fff;' }, 'Загрузка обновлений через VPN'),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-dlproxy', 'checked': secCfg.download_via_proxy === true }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'style': 'font-size: 12px; color: var(--os-muted); margin-top: 6px;' },
						'Использовать активный туннель sing-box для скачивания бинарников с GitHub при региональных блокировках.'
					)
				])
			])
		]);
		viewRoot.appendChild(secCard);

		// Section 3: Резервное копирование и восстановление (Backup & Restore)
		var bakCard = E('div', { 'class': 'os-card' }, [
			E('h3', { 'style': 'margin: 0 0 6px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '💾 Резервное копирование и восстановление (Backup & Restore)'),
			E('p', { 'style': 'margin: 0 0 16px 0; color: var(--os-muted); font-size: 13px;' },
				'Экспорт всех правил, конфигураций UCI и серверов в архив tar.gz или восстановление при перепрошивке роутера.'
			),
			E('div', { 'style': 'display: flex; gap: 16px; flex-wrap: wrap;' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = 'Создание архива...';
						callCreateBackup().then(function(res) {
							btn.disabled = false;
							btn.innerText = '📥 Скачать бэкап (.tar.gz)';
							if (res && res.data_base64) {
								var link = document.createElement('a');
								link.href = 'data:application/gzip;base64,' + res.data_base64;
								link.download = res.backup_filename || 'openstream-backup.tar.gz';
								document.body.appendChild(link);
								link.click();
								document.body.removeChild(link);
								ui.addNotification(null, E('p', {}, '✓ Архив резервной копии успешно скачан!'), 'info');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '📥 Скачать бэкап (.tar.gz)';
							ui.addNotification(null, E('p', {}, '❌ Ошибка создания бэкапа: ' + err), 'error');
						});
					}
				}, [ '📥 Скачать бэкап (.tar.gz)' ]),
				E('label', {
					'class': 'os-btn os-btn-dark',
					'style': 'cursor: pointer;'
				}, [
					'📤 Восстановить из файла',
					E('input', {
						'type': 'file',
						'style': 'display: none;',
						'accept': '.tar.gz,.tgz',
						'change': function(ev) {
							var file = ev.target.files[0];
							if (!file) return;
							var reader = new FileReader();
							reader.onload = function() {
								var b64 = reader.result.split(',')[1];
								ui.showModal('Восстановление конфигурации', [
									E('p', { 'class': 'spinning' }, 'Распаковка архива и перезапуск служб...')
								]);
								callRestoreBackup(b64).then(function(r) {
									setTimeout(function() {
										ui.hideModal();
										ui.addNotification(null, E('p', {}, '✓ ' + (r.message || 'Конфигурация восстановлена!')), 'info');
										window.location.reload();
									}, 1500);
								}).catch(function(err) {
									ui.hideModal();
									ui.addNotification(null, E('p', {}, '❌ Ошибка восстановления: ' + err), 'error');
								});
							};
							reader.readAsDataURL(file);
						}
					})
				])
			])
		]);
		viewRoot.appendChild(bakCard);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
