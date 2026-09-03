'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Self-Diagnostics View
 * Mobile-First, OLED Dark Card Layout, No HTML Tables (Rules #8, #10, #11)
 */

var callRunDiagnostics = rpc.declare({
	object: 'openstream',
	method: 'run_diagnostics',
	expect: { success: true }
});

return view.extend({
	load: function() {
		return Promise.all([
			callRunDiagnostics().catch(function() {
				return { checks: [], timestamp: 0 };
			})
		]);
	},

	render: function(results) {
		var initial = results[0] || {};
		var checks = initial.checks || [];

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
			.os-badge {
				display: inline-flex;
				align-items: center;
				font-size: 11px;
				font-weight: 700;
				padding: 4px 10px;
				border-radius: 6px;
				text-transform: uppercase;
				letter-spacing: 0.05em;
			}
			.os-badge-pass { background: rgba(16,185,129,0.2); color: var(--os-emerald); border: 1px solid rgba(16,185,129,0.4); }
			.os-badge-warn { background: rgba(245,158,11,0.2); color: var(--os-amber); border: 1px solid rgba(245,158,11,0.4); }
			.os-badge-fail { background: rgba(244,63,94,0.2); color: var(--os-rose); border: 1px solid rgba(244,63,94,0.4); }
			.os-diag-grid {
				display: grid;
				grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
				gap: 16px;
				margin-top: 16px;
			}
			.os-diag-card {
				background: #020617;
				border: 1px solid var(--os-border);
				border-radius: 10px;
				padding: 16px;
				display: flex;
				flex-direction: column;
				gap: 8px;
			}
		`);
		viewRoot.appendChild(styleNode);

		// Header
		var headerNode = E('div', { 'class': 'os-card' }, [
			E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;' }, [
				E('div', {}, [
					E('h2', { 'style': 'margin: 0 0 6px 0; font-size: 22px; font-weight: 800; color: #fff;' }, '🩺 Самодиагностика и проверка здоровья системы'),
					E('p', { 'style': 'margin: 0; color: var(--os-muted); font-size: 13px;' },
						'Автоматическая проверка целостности правил nftables, сокетов sing-box, очередей Zapret2 и отсутствия утечек DNS.'
					)
				]),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ Проверка компонентов...';
						callRunDiagnostics().then(function(res) {
							btn.disabled = false;
							btn.innerText = '▶️ Запустить самодиагностику';
							if (res && res.checks) {
								checks = res.checks;
								renderDiagCards();
								ui.addNotification(null, E('p', {}, '✓ Диагностика системы успешно завершена!'), 'info');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '▶️ Запустить самодиагностику';
							ui.addNotification(null, E('p', {}, '❌ Ошибка RPC: ' + err), 'error');
						});
					}
				}, [ '▶️ Запустить самодиагностику' ])
			])
		]);
		viewRoot.appendChild(headerNode);

		var diagContainer = E('div', { 'class': 'os-diag-grid' });
		viewRoot.appendChild(diagContainer);

		function renderDiagCards() {
			dom.content(diagContainer, null);

			if (!checks.length) {
				diagContainer.appendChild(E('div', { 'class': 'os-card', 'style': 'grid-column: 1/-1; text-align: center; color: var(--os-muted); padding: 40px;' }, [
					'Нажмите «Запустить самодиагностику» для опроса компонентов роутера.'
				]));
				return;
			}

			checks.forEach(function(c) {
				var badgeClass = 'os-badge-pass';
				var badgeText = '✓ В НОРМЕ';
				if (c.status === 'warn') {
					badgeClass = 'os-badge-warn';
					badgeText = '⚠️ ВНИМАНИЕ';
				} else if (c.status === 'fail') {
					badgeClass = 'os-badge-fail';
					badgeText = '❌ ОШИБКА';
				}

				var card = E('div', { 'class': 'os-diag-card' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('div', { 'style': 'font-weight: 700; font-size: 15px; color: #fff;' }, c.name),
						E('span', { 'class': 'os-badge ' + badgeClass }, badgeText)
					]),
					E('div', { 'style': 'font-size: 13px; color: var(--os-muted); line-height: 1.4;' }, c.details)
				]);

				diagContainer.appendChild(card);
			});
		}

		renderDiagCards();
		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
