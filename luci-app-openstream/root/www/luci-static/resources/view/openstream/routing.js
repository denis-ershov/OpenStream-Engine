'use strict';
'require view';
'require rpc';
'require ui';
'require dom';

var callRoutingGet = rpc.declare({
	object: 'openstream',
	method: 'get_routing',
	expect: { '': {} }
});

var callRoutingSave = rpc.declare({
	object: 'openstream',
	method: 'save_routing',
	params: [ 'rules' ],
	expect: { '': {} }
});

var callRoutingTest = rpc.declare({
	object: 'openstream',
	method: 'test_route',
	params: [ 'domain' ],
	expect: { '': {} }
});

return view.extend({
	load: function() {
		return callRoutingGet();
	},

	render: function(data) {
		var state = {
			rules: (data && data.rules) ? data.rules : [],
			clients: (data && data.clients) ? data.clients : [],
			zapret2_installed: (data && data.zapret2_installed) || false,
			singbox_installed: (data && data.singbox_installed) || false
		};

		var viewRoot = E('div', { 'class': 'os-modern-container' });

		// Inject modern CSS tokens and styles
		var styleNode = E('style', {}, `
			:root {
				--os-bg: #020617;
				--os-card: #0f172a;
				--os-card-hover: #1e293b;
				--os-border: #334155;
				--os-text: #f8fafc;
				--os-muted: #94a3b8;
				--os-emerald: #10b981;
				--os-blue: #38bdf8;
				--os-amber: #f59e0b;
				--os-rose: #f43f5e;
				--os-purple: #a855f7;
				--os-radius: 12px;
			}
			.os-modern-container {
				font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Inter, sans-serif;
				color: var(--os-text);
				max-width: 1280px;
				margin: 0 auto;
				padding: 10px 0;
			}
			.os-header-card {
				display: flex;
				flex-wrap: wrap;
				align-items: center;
				justify-content: space-between;
				gap: 16px;
				margin-bottom: 24px;
				padding: 24px;
				background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 100%);
				border: 1px solid var(--os-border);
				border-radius: var(--os-radius);
				box-shadow: 0 10px 25px -5px rgba(0,0,0,0.4);
			}
			.os-badge {
				display: inline-flex;
				align-items: center;
				font-size: 11px;
				font-weight: 700;
				padding: 3px 8px;
				border-radius: 9999px;
				text-transform: uppercase;
				letter-spacing: 0.05em;
			}
			.os-badge-emerald { background: rgba(16,185,129,0.2); color: var(--os-emerald); border: 1px solid rgba(16,185,129,0.4); }
			.os-badge-blue { background: rgba(56,189,248,0.2); color: var(--os-blue); border: 1px solid rgba(56,189,248,0.4); }
			.os-badge-purple { background: rgba(168,85,247,0.2); color: var(--os-purple); border: 1px solid rgba(168,85,247,0.4); }
			.os-badge-rose { background: rgba(244,63,94,0.2); color: var(--os-rose); border: 1px solid rgba(244,63,94,0.4); }
			.os-badge-amber { background: rgba(245,158,11,0.2); color: var(--os-amber); border: 1px solid rgba(245,158,11,0.4); }
			.os-badge-cyan { background: rgba(6, 182, 212, 0.15); color: #06b6d4; border: 1px solid rgba(6, 182, 212, 0.3); }

			.os-pills {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
				gap: 12px;
				margin-bottom: 24px;
			}
			.os-pill {
				background: var(--os-card);
				border: 1px solid var(--os-border);
				border-radius: var(--os-radius);
				padding: 16px 20px;
				display: flex;
				align-items: center;
				justify-content: space-between;
			}
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

			.os-card-head {
				display: flex;
				flex-wrap: wrap;
				align-items: center;
				justify-content: space-between;
				gap: 12px;
				margin-bottom: 14px;
			}
			.os-card-grid {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
				gap: 16px;
				padding-top: 14px;
				border-top: 1px solid #1e293b;
			}
			.os-field {
				display: flex;
				flex-direction: column;
				gap: 6px;
			}
			.os-label {
				font-size: 12px;
				font-weight: 600;
				color: var(--os-muted);
				text-transform: uppercase;
				letter-spacing: 0.04em;
			}
			.os-input, .os-select {
				background: #020617;
				border: 1px solid var(--os-border);
				border-radius: 8px;
				padding: 9px 12px;
				color: #fff;
				font-size: 13px;
				outline: none;
			}
			.os-input:focus, .os-select:focus { border-color: var(--os-blue); }

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

		// 1. Header Banner
		var headerNode = E('div', { 'class': 'os-header-card' }, [
			E('div', {}, [
				E('h2', { 'style': 'margin: 0 0 6px 0; font-size: 24px; font-weight: 700; color: #fff; display: flex; align-items: center; gap: 10px;' }, [
					'🛡️ OpenStream Policy Routing',
					E('span', { 'class': 'os-badge os-badge-blue' }, 'ucode Engine 2.1')
				]),
				E('div', { 'style': 'font-size: 13px; color: var(--os-muted);' },
					'Интеллектуальная оркестрация сетевых движков: Zapret2 (nfqws2), StreamProxy, VPN и DNS-маршрутизация на ucode'
				)
			]),
			E('div', { 'style': 'display: flex; gap: 10px;' }, [
				E('button', {
					'class': 'os-btn os-btn-dark',
					'click': function() {
						var name = prompt('Введите название нового правила:', 'Custom Service');
						if (!name) return;
						var newRule = {
							file: 'rule_' + Date.now() + '.osrule.yaml',
							id: 'org.openstream.rules.custom_' + Date.now(),
							name: name,
							enabled: true,
							description: 'Пользовательское правило маршрутизации',
							action: 'zapret2',
							preset: 'youtube_4k',
							raw_yaml: 'schema_version: "2.1"\nid: "org.openstream.rules.custom_' + Date.now() + '"\nname: "' + name + '"\nversion: "1.0.0"\nmatches:\n  - domains: ["example.com"]\n    action:\n      zapret2:\n        preset: "youtube_4k"\n'
						};
						state.rules.push(newRule);
						renderCards();
					}
				}, [ '➕ Добавить правило' ]),
				E('button', {
					'class': 'os-btn os-btn-success',
					'id': 'os-save-btn',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ Применение в ucode...';

						callRoutingSave(state.rules).then(function(res) {
							btn.disabled = false;
							btn.innerText = '💾 Сохранить и применить';
							if (res && res.success) {
								ui.addNotification(null, E('p', {}, '✓ ' + (res.message || 'Маршруты успешно обновлены')), 'success');
							} else {
								ui.addNotification(null, E('p', {}, '❌ Ошибка: ' + (res && res.error ? res.error : 'Неизвестный сбой')), 'error');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 Сохранить и применить';
							ui.addNotification(null, E('p', {}, '❌ Ошибка RPC: ' + err), 'error');
						});
					}
				}, [ '💾 Сохранить и применить' ])
			])
		]);
		viewRoot.appendChild(headerNode);

		// 2. Status Pills
		var pillsNode = E('div', { 'class': 'os-pills' }, [
			E('div', { 'class': 'os-pill' }, [
				E('div', {}, [
					E('div', { 'class': 'os-label' }, 'Анти-DPI (Zapret2 / nfqws2)'),
					E('div', { 'style': 'font-weight: 700; font-size: 14px; margin-top: 4px; color: ' + (state.zapret2_installed ? 'var(--os-emerald)' : 'var(--os-amber)') },
						state.zapret2_installed ? '✓ Установлен и готов' : '⚠️ Не найден (/usr/bin/nfqws2)'
					)
				]),
				E('span', { 'class': 'os-badge os-badge-purple' }, 'NFQUEUE 1088')
			]),
			E('div', { 'class': 'os-pill' }, [
				E('div', {}, [
					E('div', { 'class': 'os-label' }, 'DNS-Интеграция'),
					E('div', { 'style': 'font-weight: 700; font-size: 14px; margin-top: 4px; color: var(--os-emerald);' },
						'✓ dnsmasq nftset (без перехвата 53)'
					)
				]),
				E('span', { 'class': 'os-badge os-badge-emerald' }, 'Штатный')
			]),
			E('div', { 'class': 'os-pill' }, [
				E('div', {}, [
					E('div', { 'class': 'os-label' }, 'Клиенты LAN'),
					E('div', { 'style': 'font-weight: 700; font-size: 14px; margin-top: 4px;' },
						state.clients.length + ' устройств в DHCP'
					)
				]),
				E('span', { 'class': 'os-badge os-badge-blue' }, 'Per-Device #95')
			])
		]);
		viewRoot.appendChild(pillsNode);

		// 3. Real-Time Route Tester Widget (Issues #75, #76)
		var testBox = E('div', { 'style': 'display: none; padding: 12px; background: #020617; border-radius: 8px; margin-top: 10px;' });
		var testerNode = E('div', { 'class': 'os-card' }, [
			E('div', { 'style': 'font-weight: 700; font-size: 14px; margin-bottom: 10px; display: flex; align-items: center; gap: 8px;' }, [
				'🔍 Инспектор маршрутизации в реальном времени (без перезагрузки)'
			]),
			E('div', { 'style': 'display: flex; gap: 10px; flex-wrap: wrap;' }, [
				E('input', {
					'type': 'text',
					'id': 'os-test-input',
					'class': 'os-input',
					'style': 'flex: 1; min-width: 240px;',
					'placeholder': 'Введите домен (например: googlevideo.com, discord.gg, twitch.tv)'
				}),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function() {
						var domInput = document.getElementById('os-test-input');
						var domVal = domInput ? domInput.value.trim() : '';
						if (!domVal) return;

						testBox.style.display = 'block';
						testBox.innerHTML = '<span style="color: var(--os-muted);">Запрос маршрута к ucode RPC...</span>';

						callRoutingTest(domVal).then(function(res) {
							if (res && res.success && res.result) {
								var r = res.result;
								testBox.innerHTML = `
									<div style="font-weight: 700; color: #fff; margin-bottom: 4px;">Результат: ${domVal}</div>
									<div style="display: flex; gap: 8px; align-items: center; margin-bottom: 4px;">
										<span>Маршрут:</span>
										<span class="os-badge os-badge-blue">${r.engine}</span>
									</div>
									<div style="color: var(--os-muted); font-size: 12px;">${r.details}</div>
								`;
							} else {
								testBox.innerHTML = '<span style="color: var(--os-rose);">Маршрут не определен.</span>';
							}
						}).catch(function(err) {
							testBox.innerHTML = '<span style="color: var(--os-rose);">Ошибка ubus: ' + err + '</span>';
						});
					}
				}, [ 'Проверить маршрут' ])
			]),
			testBox
		]);
		viewRoot.appendChild(testerNode);

		// 4. Cards Container (No Tables - Rules #10, #11)
		var cardsContainer = E('div', { 'id': 'os-cards-container' });
		viewRoot.appendChild(cardsContainer);

		function renderCards() {
			dom.content(cardsContainer, null);

			state.rules.forEach(function(rule, idx) {
				var badgeClass = 'os-badge-amber';
				var badgeText = '➡️ Direct (WAN)';
				if (rule.action === 'bypass') {
					badgeClass = 'os-badge-cyan';
					badgeText = '⏩ Bypass / Исключение';
				} else if (rule.action === 'zapret2') {
					badgeClass = 'os-badge-purple';
					badgeText = '🚀 Zapret2 (Anti-DPI)';
				} else if (rule.action === 'streamproxy') {
					badgeClass = 'os-badge-emerald';
					badgeText = '🛡️ StreamProxy';
				} else if (rule.action === 'vpn') {
					badgeClass = 'os-badge-blue';
					badgeText = '🌐 VPN Tunnel';
				} else if (rule.action === 'block') {
					badgeClass = 'os-badge-rose';
					badgeText = '⛔ Block (0.0.0.0)';
				}

				var cardNode = E('div', { 'class': 'os-card' }, [
					E('div', { 'class': 'os-card-head' }, [
						E('div', { 'style': 'display: flex; align-items: center; gap: 10px;' }, [
							E('div', { 'style': 'width: 28px; height: 28px; border-radius: 6px; background: #1e293b; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 700; color: #94a3b8;' }, [ String(idx + 1) ]),
							E('div', {}, [
								E('div', { 'style': 'font-size: 16px; font-weight: 700; color: #fff;' }, [ rule.name ]),
								E('div', { 'style': 'font-size: 12px; color: var(--os-muted);' }, [ rule.description || rule.id ])
							]),
							E('span', { 'class': 'os-badge ' + badgeClass }, [ badgeText ])
						]),
						E('div', { 'style': 'display: flex; align-items: center; gap: 8px;' }, [
							E('label', { 'class': 'os-switch', 'title': 'Включить / Отключить' }, [
								E('input', {
									'type': 'checkbox',
									'checked': rule.enabled,
									'change': function(ev) { rule.enabled = ev.target.checked; }
								}),
								E('span', { 'class': 'os-slider' })
							]),
							E('button', {
								'class': 'os-btn os-btn-dark',
								'style': 'padding: 4px 10px;',
								'disabled': idx === 0,
								'title': 'Поднять приоритет',
								'click': function() {
									var tmp = state.rules[idx];
									state.rules[idx] = state.rules[idx - 1];
									state.rules[idx - 1] = tmp;
									renderCards();
								}
							}, [ '▲' ]),
							E('button', {
								'class': 'os-btn os-btn-dark',
								'style': 'padding: 4px 10px;',
								'disabled': idx === state.rules.length - 1,
								'title': 'Понизить приоритет',
								'click': function() {
									var tmp = state.rules[idx];
									state.rules[idx] = state.rules[idx + 1];
									state.rules[idx + 1] = tmp;
									renderCards();
								}
							}, [ '▼' ]),
							E('button', {
								'class': 'os-btn os-btn-dark',
								'style': 'padding: 4px 10px; color: var(--os-rose);',
								'title': 'Удалить',
								'click': function() {
									if (confirm('Удалить правило ' + rule.name + '?')) {
										state.rules.splice(idx, 1);
										renderCards();
									}
								}
							}, [ '🗑️' ])
						])
					]),

					E('div', { 'class': 'os-card-grid' }, [
						E('div', { 'class': 'os-field' }, [
							E('label', { 'class': 'os-label' }, 'Сетевой движок (Action)'),
							E('select', {
								'class': 'os-select',
								'change': function(ev) {
									rule.action = ev.target.value;
									if (rule.action === 'zapret2' && !rule.preset) rule.preset = 'youtube_4k';
									renderCards();
								}
							}, [
								E('option', { 'value': 'bypass', 'selected': rule.action === 'bypass' }, '⏩ Пропустить / Bypass (Исключение в WAN)'),
								E('option', { 'value': 'zapret2', 'selected': rule.action === 'zapret2' }, '🚀 Zapret2 (nfqws2 Anti-DPI)'),
								E('option', { 'value': 'streamproxy', 'selected': rule.action === 'streamproxy' }, '🛡️ StreamProxy (Twitch/AdBlock)'),
								E('option', { 'value': 'vpn', 'selected': rule.action === 'vpn' }, '🌐 VPN / sing-box Gateway'),
								E('option', { 'value': 'direct', 'selected': rule.action === 'direct' }, '➡️ Direct / WAN'),
								E('option', { 'value': 'block', 'selected': rule.action === 'block' }, '⛔ Блокировка (0.0.0.0)')
							])
						]),

						E('div', { 'class': 'os-field' }, [
							E('label', { 'class': 'os-label' }, 'Клиенты LAN (Per-Device #95)'),
							E('select', { 'class': 'os-select' }, [
								E('option', { 'value': 'all' }, 'Все устройства локальной сети'),
								...state.clients.map(function(c) {
									return E('option', { 'value': c.mac }, c.hostname + ' (' + c.ip + ' / ' + c.mac + ')');
								})
							])
						]),

						(rule.action === 'zapret2') ? E('div', { 'class': 'os-field' }, [
							E('label', { 'class': 'os-label' }, 'Пресет десинхронизации Zapret2'),
							E('select', {
								'class': 'os-select',
								'change': function(ev) {
									rule.preset = ev.target.value;
									renderCards();
								}
							}, [
								E('option', { 'value': 'youtube_4k', 'selected': rule.preset === 'youtube_4k' }, 'YouTube 4K (split2 badseq)'),
								E('option', { 'value': 'discord_voice', 'selected': rule.preset === 'discord_voice' }, 'Discord Voice (UDP fake 50000:65535)'),
								E('option', { 'value': 'general_multisplit', 'selected': rule.preset === 'general_multisplit' }, 'General (multisplit midsld)'),
								E('option', { 'value': 'custom', 'selected': rule.preset === 'custom' }, '⚙️ Пользовательские аргументы (Custom Flags)')
							]),
							(rule.preset === 'custom') ? E('input', {
								'type': 'text',
								'class': 'os-input',
								'style': 'margin-top: 6px; font-family: monospace;',
								'placeholder': '--dpi-desync=fake,split2 --dpi-desync-split-pos=3',
								'value': rule.custom_args || '',
								'input': function(ev) { rule.custom_args = ev.target.value; }
							}) : E('div', {})
						]) : E('div', {})
					])
				]);

				cardsContainer.appendChild(cardNode);
			});
		}

		renderCards();
		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
