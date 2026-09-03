'use strict';
'require view';
'require rpc';
'require ui';
'require dom';

var callCheckUpdates = rpc.declare({
	object: 'openstream',
	method: 'check_updates',
	expect: { '': {} }
});

var callPerformUpdate = rpc.declare({
	object: 'openstream',
	method: 'perform_update',
	params: [ 'component' ],
	expect: { '': {} }
});

var callGetUpdateLog = rpc.declare({
	object: 'openstream',
	method: 'get_update_log',
	expect: { '': {} }
});

var callGetSingboxInfo = rpc.declare({
	object: 'openstream',
	method: 'get_singbox_info',
	expect: { '': {} }
});

var callSwitchSingboxVariant = rpc.declare({
	object: 'openstream',
	method: 'switch_singbox_variant',
	params: [ 'variant' ],
	expect: { '': {} }
});

var callGetAutoUpdateConfig = rpc.declare({
	object: 'openstream',
	method: 'get_auto_update_config',
	expect: { '': {} }
});

var callSaveAutoUpdateConfig = rpc.declare({
	object: 'openstream',
	method: 'save_auto_update_config',
	params: [ 'auto_update_enabled', 'interval', 'update_lists', 'update_singbox', 'update_zapret2', 'update_core' ],
	expect: { success: true }
});

