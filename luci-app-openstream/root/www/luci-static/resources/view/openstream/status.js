'use strict';
'require view';
'require rpc';
'require ui';
'require dom';

var callStatus = rpc.declare({
	object: 'openstream',
	method: 'status',
	expect: { '': {} }
});

return view.extend({
	load: function() {
		return callStatus();
	},

	render: function(data) {
		var isRunning = data && data.running;
		var zapret2Installed = data && data.zapret2_installed;
		var zapret2Running = data && data.zapret2_running;

		var viewRoot = E('div', { 'style': 'font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Inter, sans-serif; color: #f8fafc; max-width: 1280px; margin: 0 auto;' }, [
			E('style', {}, `
				.os-status-card {
					background: #0f172a;
					border: 1px solid #334155;
					border-radius: 12px;
					padding: 24px;
					margin-bottom: 20px;
					box-shadow: 0 10px 25px -5px rgba(0,0,0,0.4);
				}
				.os-grid {
					display: grid;
					grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
					gap: 16px;
					margin-top: 20px;
				}
				.os-stat-box {
					background: #020617;
					border: 1px solid #1e293b;
					border-radius: 10px;
					padding: 18px;
				}
				.os-stat-title {
					font-size: 12px;
					color: #94a3b8;
					text-transform: uppercase;
					font-weight: 600;
					letter-spacing: 0.05em;
				}
				.os-stat-value {
					font-size: 18px;
					font-weight: 700;
					margin-top: 6px;
				}
			`),

			E('div', { 'class': 'os-status-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;' }, [
					E('div', {}, [
						E('h2', { 'style': 'margin: 0 0 4px 0; font-size: 22px; font-weight: 700; color: #fff;' }, '⚡ OpenStream Engine 2.1 Status'),
						E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'ucode High-Speed RPC & Orchestration Architecture')
					]),
					E('span', {
						'style': 'padding: 6px 14px; border-radius: 9999px; font-weight: 700; font-size: 13px; background: ' +
							(isRunning ? 'rgba(16,185,129,0.2); color: #10b981; border: 1px solid rgba(16,185,129,0.4);' : 'rgba(244,63,94,0.2); color: #f43f5e; border: 1px solid rgba(244,63,94,0.4);')
					}, isRunning ? '● СЛУЖБА АКТИВНА' : '○ ОСТАНОВЛЕНО')
				]),

				E('div', { 'class': 'os-grid' }, [
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-title' }, 'Движок ядра (Rust daemon)'),
						E('div', { 'class': 'os-stat-value', 'style': 'color: ' + (isRunning ? '#10b981' : '#f43f5e') },
							isRunning ? 'Работает (streamproxyd)' : 'Остановлен'
						)
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-title' }, 'Анти-DPI (Zapret2 nfqws2)'),
						E('div', { 'class': 'os-stat-value', 'style': 'color: ' + (zapret2Running ? '#10b981' : (zapret2Installed ? '#38bdf8' : '#f59e0b')) },
							zapret2Running ? 'Активен (Очередь 1088)' : (zapret2Installed ? 'Установлен (Готов)' : 'Не установлен')
						)
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-title' }, 'Рантайм и протокол'),
						E('div', { 'class': 'os-stat-value', 'style': 'color: #38bdf8;' }, 'ucode 1.0 / C API')
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-title' }, 'Расход памяти ядра'),
						E('div', { 'class': 'os-stat-value', 'style': 'color: #10b981;' }, '< 2.5 МБ RAM (Zero-Copy)')
					])
				])
			])
		]);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
