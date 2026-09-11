'use strict';
'require view';
'require rpc';
'require ui';
'require dom';
'require poll';

var callMonitorFlows = rpc.declare({
	object: 'openstream',
	method: 'get_monitor_flows',
	expect: { '': {} }
});

var callTestRoute = rpc.declare({
	object: 'openstream',
	method: 'test_route',
	params: [ 'domain' ],
	expect: { '': {} }
});

var callClearFlows = rpc.declare({
	object: 'openstream',
	method: 'clear_monitor_flows',
	expect: { '': {} }
});

return view.extend({
	filterSection: 'all',
	filterClient: 'all',
	pollInterval: 3,

	load: function() {
		return callMonitorFlows();
	},

	formatBytes: function(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		var k = 1024;
		var sizes = ['B', 'KB', 'MB', 'GB'];
		var i = Math.floor(Math.log(bytes) / Math.log(k));
		return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
	},

	render: function(data) {
		var self = this;
		var flows = (data && data.flows) ? data.flows : [];

		var viewRoot = E('div', { 'style': 'font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Inter, sans-serif; color: #f8fafc; max-width: 1400px; margin: 0 auto;' }, [
			E('style', {}, `
				.os-card {
					background: #0f172a;
					border: 1px solid #1e293b;
					border-radius: 12px;
					padding: 22px;
					margin-bottom: 20px;
					box-shadow: 0 10px 25px -5px rgba(0,0,0,0.4);
				}
				.os-metric-grid {
					display: grid;
					grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
					gap: 14px;
					margin-bottom: 22px;
				}
				.os-metric-box {
					background: #020617;
					border: 1px solid #1e293b;
					border-radius: 10px;
					padding: 16px;
					display: flex;
					flex-direction: column;
					justify-content: space-between;
				}
				.os-metric-val {
					font-size: 24px;
					font-weight: 800;
					margin-top: 6px;
				}
				.os-chip-group {
					display: flex;
					flex-wrap: wrap;
					gap: 8px;
					margin-top: 14px;
				}
				.os-chip {
					padding: 6px 14px;
					border-radius: 9999px;
					font-size: 13px;
					font-weight: 600;
					cursor: pointer;
					border: 1px solid transparent;
					transition: all 0.2s ease;
				}
				.os-chip-active {
					background: #38bdf8;
					color: #020617;
				}
				.os-chip-inactive {
					background: #1e293b;
					color: #94a3b8;
				}
				.os-chip-inactive:hover {
					background: #334155;
					color: #fff;
				}
				.os-flow-card {
					background: #020617;
					border: 1px solid #1e293b;
					border-radius: 10px;
					padding: 18px;
					margin-bottom: 12px;
					display: flex;
					justify-content: space-between;
					align-items: center;
					flex-wrap: wrap;
					gap: 14px;
					transition: border-color 0.2s, transform 0.2s;
				}
				.os-flow-card:hover {
					border-color: #38bdf8;
					transform: translateY(-2px);
				}
				.os-live-pulse {
					display: inline-block;
					width: 8px;
					height: 8px;
					border-radius: 50%;
					margin-right: 6px;
					background: #10b981;
					box-shadow: 0 0 10px #10b981;
				}
				.os-inspector-input {
					background: #020617;
					border: 1px solid #334155;
					color: #fff;
					padding: 10px 14px;
					border-radius: 8px;
					font-size: 14px;
					flex: 1;
					min-width: 240px;
				}
				.os-inspector-input:focus {
					border-color: #38bdf8;
					outline: none;
				}
			`),

			// 1. Верхняя панель управления
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 14px;' }, [
					E('div', {}, [
						E('h2', { 'style': 'margin: 0 0 4px 0; font-size: 22px; font-weight: 700; color: #fff;' }, '📡 Мониторинг маршрутизации и потоков (Live)'),
						E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'Отслеживание активных соединений, клиентов LAN и секций ядра в реальном времени')
					]),
					E('div', { 'style': 'display: flex; gap: 10px; align-items: center; flex-wrap: wrap;' }, [
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'background: #1e293b; color: #fff; border: 1px solid #334155;',
							'click': function() {
								callClearFlows().then(function() {
									ui.addNotification(null, E('p', {}, 'История мониторинга очищена.'), 'info');
									window.location.reload();
								});
							}
						}, 'Очистить историю'),
						E('button', {
							'class': 'btn cbi-button-positive',
							'style': 'background: #10b981; color: #020617; font-weight: 700;',
							'click': function() {
								window.location.reload();
							}
						}, '🔄 Обновить сейчас')
					])
				]),

				// Метрики
				E('div', { 'class': 'os-metric-grid', 'style': 'margin-top: 22px;' }, [
					E('div', { 'class': 'os-metric-box' }, [
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 700;' }, 'Всего потоков'),
						E('div', { 'class': 'os-metric-val', 'style': 'color: #38bdf8;' }, String(flows.length))
					]),
					E('div', { 'class': 'os-metric-box' }, [
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 700;' }, 'Zapret2 (nfqws2)'),
						E('div', { 'class': 'os-metric-val', 'style': 'color: #10b981;' },
							String(flows.filter(function(f) { return f.section === 'zapret2'; }).length)
						)
					]),
					E('div', { 'class': 'os-metric-box' }, [
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 700;' }, 'StreamProxy (:8888)'),
						E('div', { 'class': 'os-metric-val', 'style': 'color: #38bdf8;' },
							String(flows.filter(function(f) { return f.section === 'streamproxy'; }).length)
						)
					]),
					E('div', { 'class': 'os-metric-box' }, [
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 700;' }, 'sing-box / VPN'),
						E('div', { 'class': 'os-metric-val', 'style': 'color: #a855f7;' },
							String(flows.filter(function(f) { return f.section === 'singbox' || f.section === 'vpn'; }).length)
						)
					]),
					E('div', { 'class': 'os-metric-box' }, [
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; text-transform: uppercase; font-weight: 700;' }, 'DNS Sinkhole (Block)'),
						E('div', { 'class': 'os-metric-val', 'style': 'color: #f43f5e;' },
							String(flows.filter(function(f) { return f.section === 'block'; }).length)
						)
					])
				])
			]),

			// 2. Инспектор маршрутов в реальном времени
			E('div', { 'class': 'os-card' }, [
				E('h3', { 'style': 'margin: 0 0 10px 0; font-size: 17px; font-weight: 700; color: #fff;' }, '🔍 Живой инспектор маршрутов (Live Domain Inspector)'),
				E('div', { 'style': 'font-size: 13px; color: #94a3b8; margin-bottom: 14px;' }, 'Мгновенная проверка, через какую секцию ядра пойдет трафик для целевого домена:'),
				E('div', { 'style': 'display: flex; gap: 10px; flex-wrap: wrap;' }, [
					E('input', {
						'id': 'os-inspector-input',
						'class': 'os-inspector-input',
						'type': 'text',
						'placeholder': 'Например: googlevideo.com, discord.gg, twitch.tv, bank.ru'
					}),
					E('button', {
						'class': 'btn cbi-button-action',
						'style': 'background: #38bdf8; color: #020617; font-weight: 700;',
						'click': function() {
							var input = document.getElementById('os-inspector-input');
							var resBox = document.getElementById('os-inspector-result');
							if (!input || !input.value) return;

							callTestRoute(input.value.trim()).then(function(res) {
								if (res && res.result) {
									var r = res.result;
									var color = '#94a3b8';
									if (r.route === 'zapret2') color = '#10b981';
									else if (r.route === 'streamproxy') color = '#38bdf8';
									else if (r.route === 'vpn') color = '#a855f7';
									else if (r.route === 'block') color = '#f43f5e';

									resBox.style.display = 'block';
									resBox.innerHTML = `
										<div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 10px;">
											<div>
												<span style="font-size: 16px; font-weight: 800; color: #fff;">${r.domain}</span>
												<div style="font-size: 13px; color: #94a3b8; margin-top: 4px;">${r.details}</div>
											</div>
											<span style="padding: 6px 14px; border-radius: 9999px; font-weight: 700; font-size: 13px; background: ${color}20; color: ${color}; border: 1px solid ${color}40;">
												● ${r.engine}
											</span>
										</div>
									`;
								}
							});
						}
					}, 'Проверить маршрут')
				]),
				E('div', {
					'id': 'os-inspector-result',
					'style': 'display: none; margin-top: 14px; padding: 14px; background: #020617; border: 1px solid #1e293b; border-radius: 8px;'
				})
			]),

			// 3. Список активных потоков (Mobile First: Карточки, без HTML-таблиц!)
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;' }, [
					E('h3', { 'style': 'margin: 0; font-size: 17px; font-weight: 700; color: #fff;' }, '⚡ Активные потоки и сопоставления секций'),
					E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'Отображение правил и активных сетевых клиентов')
				]),

				// Фильтры-чипы
				E('div', { 'class': 'os-chip-group' }, [
					E('span', {
						'class': 'os-chip ' + (self.filterSection === 'all' ? 'os-chip-active' : 'os-chip-inactive'),
						'click': function() { self.filterSection = 'all'; self.renderFlowList(flows); }
					}, 'Все секции'),
					E('span', {
						'class': 'os-chip ' + (self.filterSection === 'zapret2' ? 'os-chip-active' : 'os-chip-inactive'),
						'click': function() { self.filterSection = 'zapret2'; self.renderFlowList(flows); }
					}, '⚡ Zapret2 (nfqws2)'),
					E('span', {
						'class': 'os-chip ' + (self.filterSection === 'streamproxy' ? 'os-chip-active' : 'os-chip-inactive'),
						'click': function() { self.filterSection = 'streamproxy'; self.renderFlowList(flows); }
					}, '🛡️ StreamProxy (:8888)'),
					E('span', {
						'class': 'os-chip ' + (self.filterSection === 'singbox' ? 'os-chip-active' : 'os-chip-inactive'),
						'click': function() { self.filterSection = 'singbox'; self.renderFlowList(flows); }
					}, '🌐 sing-box / VPN'),
					E('span', {
						'class': 'os-chip ' + (self.filterSection === 'block' ? 'os-chip-active' : 'os-chip-inactive'),
						'click': function() { self.filterSection = 'block'; self.renderFlowList(flows); }
					}, '🚫 DNS Block')
				]),

				// Контейнер карточек потоков
				E('div', { 'id': 'os-flows-container', 'style': 'margin-top: 18px;' })
			])
		]);

		setTimeout(function() {
			self.renderFlowList(flows);
		}, 20);

		return viewRoot;
	},

	renderFlowList: function(flows) {
		var container = document.getElementById('os-flows-container');
		if (!container) return;

		var filtered = flows;
		if (this.filterSection !== 'all') {
			filtered = flows.filter(function(f) {
				if (this.filterSection === 'singbox') {
					return f.section === 'singbox' || f.section === 'vpn';
				}
				return f.section === this.filterSection;
			}.bind(this));
		}

		dom.content(container, null);

		if (filtered.length === 0) {
			container.appendChild(E('div', {
				'style': 'text-align: center; padding: 40px 20px; color: #94a3b8; background: #020617; border-radius: 10px;'
			}, [
				E('div', { 'style': 'font-size: 28px; margin-bottom: 8px;' }, '🔍'),
				E('div', { 'style': 'font-weight: 600; color: #e2e8f0;' }, 'Потоков по выбранному фильтру не обнаружено'),
				E('div', { 'style': 'font-size: 13px; margin-top: 4px;' }, 'Трафик для этой категории сейчас направляется через стандартный провайдерский шлюз WAN.')
			]));
			return;
		}

		filtered.forEach(function(flow) {
			// Маршрут из конфигурации; счётчики пакетов не выдумываются.
			var card = E('div', { 'class': 'os-flow-card' }, [
				E('div', { 'style': 'display: flex; align-items: center; gap: 14px; min-width: 240px;' }, [
					E('div', {}, [
						E('div', { 'style': 'font-size: 16px; font-weight: 700; color: #fff;' }, [
							E('span', { 'class': 'os-live-pulse', 'style': 'background: ' + flow.badge_color + '; box-shadow: 0 0 8px ' + flow.badge_color + ';' }),
							flow.domain
						]),
						E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 3px;' },
							'Применяется ко всем клиентам LAN')
					])
				]),

				E('div', { 'style': 'display: flex; align-items: center; gap: 14px; flex-wrap: wrap;' }, [
					E('span', {
						'style': 'padding: 6px 14px; border-radius: 9999px; font-weight: 700; font-size: 12px; background: ' +
							flow.badge_color + '20; color: ' + flow.badge_color + '; border: 1px solid ' + flow.badge_color + '40;'
					}, flow.details)
				])
			]);

			container.appendChild(card);
		});
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
