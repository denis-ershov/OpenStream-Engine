'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Servers & Subscriptions View (sing-box 4-flavor)
 * Mobile-First, OLED Dark Card Layout, No HTML Tables (Rules #8, #10, #11)
 */

var callGetServers = rpc.declare({
	object: 'openstream',
	method: 'get_servers',
	expect: { success: true }
});

var callSaveServers = rpc.declare({
	object: 'openstream',
	method: 'save_servers',
	params: [ 'servers', 'selector_mode', 'active_server' ],
	expect: { success: true }
});

var callImportSubscription = rpc.declare({
	object: 'openstream',
	method: 'import_subscription',
	params: [ 'input' ],
	expect: { success: true }
});

var callTestLatency = rpc.declare({
	object: 'openstream',
	method: 'test_server_latency',
	params: [ 'server', 'port' ],
	expect: { success: true }
});

return view.extend({
	load: function() {
		return Promise.all([
			callGetServers().catch(function() {
				return {
					servers: [],
					selector_mode: 'urltest',
					active_server: 'auto'
				};
			})
		]);
	},

	render: function(results) {
		var initial = results[0] || {};
		var servers = initial.servers || [];
		var selectorMode = initial.selector_mode || 'urltest';
		var activeServer = initial.active_server || 'auto';

		var viewRoot = E('div', { 'class': 'cbi-map' });

		// Стили дизайн-системы (OLED Dark, Linear/Apple modern UI)
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
			.os-badge {
				display: inline-flex;
				align-items: center;
				font-size: 11px;
				font-weight: 700;
				padding: 3px 8px;
				border-radius: 6px;
				text-transform: uppercase;
				letter-spacing: 0.05em;
			}
			.os-badge-blue { background: rgba(59,130,246,0.2); color: var(--os-blue); border: 1px solid rgba(59,130,246,0.4); }
			.os-badge-emerald { background: rgba(16,185,129,0.2); color: var(--os-emerald); border: 1px solid rgba(16,185,129,0.4); }
			.os-badge-purple { background: rgba(168,85,247,0.2); color: var(--os-purple); border: 1px solid rgba(168,85,247,0.4); }
			.os-badge-amber { background: rgba(245,158,11,0.2); color: var(--os-amber); border: 1px solid rgba(245,158,11,0.4); }
			.os-badge-cyan { background: rgba(6,182,212,0.2); color: var(--os-cyan); border: 1px solid rgba(6,182,212,0.4); }
			.os-input, .os-select, .os-textarea {
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
			.os-input:focus, .os-select:focus, .os-textarea:focus { border-color: var(--os-blue); }
			.os-server-grid {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
				gap: 16px;
				margin-top: 16px;
			}
			.os-server-card {
				background: #020617;
				border: 1px solid var(--os-border);
				border-radius: 10px;
				padding: 16px;
				display: flex;
				flex-direction: column;
				gap: 12px;
				position: relative;
			}
		`);
		viewRoot.appendChild(styleNode);

		// Header
		var headerNode = E('div', { 'class': 'os-card', 'style': 'margin-bottom: 20px;' }, [
			E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;' }, [
				E('div', {}, [
					E('h2', { 'style': 'margin: 0 0 6px 0; font-size: 22px; font-weight: 800; color: #fff;' }, '🌐 Серверы и подписки sing-box'),
					E('p', { 'style': 'margin: 0; color: var(--os-muted); font-size: 13px;' },
						'Поддержка VLESS Reality/xHTTP, Hysteria 2, TUIC, Shadowsocks 2022, Trojan и автовыбора узла по наименьшему пингу.'
					)
				]),
				E('div', { 'style': 'display: flex; gap: 10px;' }, [
					E('button', {
						'class': 'os-btn os-btn-primary',
						'id': 'os-btn-test-ping',
						'click': function(ev) {
							var btn = ev.target;
							btn.disabled = true;
							btn.innerText = '⏳ Замер задержки...';
							callTestLatency('', 443).then(function(res) {
								btn.disabled = false;
								btn.innerText = '⚡ Проверить пинг';
								if (res && res.servers) {
									servers = res.servers;
									renderServerCards();
									ui.addNotification(null, E('p', {}, '✓ Замер задержки серверов успешно выполнен!'), 'info');
								}
							}).catch(function(err) {
								btn.disabled = false;
								btn.innerText = '⚡ Проверить пинг';
								ui.addNotification(null, E('p', {}, '❌ Ошибка проверки пинга: ' + err), 'error');
							});
						}
					}, [ '⚡ Проверить пинг' ]),
					E('button', {
						'class': 'os-btn os-btn-success',
						'click': function(ev) {
							var btn = ev.target;
							btn.disabled = true;
							btn.innerText = 'Сохранение...';
							callSaveServers(servers, selectorMode, activeServer).then(function() {
								btn.disabled = false;
								btn.innerText = '💾 Сохранить и применить';
								ui.addNotification(null, E('p', {}, '✓ Список серверов и outbounds sing-box успешно применены!'), 'info');
							}).catch(function(err) {
								btn.disabled = false;
								btn.innerText = '💾 Сохранить и применить';
								ui.addNotification(null, E('p', {}, '❌ Ошибка RPC: ' + err), 'error');
							});
						}
					}, [ '💾 Сохранить и применить' ])
				])
			])
		]);
		viewRoot.appendChild(headerNode);

		// Selector Mode Switcher (URLTest vs Fixed)
		var modeCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'style': 'font-weight: 700; font-size: 15px; margin-bottom: 12px; color: #fff;' }, '⚡ Алгоритм выбора исходящего шлюза (Outbound Selector)'),
			E('div', { 'style': 'display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px;' }, [
				E('div', {}, [
					E('label', { 'style': 'display: block; font-size: 12px; font-weight: 600; color: var(--os-muted); margin-bottom: 6px;' }, 'РЕЖИМ ВЫБОРА ШЛЮЗА'),
					E('select', {
						'class': 'os-select',
						'change': function(ev) {
							selectorMode = ev.target.value;
							renderServerCards();
						}
					}, [
						E('option', { 'value': 'urltest', 'selected': selectorMode === 'urltest' }, '⚡ Автовыбор наименьшей задержки (URLTest Auto-Best)'),
						E('option', { 'value': 'manual', 'selected': selectorMode === 'manual' }, '📌 Фиксированный сервер (Manual Choice)')
					])
				]),
				E('div', {}, [
					E('label', { 'style': 'display: block; font-size: 12px; font-weight: 600; color: var(--os-muted); margin-bottom: 6px;' }, 'ТЕКУЩИЙ АКТИВНЫЙ УЗЕЛ'),
					E('div', { 'style': 'font-size: 14px; font-weight: 700; padding: 9px 12px; background: #020617; border: 1px solid var(--os-border); border-radius: 8px; color: var(--os-emerald);' },
						selectorMode === 'urltest' ? '⚡ Автоматический выбор (Best Latency < 50ms)' : (activeServer || 'Не выбран')
					)
				])
			])
		]);
		viewRoot.appendChild(modeCard);

		// Import Subscription Box
		var importCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'style': 'font-weight: 700; font-size: 15px; margin-bottom: 10px; color: #fff;' }, '📥 Импорт подписок и ссылок (VLESS / Hy2 / TUIC / SS / Clash YAML / Base64)'),
			E('textarea', {
				'class': 'os-textarea',
				'id': 'os-subscription-input',
				'rows': 3,
				'placeholder': 'Вставьте ссылку на подписку (https://...), Base64 код или ссылки vless://, hysteria2://, tuic://, ss://, trojan://'
			}),
			E('div', { 'style': 'display: flex; justify-content: flex-end; margin-top: 10px;' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var txt = document.getElementById('os-subscription-input');
						var val = txt ? txt.value.trim() : '';
						if (!val) {
							ui.addNotification(null, E('p', {}, 'Поле ввода ссылки или подписки пусто.'), 'warning');
							return;
						}
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = 'Загрузка и парсинг...';
						callImportSubscription(val).then(function(res) {
							btn.disabled = false;
							btn.innerText = '📥 Импортировать узлы';
							if (res && res.success) {
								ui.addNotification(null, E('p', {}, '✓ ' + (res.message || 'Узлы успешно импортированы!')), 'info');
								if (txt) txt.value = '';
								callGetServers().then(function(sRes) {
									if (sRes && sRes.servers) {
										servers = sRes.servers;
										renderServerCards();
									}
								});
							} else {
								ui.addNotification(null, E('p', {}, '❌ Ошибка импорта: ' + (res.error || 'Неизвестная ошибка')), 'error');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '📥 Импортировать узлы';
							ui.addNotification(null, E('p', {}, '❌ Ошибка импорта: ' + err), 'error');
						});
					}
				}, [ '📥 Импортировать узлы' ])
			])
		]);
		viewRoot.appendChild(importCard);

		// Servers Grid Container (Cards layout, strictly Mobile First - Rule #8)
		var serversContainer = E('div', { 'class': 'os-server-grid', 'id': 'os-server-grid-box' });
		viewRoot.appendChild(serversContainer);

		function renderServerCards() {
			dom.content(serversContainer, null);

			if (!servers.length) {
				serversContainer.appendChild(E('div', { 'class': 'os-card', 'style': 'grid-column: 1/-1; text-align: center; color: var(--os-muted); padding: 40px;' }, [
					'Нет добавленных серверов. Вставьте ссылку на подписку выше для импорта узлов.'
				]));
				return;
			}

			servers.forEach(function(s, idx) {
				var protoBadge = 'os-badge-blue';
				if (s.protocol === 'vless') protoBadge = 'os-badge-purple';
				if (s.protocol === 'hysteria2') protoBadge = 'os-badge-cyan';
				if (s.protocol === 'shadowsocks') protoBadge = 'os-badge-amber';

				var latencyColor = 'var(--os-emerald)';
				var latencyText = (s.latency_ms ? s.latency_ms + ' ms' : 'Не проверен');
				if (s.latency_ms > 120) latencyColor = 'var(--os-rose)';
				else if (s.latency_ms > 70) latencyColor = 'var(--os-amber)';

				var card = E('div', { 'class': 'os-server-card' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('div', { 'style': 'display: flex; align-items: center; gap: 8px;' }, [
							E('span', { 'style': 'font-size: 20px;' }, s.country_flag || '🌐'),
							E('span', { 'style': 'font-weight: 700; font-size: 15px; color: #fff;' }, s.name || s.server)
						]),
						E('span', { 'class': 'os-badge ' + protoBadge }, s.protocol)
					]),
					E('div', { 'style': 'font-size: 13px; color: var(--os-muted); display: flex; justify-content: space-between;' }, [
						E('span', {}, s.server + ':' + s.port),
						E('span', { 'style': 'font-weight: 700; color: ' + latencyColor }, [ '⚡ ' + latencyText ])
					]),
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-top: 6px; padding-top: 10px; border-top: 1px solid var(--os-border);' }, [
						E('button', {
							'class': 'os-btn os-btn-dark',
							'style': 'padding: 5px 12px; font-size: 12px;',
							'click': function(ev) {
								var b = ev.target;
								b.innerText = '⏳';
								callTestLatency(s.server, s.port).then(function(res) {
									b.innerText = '⚡ Тест';
									if (res && res.latency_ms) {
										s.latency_ms = res.latency_ms;
										renderServerCards();
									}
								});
							}
						}, [ '⚡ Тест' ]),
						E('button', {
							'class': 'os-btn os-btn-dark',
							'style': 'padding: 5px 10px; color: var(--os-rose);',
							'title': 'Удалить сервер',
							'click': function() {
								if (confirm('Удалить сервер ' + (s.name || s.server) + '?')) {
									servers.splice(idx, 1);
									renderServerCards();
								}
							}
						}, [ '🗑️' ])
					])
				]);

				serversContainer.appendChild(card);
			});
		}

		renderServerCards();
		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
