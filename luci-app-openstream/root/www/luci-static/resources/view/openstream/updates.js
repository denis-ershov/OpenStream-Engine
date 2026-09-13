'use strict';
'require view';
'require rpc';
'require ui';
'require dom';

/*
 * OpenStream Engine 2.1 - Component Updates & sing-box Flavor Manager
 * Unified Linear/Apple Design System, Mobile-First, 100% i18n
 */

function ensureStylesheet() {
	var id = 'openstream-css';
	if (document.getElementById(id)) return;
	var link = E('link', {
		'id': id,
		'rel': 'stylesheet',
		'type': 'text/css',
		'href': (window.L && L.resource) ? L.resource('openstream/openstream.css') : '/luci-static/resources/openstream/openstream.css'
	});
	document.head.appendChild(link);
}

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
			callCheckUpdates().catch(function() { return {}; }),
			callGetSingboxInfo().catch(function() { return {}; }),
			callGetUpdateLog().catch(function() { return {}; }),
			callGetAutoUpdateConfig().catch(function() { return {}; })
		]);
	},

	render: function(data) {
		ensureStylesheet();

		var updateData = data[0] || {};
		var singboxInfo = data[1] || {};
		var logData = data[2] || {};
		var autoCfg = data[3] || {};

		var components = updateData.components || [];

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '🚀 ', _('Component & Update Center') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Manage OSE core services, sing-box flavor variants, Zapret2 DPI rules, and automated cron syncing.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-secondary',
					'click': function() {
						ui.showModal(_('Checking for Updates'), [
							E('p', { 'class': 'spinning' }, _('Querying remote repositories and package index...'))
						]);
						callCheckUpdates().then(function() {
							ui.hideModal();
							ui.addNotification(null, E('p', {}, '✓ ' + _('Update check complete. Versions refreshed.')), 'info');
							window.location.reload();
						});
					}
				}, [ '🔍 ', _('Check for Updates') ]),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function() {
						ui.showModal(_('Updating Components'), [
							E('p', { 'class': 'spinning' }, _('Submitting update request...'))
						]);
						callPerformUpdate('all').then(function(res) {
							ui.hideModal();
							if (res && res.not_implemented) {
								ui.addNotification(null, E('p', {},
									_('Automated updates are disabled in this binary release. Run "opkg update && opkg upgrade" in terminal.')), 'warning');
							} else if (res && res.success) {
								ui.addNotification(null, E('p', {}, '✓ ' + _('All components updated successfully.')), 'info');
								window.location.reload();
							} else {
								ui.addNotification(null, E('p', {},
									'❌ ' + _('Update failed: ') + ((res && res.error) || _('Unknown error'))), 'error');
							}
						}).catch(function(err) {
							ui.hideModal();
							ui.addNotification(null, E('p', {}, '❌ ' + _('Update error: ') + err), 'error');
						});
					}
				}, [ '⚡ ', _('Update All (1-Click)') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// sing-box Flavors Card
		var curVariant = singboxInfo.configured_variant || 'stable';
		var sbCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🌐 ', _('sing-box Binary Flavor Selector') ]),
				E('span', {
					'class': 'os-badge ' + (singboxInfo.installed ? 'os-badge-success' : 'os-badge-warning')
				}, singboxInfo.installed ? _('Installed: ') + (singboxInfo.version || 'v1.11.x') : _('Not Installed'))
			]),
			E('p', { 'class': 'os-help', 'style': 'margin-bottom: 16px;' },
				_('Choose the sing-box build tailored to your router hardware constraints (RAM, flash storage, and protocol features):')
			),
			E('div', { 'class': 'os-grid' }, [
				// Flavor 1: Stable
				E('div', {
					'class': 'os-card',
					'style': 'margin-bottom: 0; display: flex; flex-direction: column; justify-content: space-between;' + (curVariant === 'stable' ? ' border-color: var(--os-primary);' : '')
				}, [
					E('div', {}, [
						E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;' }, [
							E('span', { 'class': 'os-label' }, 'sing-box Stable'),
							curVariant === 'stable' ? E('span', { 'class': 'os-badge os-badge-success' }, _('Active')) : null
						]),
						E('div', { 'class': 'os-help', 'style': 'line-height: 1.4;' },
							_('Official standard feed build. Rock-solid Shadowsocks, VLESS, WireGuard and DNS routing.')
						),
						E('div', { 'style': 'font-size: 11px; color: var(--os-text-muted); margin-top: 8px;' },
							'Flash: ~18 MB • RAM: 256+ MB'
						)
					]),
					E('button', {
						'class': 'os-btn os-btn-sm ' + (curVariant === 'stable' ? 'os-btn-secondary' : 'os-btn-primary'),
						'style': 'margin-top: 14px;',
						'disabled': curVariant === 'stable',
						'click': function() {
							callSwitchSingboxVariant('stable').then(function() {
								ui.addNotification(null, E('p', {}, '✓ ' + _('Switched to Stable variant.')), 'info');
								window.location.reload();
							});
						}
					}, curVariant === 'stable' ? '✓ ' + _('Selected') : _('Switch to Stable'))
				]),

				// Flavor 2: Extended
				E('div', {
					'class': 'os-card',
					'style': 'margin-bottom: 0; display: flex; flex-direction: column; justify-content: space-between;' + (curVariant === 'extended' ? ' border-color: var(--os-purple);' : '')
				}, [
					E('div', {}, [
						E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;' }, [
							E('span', { 'class': 'os-label' }, 'sing-box Extended (xHTTP)'),
							curVariant === 'extended' ? E('span', { 'class': 'os-badge os-badge-purple' }, _('Active')) : null
						]),
						E('div', { 'class': 'os-help', 'style': 'line-height: 1.4;' },
							_('Full protocol suite: VLESS Reality, xHTTP transport, TUIC v5, Shadowsocks 2022 and state-of-the-art anti-censorship.')
						),
						E('div', { 'style': 'font-size: 11px; color: var(--os-text-muted); margin-top: 8px;' },
							'Flash: ~28 MB • RAM: 256+ MB'
						)
					]),
					E('button', {
						'class': 'os-btn os-btn-sm ' + (curVariant === 'extended' ? 'os-btn-secondary' : 'os-btn-primary'),
						'style': 'margin-top: 14px;',
						'disabled': curVariant === 'extended',
						'click': function() {
							callSwitchSingboxVariant('extended').then(function() {
								ui.addNotification(null, E('p', {}, '✓ ' + _('Switched to Extended variant.')), 'info');
								window.location.reload();
							});
						}
					}, curVariant === 'extended' ? '✓ ' + _('Selected') : _('Switch to Extended'))
				]),

				// Flavor 3: Tiny
				E('div', {
					'class': 'os-card',
					'style': 'margin-bottom: 0; display: flex; flex-direction: column; justify-content: space-between;' + (curVariant === 'tiny' ? ' border-color: var(--os-success);' : '')
				}, [
					E('div', {}, [
						E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;' }, [
							E('span', { 'class': 'os-label' }, 'sing-box Tiny (Lightweight)'),
							curVariant === 'tiny' ? E('span', { 'class': 'os-badge os-badge-success' }, _('Active')) : null
						]),
						E('div', { 'class': 'os-help', 'style': 'line-height: 1.4;' },
							_('Stripped minimal build (<15 MB RAM usage). Optimized for entry-level routers with 64–128 MB RAM.')
						),
						E('div', { 'style': 'font-size: 11px; color: var(--os-text-muted); margin-top: 8px;' },
							'Flash: <8 MB • RAM: 64–128 MB'
						)
					]),
					E('button', {
						'class': 'os-btn os-btn-sm ' + (curVariant === 'tiny' ? 'os-btn-secondary' : 'os-btn-primary'),
						'style': 'margin-top: 14px;',
						'disabled': curVariant === 'tiny',
						'click': function() {
							callSwitchSingboxVariant('tiny').then(function() {
								ui.addNotification(null, E('p', {}, '✓ ' + _('Switched to Tiny variant.')), 'info');
								window.location.reload();
							});
						}
					}, curVariant === 'tiny' ? '✓ ' + _('Selected') : _('Switch to Tiny'))
				]),

				// Flavor 4: Extended Compress
				E('div', {
					'class': 'os-card',
					'style': 'margin-bottom: 0; display: flex; flex-direction: column; justify-content: space-between;' + (curVariant === 'extended_compress' ? ' border-color: var(--os-warning);' : '')
				}, [
					E('div', {}, [
						E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;' }, [
							E('span', { 'class': 'os-label' }, 'sing-box Compressed (UPX)'),
							curVariant === 'extended_compress' ? E('span', { 'class': 'os-badge os-badge-warning' }, _('Active')) : null
						]),
						E('div', { 'class': 'os-help', 'style': 'line-height: 1.4;' },
							_('UPX-compressed Extended build. Provides all modern protocol features while saving 65% flash storage.')
						),
						E('div', { 'style': 'font-size: 11px; color: var(--os-text-muted); margin-top: 8px;' },
							'Flash: ~11 MB • RAM: 128+ MB'
						)
					]),
					E('button', {
						'class': 'os-btn os-btn-sm ' + (curVariant === 'extended_compress' ? 'os-btn-secondary' : 'os-btn-primary'),
						'style': 'margin-top: 14px;',
						'disabled': curVariant === 'extended_compress',
						'click': function() {
							callSwitchSingboxVariant('extended_compress').then(function() {
								ui.addNotification(null, E('p', {}, '✓ ' + _('Switched to Compressed variant.')), 'info');
								window.location.reload();
							});
						}
					}, curVariant === 'extended_compress' ? '✓ ' + _('Selected') : _('Switch to Compressed'))
				])
			])
		]);
		viewRoot.appendChild(sbCard);

		// Scheduled Auto-Updates (Cron)
		var cronCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '⏰ ', _('Automated Sync & Cron Schedule') ]),
				E('button', {
					'class': 'os-btn os-btn-primary os-btn-sm',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = _('Saving...');

						var isEnabled = document.getElementById('os-auto-enabled') ? document.getElementById('os-auto-enabled').checked : false;
						var interval = document.getElementById('os-auto-interval') ? document.getElementById('os-auto-interval').value : 'daily';
						var upLists = document.getElementById('os-up-lists') ? document.getElementById('os-up-lists').checked : true;
						var upSb = document.getElementById('os-up-sb') ? document.getElementById('os-up-sb').checked : false;
						var upZ2 = document.getElementById('os-up-z2') ? document.getElementById('os-up-z2').checked : false;
						var upCore = document.getElementById('os-up-core') ? document.getElementById('os-up-core').checked : false;

						callSaveAutoUpdateConfig(isEnabled, interval, upLists, upSb, upZ2, upCore).then(function() {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save Schedule');
							ui.addNotification(null, E('p', {}, '✓ ' + _('Automated update schedule saved successfully!')), 'info');
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save Schedule');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Failed to save schedule: ') + err), 'error');
						});
					}
				}, [ '💾 ', _('Save Schedule') ])
			]),
			E('div', { 'class': 'os-grid' }, [
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('Enable Automated Cron')),
						E('label', { 'class': 'os-switch' }, [
							E('input', {
								'type': 'checkbox',
								'id': 'os-auto-enabled',
								'checked': autoCfg.auto_update_enabled === true
							}),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Executes periodic syncing in the background (/etc/crontabs/root).'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Sync Interval')),
					E('select', {
						'class': 'os-select',
						'id': 'os-auto-interval'
					}, [
						E('option', { 'value': 'daily', 'selected': (autoCfg.interval || 'daily') === 'daily' }, _('Daily (at 04:00 AM)')),
						E('option', { 'value': '3days', 'selected': autoCfg.interval === '3days' }, _('Every 3 Days')),
						E('option', { 'value': 'weekly', 'selected': autoCfg.interval === 'weekly' }, _('Weekly (Sunday at 04:00 AM)'))
					]),
					E('div', { 'class': 'os-help' }, _('Low-traffic hours recommended for transparent updates.'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Synchronized Modules')),
					E('div', { 'style': 'display: flex; flex-direction: column; gap: 6px; margin-top: 4px;' }, [
						E('label', { 'style': 'display: flex; align-items: center; gap: 8px; font-size: 13px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-lists', 'checked': autoCfg.update_lists !== false }),
							_('Community Domain & GeoIP Lists')
						]),
						E('label', { 'style': 'display: flex; align-items: center; gap: 8px; font-size: 13px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-sb', 'checked': autoCfg.update_singbox === true }),
							_('sing-box Core Binary')
						]),
						E('label', { 'style': 'display: align-items: center; gap: 8px; font-size: 13px;' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-up-z2', 'checked': autoCfg.update_zapret2 === true }),
							_('Zapret2 (nfqws2) Binary & Strategy')
						]),
						E('input', { 'type': 'hidden', 'id': 'os-up-core', 'value': '0' })
					])
				])
			])
		]);
		viewRoot.appendChild(cronCard);

		// Component Status Grid Card
		var compCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '📦 ', _('Installed Subsystem Status') ]),
				E('span', { 'class': 'os-badge os-badge-info' }, components.length + ' ' + _('packages'))
			]),
			E('div', { 'class': 'os-grid' }, components.map(function(comp) {
				return E('div', { 'class': 'os-entity-row', 'style': 'flex-direction: column; align-items: stretch; margin-bottom: 0;' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;' }, [
						E('span', { 'class': 'os-entity-name' }, comp.name),
						E('span', {
							'class': 'os-badge ' + (comp.update_available ? 'os-badge-warning' : 'os-badge-success')
						}, comp.update_available ? _('Update: ') + comp.latest_version : _('Up to Date'))
					]),
					E('div', { 'class': 'os-entity-desc', 'style': 'margin-bottom: 8px;' }, comp.description),
					E('div', { 'style': 'font-size: 11px; color: var(--os-text-secondary); margin-bottom: 10px;' }, [
						_('Installed Version: '),
						E('strong', { 'style': 'color: var(--os-text-primary);' }, comp.installed_version || '–')
					]),
					E('button', {
						'class': 'os-btn os-btn-sm ' + (comp.update_available ? 'os-btn-primary' : 'os-btn-secondary'),
						'click': function() {
							ui.showModal(_('Updating Component'), [
								E('p', { 'class': 'spinning' }, _('Submitting update request for ') + comp.name + '...')
							]);
							callPerformUpdate(comp.id).then(function(res) {
								ui.hideModal();
								if (res && res.not_implemented) {
									ui.addNotification(null, E('p', {},
										_('Automated updates disabled. Upgrade via terminal: opkg update && opkg upgrade ') + comp.id), 'warning');
								} else if (res && res.success) {
									ui.addNotification(null, E('p', {}, comp.name + ': ' + _('Update completed.')), 'info');
									window.location.reload();
								} else {
									ui.addNotification(null, E('p', {},
										comp.name + ': ' + ((res && res.error) || _('Update failed'))), 'error');
								}
							}).catch(function(err) {
								ui.hideModal();
								ui.addNotification(null, E('p', {}, '❌ ' + _('Error: ') + err), 'error');
							});
						}
					}, comp.update_available ? _('Upgrade to ') + comp.latest_version : _('Recheck'))
				]);
			}))
		]);
		viewRoot.appendChild(compCard);

		// Terminal Log Card
		var logCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🖥️ ', _('Update Execution Console') ]),
				E('button', {
					'class': 'os-btn os-btn-secondary os-btn-sm',
					'click': function() {
						callGetUpdateLog().then(function(res) {
							var el = document.getElementById('os-update-log');
							if (el && res && res.log) {
								el.textContent = res.log;
							}
						});
					}
				}, [ '🔄 ', _('Refresh Console') ])
			]),
			E('div', {
				'id': 'os-update-log',
				'class': 'os-console'
			}, logData.log || _('No update activity logged.'))
		]);
		viewRoot.appendChild(logCard);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