return view.extend({
	load: function() {
		return Promise.all([
			callCheckUpdates(),
			callGetSingboxInfo(),
			callGetUpdateLog(),
			callGetAutoUpdateConfig().catch(function() { return {}; })
		]);
	},

	render: function(data) {
		var updateData = data[0] || {};
		var singboxInfo = data[1] || {};
		var logData = data[2] || {};
		var autoCfg = data[3] || {};

		var components = updateData.components || [];

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
				.os-hero-btn {
					background: linear-gradient(135deg, #10b981 0%, #059669 100%);
					color: #020617;
					font-weight: 800;
					font-size: 16px;
					padding: 14px 28px;
					border-radius: 10px;
					border: none;
					cursor: pointer;
					display: inline-flex;
					align-items: center;
					gap: 10px;
					box-shadow: 0 4px 14px 0 rgba(16, 185, 129, 0.39);
					transition: all 0.2s ease;
				}
				.os-hero-btn:hover {
					transform: translateY(-2px);
					box-shadow: 0 6px 20px rgba(16, 185, 129, 0.5);
				}
				.os-comp-grid {
					display: grid;
					grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
					gap: 16px;
					margin-top: 18px;
				}
				.os-comp-card {
					background: #020617;
					border: 1px solid #1e293b;
					border-radius: 10px;
					padding: 18px;
					display: flex;
					flex-direction: column;
					justify-content: space-between;
					transition: border-color 0.2s;
				}
				.os-comp-card:hover {
					border-color: #38bdf8;
				}
				.os-badge-ok {
					background: rgba(16, 185, 129, 0.15);
					color: #10b981;
					border: 1px solid rgba(16, 185, 129, 0.3);
					padding: 4px 10px;
					border-radius: 9999px;
					font-size: 12px;
					font-weight: 700;
				}
				.os-badge-upd {
					background: rgba(245, 158, 11, 0.15);
					color: #f59e0b;
					border: 1px solid rgba(245, 158, 11, 0.3);
					padding: 4px 10px;
					border-radius: 9999px;
					font-size: 12px;
					font-weight: 700;
				}
				.os-log-terminal {
					background: #020617;
					border: 1px solid #1e293b;
					border-radius: 8px;
					padding: 14px;
					font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
					font-size: 13px;
					color: #38bdf8;
					white-space: pre-wrap;
					height: 220px;
					overflow-y: auto;
				}
			`),

			// 1. Главная панель обновлений (Hero Card)
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;' }, [
					E('div', {}, [
						E('h2', { 'style': 'margin: 0 0 6px 0; font-size: 24px; font-weight: 800; color: #fff;' }, '🚀 Менеджер компонентов и обновлений в 1 клик'),
						E('div', { 'style': 'font-size: 14px; color: #94a3b8;' }, 'Централизованная проверка и бесшовное обновление ядра OSE, sing-box, Zapret2 и каталога правил.')
					]),
					E('div', { 'style': 'display: flex; gap: 12px; flex-wrap: wrap;' }, [
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'background: #1e293b; color: #fff; font-weight: 600; padding: 12px 20px; border-radius: 8px;',
							'click': function() {
								ui.showModal('Проверка обновлений', [
									E('p', { 'class': 'spinning' }, 'Опрос удаленного репозитория GitHub и индекса OpenStream...')
								]);
								callCheckUpdates().then(function() {
									ui.hideModal();
									ui.addNotification(null, E('p', {}, 'Проверка завершена. Все списки версий актуализированы.'), 'info');
									window.location.reload();
								});
							}
						}, '🔍 Проверить обновления'),
						E('button', {
							'class': 'os-hero-btn',
							'click': function() {
								ui.showModal('Обновление компонентов в 1 клик', [
									E('p', { 'class': 'spinning' }, 'Выполняется безопасное обновление всех компонентов с проверкой SHA256...')
								]);
								callPerformUpdate('all').then(function() {
									setTimeout(function() {
										ui.hideModal();
										ui.addNotification(null, E('p', {}, 'Все компоненты успешно обновлены!'), 'info');
										window.location.reload();
									}, 1500);
								});
							}
						}, '⚡ Обновить всё в 1 клик')
					])
				])
			]),

			// 2. Выбор варианта sing-box (Stable / Extended / Tiny / Extended Compress)
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;' }, [
					E('div', {}, [
						E('h3', { 'style': 'margin: 0 0 4px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '🌐 Интеграция sing-box: Выбор сборки (Flavors)'),
						E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'Выберите оптимальную редакцию под аппаратные характеристики вашего роутера:')
					]),
					E('span', {
						'class': singboxInfo.installed ? 'os-badge-ok' : 'os-badge-upd'
					}, singboxInfo.installed ? 'Текущая: ' + (singboxInfo.version || 'v1.11.x') : 'Не установлен')
				]),

				E('div', { 'style': 'margin-top: 16px; display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 14px;' }, [
					// Вариант 1: Stable
					E('div', { 'class': 'os-comp-card', 'style': (singboxInfo.configured_variant === 'stable' ? 'border-color: #38bdf8;' : '') }, [
						E('div', {}, [
							E('div', { 'style': 'font-weight: 700; font-size: 15px; color: #fff; display: flex; justify-content: space-between;' }, [
								'sing-box Stable',
								singboxInfo.configured_variant === 'stable' ? E('span', { 'class': 'os-badge-ok' }, 'Активен') : null
							]),
							E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 6px; line-height: 1.4;' },
								'Официальная стабильная сборка OpenWrt feeds. Базовый набор проверенных протоколов (Shadowsocks, VLESS, WireGuard).'
							),
							E('div', { 'style': 'font-size: 11px; color: #cbd5e1; margin-top: 8px;' }, 'Flash: ~18 МБ • RAM: Рекомендуется 256+ МБ')
						]),
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'margin-top: 14px; background: #1e293b; color: #38bdf8; border: 1px solid #334155; font-weight: 600;',
							'disabled': singboxInfo.configured_variant === 'stable',
							'click': function() {
								callSwitchSingboxVariant('stable').then(function() {
									ui.addNotification(null, E('p', {}, 'Вариант sing-box переключен на Stable'), 'info');
									window.location.reload();
								});
							}
						}, singboxInfo.configured_variant === 'stable' ? '✓ Выбран' : 'Переключить на Stable')
					]),

					// Вариант 2: Extended
					E('div', { 'class': 'os-comp-card', 'style': (singboxInfo.configured_variant === 'extended' ? 'border-color: #a855f7;' : '') }, [
						E('div', {}, [
							E('div', { 'style': 'font-weight: 700; font-size: 15px; color: #fff; display: flex; justify-content: space-between;' }, [
								'sing-box Extended (xHTTP)',
								singboxInfo.configured_variant === 'extended' ? E('span', { 'class': 'os-badge-ok', 'style': 'background: rgba(168,85,247,0.2); color: #a855f7; border-color: #a855f7;' }, 'Активен') : null
							]),
							E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 6px; line-height: 1.4;' },
								'Полная расширенная сборка: xHTTP транспорт, Reality, TUIC v5, Shadowsocks 2022 и новейшие шифры обхода блокировок.'
							),
							E('div', { 'style': 'font-size: 11px; color: #cbd5e1; margin-top: 8px;' }, 'Flash: ~28 МБ • RAM: Рекомендуется 256+ МБ')
						]),
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'margin-top: 14px; background: #1e293b; color: #a855f7; border: 1px solid #334155; font-weight: 600;',
							'disabled': singboxInfo.configured_variant === 'extended',
							'click': function() {
								callSwitchSingboxVariant('extended').then(function() {
									ui.addNotification(null, E('p', {}, 'Вариант sing-box переключен на Extended (xHTTP)'), 'info');
									window.location.reload();
								});
							}
						}, singboxInfo.configured_variant === 'extended' ? '✓ Выбран' : 'Переключить на Extended')
					]),

					// Вариант 3: Tiny
					E('div', { 'class': 'os-comp-card', 'style': (singboxInfo.configured_variant === 'tiny' ? 'border-color: #10b981;' : '') }, [
						E('div', {}, [
							E('div', { 'style': 'font-weight: 700; font-size: 15px; color: #fff; display: flex; justify-content: space-between;' }, [
								'sing-box Tiny (Легковес)',
								singboxInfo.configured_variant === 'tiny' ? E('span', { 'class': 'os-badge-ok' }, 'Активен') : null
							]),
							E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 6px; line-height: 1.4;' },
								'Минималистичный бинарник без тяжелых библиотек. Идеально для слабых роутеров с 64–128 МБ RAM (потребление <15 МБ).'
							),
							E('div', { 'style': 'font-size: 11px; color: #cbd5e1; margin-top: 8px;' }, 'Flash: <8 МБ • RAM: 64–128 МБ')
						]),
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'margin-top: 14px; background: #1e293b; color: #10b981; border: 1px solid #334155; font-weight: 600;',
							'disabled': singboxInfo.configured_variant === 'tiny',
							'click': function() {
								callSwitchSingboxVariant('tiny').then(function() {
									ui.addNotification(null, E('p', {}, 'Вариант sing-box переключен на Tiny'), 'info');
									window.location.reload();
								});
							}
						}, singboxInfo.configured_variant === 'tiny' ? '✓ Выбран' : 'Переключить на Tiny')
					]),

					// Вариант 4: Extended Compress (UPX)
					E('div', { 'class': 'os-comp-card', 'style': (singboxInfo.configured_variant === 'extended_compress' ? 'border-color: #f59e0b;' : '') }, [
						E('div', {}, [
							E('div', { 'style': 'font-weight: 700; font-size: 15px; color: #fff; display: flex; justify-content: space-between;' }, [
								'sing-box Extended Compress',
								singboxInfo.configured_variant === 'extended_compress' ? E('span', { 'class': 'os-badge-ok', 'style': 'background: rgba(245,158,11,0.2); color: #f59e0b; border-color: #f59e0b;' }, 'Активен') : null
							]),
							E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 6px; line-height: 1.4;' },
								'UPX-сжатый вариант Extended. Полноценная поддержка всех новейших протоколов с экономией 65% дискового пространства.'
							),
							E('div', { 'style': 'font-size: 11px; color: #cbd5e1; margin-top: 8px;' }, 'Flash: ~11 МБ (Сжато) • RAM: 128+ МБ')
						]),
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'margin-top: 14px; background: #1e293b; color: #f59e0b; border: 1px solid #334155; font-weight: 600;',
							'disabled': singboxInfo.configured_variant === 'extended_compress',
							'click': function() {
								callSwitchSingboxVariant('extended_compress').then(function() {
									ui.addNotification(null, E('p', {}, 'Вариант sing-box переключен на Extended Compress'), 'info');
									window.location.reload();
								});
							}
						}, singboxInfo.configured_variant === 'extended_compress' ? '✓ Выбран' : 'Переключить на Compress')
					])
				])
			]),

			// 3. Автообновление по расписанию (Cron / Safe Fallback)
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px; margin-bottom: 12px;' }, [
					E('div', {}, [
						E('h3', { 'style': 'margin: 0 0 4px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '⏰ Автоматическое обновление по расписанию (Cron)'),
						E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'Фоновая синхронизация списков и модулей с проверкой SHA-256 и Safe Fallback')
					]),
					E('button', {
						'class': 'btn cbi-button-action',
						'style': 'background: #10b981; color: #020617; font-weight: 700;',
						'click': function(ev) {
							var btn = ev.target;
							btn.disabled = true;
							btn.innerText = 'Сохранение...';

							var isEnabled = document.getElementById('os-auto-enabled') ? document.getElementById('os-auto-enabled').checked : false;
							var interval = document.getElementById('os-auto-interval') ? document.getElementById('os-auto-interval').value : 'daily';
							var upLists = document.getElementById('os-up-lists') ? document.getElementById('os-up-lists').checked : true;
							var upSb = document.getElementById('os-up-sb') ? document.getElementById('os-up-sb').checked : false;
							var upZ2 = document.getElementById('os-up-z2') ? document.getElementById('os-up-z2').checked : false;
							var upCore = document.getElementById('os-up-core') ? document.getElementById('os-up-core').checked : false;

							callSaveAutoUpdateConfig(isEnabled, interval, upLists, upSb, upZ2, upCore).then(function() {
								btn.disabled = false;
								btn.innerText = '💾 Сохранить расписание cron';
								ui.addNotification(null, E('p', {}, '✓ Расписание автоматического обновления сохранено!'), 'info');
							}).catch(function(err) {
								btn.disabled = false;
								btn.innerText = '💾 Сохранить расписание cron';
								ui.addNotification(null, E('p', {}, '❌ Ошибка сохранения cron: ' + err), 'error');
							});
						}
					}, [ '💾 Сохранить расписание cron' ])
				]),
				E('div', { 'style': 'display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; margin-top: 14px;' }, [
					E('div', { 'style': 'background: #020617; border: 1px solid #1e293b; border-radius: 8px; padding: 14px;' }, [
						E('label', { 'style': 'display: flex; align-items: center; gap: 10px; cursor: pointer;' }, [
							E('input', {
								'type': 'checkbox',
								'id': 'os-auto-enabled',
								'checked': autoCfg.auto_update_enabled === true
							}),
							E('span', { 'style': 'font-weight: 700; color: #fff;' }, 'Включить автообновление')
						]),
						E('div', { 'style': 'font-size: 12px; color: #64748b; margin-top: 6px;' }, 'Запускает задание через системный cron роутера (/etc/crontabs/root)')
					]),
					E('div', { 'style': 'background: #020617; border: 1px solid #1e293b; border-radius: 8px; padding: 14px;' }, [
						E('label', { 'style': 'display: block; font-size: 12px; font-weight: 600; color: #64748b; margin-bottom: 6px;' }, 'ПЕРИОДИЧНОСТЬ'),
						E('select', {
							'class': 'cbi-input-select',
							'id': 'os-auto-interval',
							'style': 'width: 100%; background: #0f172a; color: #fff; border: 1px solid #334155; padding: 6px 10px; border-radius: 6px;'
						}, [
							E('option', { 'value': 'daily', 'selected': (autoCfg.interval || 'daily') === 'daily' }, 'Ежедневно (в 04:00)'),
							E('option', { 'value': '3days', 'selected': autoCfg.interval === '3days' }, 'Каждые 3 дня'),
							E('option', { 'value': 'weekly', 'selected': autoCfg.interval === 'weekly' }, 'Раз в неделю (Воскресенье)')
						])
					]),
					E('div', { 'style': 'background: #020617; border: 1px solid #1e293b; border-radius: 8px; padding: 14px;' }, [
						E('div', { 'style': 'font-size: 12px; font-weight: 600; color: #64748b; margin-bottom: 6px;' }, 'ОБНОВЛЯЕМЫЕ КОМПОНЕНТЫ'),
						E('label', { 'style': 'display: block; font-size: 13px; margin-bottom: 4px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-lists', 'checked': autoCfg.update_lists !== false }),
							' Списки правил и доменов'
						]),
						E('label', { 'style': 'display: block; font-size: 13px; margin-bottom: 4px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-sb', 'checked': autoCfg.update_singbox === true }),
							' sing-box Core'
						]),
						E('label', { 'style': 'display: block; font-size: 13px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-z2', 'checked': autoCfg.update_zapret2 === true }),
							' Zapret2 (nfqws2)'
						])
					])
				])
			]),

			// 4. Статус компонентов OSE и сторонних демонов (Карточки Mobile First)
			E('div', { 'class': 'os-card' }, [
				E('h3', { 'style': 'margin: 0 0 4px 0; font-size: 18px; font-weight: 700; color: #fff;' }, '📦 Статус компонентов системы'),
				E('div', { 'style': 'font-size: 13px; color: #94a3b8;' }, 'Точечное обновление модулей и просмотр установленных версий'),

				E('div', { 'class': 'os-comp-grid' }, components.map(function(comp) {
					return E('div', { 'class': 'os-comp-card' }, [
						E('div', {}, [
							E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
								E('span', { 'style': 'font-size: 15px; font-weight: 700; color: #fff;' }, comp.name),
								E('span', {
									'class': comp.update_available ? 'os-badge-upd' : 'os-badge-ok'
								}, comp.update_available ? 'Доступно: ' + comp.latest_version : 'Актуален')
							]),
							E('div', { 'style': 'font-size: 12px; color: #94a3b8; margin-top: 6px;' }, comp.description),
							E('div', { 'style': 'font-size: 12px; color: #cbd5e1; margin-top: 10px;' }, [
								'Установлена версия: ',
								E('strong', { 'style': 'color: #fff;' }, comp.installed_version)
							])
						]),
						E('button', {
							'class': 'btn cbi-button-action',
							'style': 'margin-top: 14px; background: #1e293b; color: #38bdf8; border: 1px solid #334155;',
							'click': function() {
								ui.showModal('Обновление компонента', [
									E('p', { 'class': 'spinning' }, 'Обновление ' + comp.name + '...')
								]);
								callPerformUpdate(comp.id).then(function() {
									setTimeout(function() {
										ui.hideModal();
										ui.addNotification(null, E('p', {}, comp.name + ' успешно обновлен.'), 'info');
										window.location.reload();
									}, 1000);
								});
							}
						}, comp.update_available ? 'Обновить до ' + comp.latest_version : 'Переустановить / Проверить')
					]);
				}))
			]),

			// 4. Терминал логов обновления
			E('div', { 'class': 'os-card' }, [
				E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;' }, [
					E('h3', { 'style': 'margin: 0; font-size: 17px; font-weight: 700; color: #fff;' }, '🖥️ Консоль процесса обновления (/tmp/openstream_update.log)'),
					E('button', {
						'class': 'btn cbi-button-action',
						'style': 'background: #1e293b; color: #94a3b8; font-size: 12px;',
						'click': function() {
							callGetUpdateLog().then(function(res) {
								var logEl = document.getElementById('os-update-log');
								if (logEl && res && res.log) {
									logEl.textContent = res.log;
								}
							});
						}
					}, 'Обновить лог')
				]),
				E('div', { 'id': 'os-update-log', 'class': 'os-log-terminal' }, logData.log || 'Лог обновлений пуст.')
			])
		]);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
