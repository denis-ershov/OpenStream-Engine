'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Servers & Subscriptions View (sing-box 4-flavor)
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
	params: [ 'input', 'user_agent', 'hwid' ],
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
		ensureStylesheet();

		var initial = results[0] || {};
		var servers = initial.servers || [];
		var selectorMode = initial.selector_mode || 'urltest';
		var activeServer = initial.active_server || 'auto';

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '🌐 ', _('Outbound Servers & Subscriptions') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Management of sing-box outbounds: VLESS Reality, Hysteria 2, TUIC, Shadowsocks 2022, Trojan and auto-failover.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-secondary',
					'id': 'os-btn-test-ping',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ ' + _('Measuring latency...');
						callTestLatency('', 443).then(function(res) {
							btn.disabled = false;
							btn.innerText = '⚡ ' + _('Check Ping');
							if (res && res.servers) {
								servers = res.servers;
								renderServerCards();
								ui.addNotification(null, E('p', {}, '✓ ' + _('Server latency test completed!')), 'info');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '⚡ ' + _('Check Ping');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Ping test failed: ') + err), 'error');
						});
					}
				}, [ '⚡ ', _('Check Ping') ]),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = _('Saving...');
						callSaveServers(servers, selectorMode, activeServer).then(function() {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save & Apply');
							ui.addNotification(null, E('p', {}, '✓ ' + _('Server list and outbounds successfully applied!')), 'info');
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save & Apply');
							ui.addNotification(null, E('p', {}, '❌ ' + _('RPC Error: ') + err), 'error');
						});
					}
				}, [ '💾 ', _('Save & Apply') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// Selector Mode Card
		var activeServerSelect;
		var modeCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '⚡ ', _('Outbound Gateway Selector') ])
			]),
			E('div', { 'class': 'os-grid-2' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Selector Mode')),
					E('select', {
						'class': 'os-select',
						'change': function(ev) {
							selectorMode = ev.target.value;
							updateSelectorVisibility();
							renderServerCards();
						}
					}, [
						E('option', { 'value': 'urltest', 'selected': selectorMode === 'urltest' }, '⚡ ' + _('URLTest (Auto-best latency)')),
						E('option', { 'value': 'manual', 'selected': selectorMode === 'manual' }, '📌 ' + _('Manual Selection'))
					]),
					E('div', { 'class': 'os-help' }, _('URLTest automatically chooses the lowest-latency outbound; Manual locks to a specific server.'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Active Outbound Node')),
					activeServerSelect = E('select', {
						'class': 'os-select',
						'disabled': selectorMode === 'urltest',
						'change': function(ev) {
							activeServer = ev.target.value;
							renderServerCards();
						}
					}),
					E('div', { 'class': 'os-help', 'id': 'os-active-help' },
						selectorMode === 'urltest' ? _('Automatically determined based on lowest ping') : _('Traffic routes through this selected server')
					)
				])
			])
		]);
		viewRoot.appendChild(modeCard);

		function updateSelectorVisibility() {
			dom.content(activeServerSelect, null);
			if (selectorMode === 'urltest') {
				activeServerSelect.disabled = true;
				activeServerSelect.appendChild(E('option', { 'value': 'auto', 'selected': true }, '⚡ ' + _('Auto (Lowest Latency)')));
				var hEl = document.getElementById('os-active-help');
				if (hEl) hEl.textContent = _('Automatically determined based on lowest ping');
			} else {
				activeServerSelect.disabled = false;
				if (!servers.length) {
					activeServerSelect.appendChild(E('option', { 'value': '' }, _('No servers available')));
				} else {
					servers.forEach(function(s) {
						var sId = s.id || s.server;
						var opt = E('option', { 'value': sId, 'selected': activeServer === sId }, (s.name || s.server) + ' (' + (s.protocol || 'node') + ')');
						activeServerSelect.appendChild(opt);
					});
				}
				var hEl2 = document.getElementById('os-active-help');
				if (hEl2) hEl2.textContent = _('Traffic routes through this selected server');
			}
		}

		// Import Subscription Box
		var importCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '📥 ', _('Import Subscriptions & Nodes') ])
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Subscription URL, Base64 or Node URI')),
				E('textarea', {
					'class': 'os-textarea',
					'id': 'os-subscription-input',
					'rows': 3,
					'placeholder': _('Paste https:// subscription link, Base64 block, or vless://, hysteria2://, tuic://, ss://, trojan:// URI')
				})
			]),
			E('div', { 'class': 'os-grid-2' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('User-Agent Header (Default: ClashMeta)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-subscription-ua',
						'placeholder': 'ClashMeta/v1.18.0, sing-box/1.10',
						'value': 'ClashMeta/v1.18.0'
					})
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Hardware ID (X-HWID for Private Providers)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-subscription-hwid',
						'placeholder': _('Optional: e.g. 64-char hex hardware ID')
					})
				])
			]),
			E('div', { 'style': 'display: flex; justify-content: flex-end; margin-top: 8px;' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var txt = document.getElementById('os-subscription-input');
						var val = txt ? txt.value.trim() : '';
						if (!val) {
							ui.addNotification(null, E('p', {}, _('Subscription or node link field is empty.')), 'warning');
							return;
						}
						var uaElem = document.getElementById('os-subscription-ua');
						var hwidElem = document.getElementById('os-subscription-hwid');
						var uaVal = uaElem ? uaElem.value.trim() : '';
						var hwidVal = hwidElem ? hwidElem.value.trim() : '';

						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ ' + _('Downloading & parsing...');
						callImportSubscription(val, uaVal, hwidVal).then(function(res) {
							btn.disabled = false;
							btn.innerText = '📥 ' + _('Import Nodes');
							if (res && res.success) {
								ui.addNotification(null, E('p', {}, '✓ ' + (res.message || _('Nodes imported successfully!'))), 'info');
								if (txt) txt.value = '';
								callGetServers().then(function(sRes) {
									if (sRes && sRes.servers) {
										servers = sRes.servers;
										updateSelectorVisibility();
										renderServerCards();
									}
								});
							} else {
								ui.addNotification(null, E('p', {}, '❌ ' + _('Import failed: ') + (res.error || _('Unknown error'))), 'error');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '📥 ' + _('Import Nodes');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Import error: ') + err), 'error');
						});
					}
				}, [ '📥 ', _('Import Nodes') ])
			])
		]);
		viewRoot.appendChild(importCard);

		// Servers List Card
		var serversCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🖥️ ', _('Configured Servers') ]),
				E('span', { 'class': 'os-badge os-badge-info', 'id': 'os-server-count' }, servers.length + ' ' + _('nodes'))
			]),
			E('div', { 'id': 'os-server-grid-box' })
		]);
		viewRoot.appendChild(serversCard);

		function renderServerCards() {
			var serversContainer = document.getElementById('os-server-grid-box');
			if (!serversContainer) return;

			var countEl = document.getElementById('os-server-count');
			if (countEl) countEl.textContent = servers.length + ' ' + _('nodes');

			dom.content(serversContainer, null);

			if (!servers.length) {
				serversContainer.appendChild(E('div', { 'style': 'text-align: center; padding: 40px 16px; color: var(--os-text-secondary);' }, [
					E('div', { 'style': 'font-size: 32px; margin-bottom: 8px;' }, '🌐'),
					E('div', { 'style': 'font-size: 14px; font-weight: 600;' }, _('No servers added yet')),
					E('div', { 'style': 'font-size: 12px; margin-top: 4px;' }, _('Paste a subscription link or node URI above to import outbounds.'))
				]));
				return;
			}

			servers.forEach(function(s, idx) {
				var protoBadge = 'os-badge-info';
				if (s.protocol === 'vless') protoBadge = 'os-badge-purple';
				else if (s.protocol === 'hysteria2') protoBadge = 'os-badge-info';
				else if (s.protocol === 'shadowsocks') protoBadge = 'os-badge-warning';

				var latencyBadge = 'os-badge-muted';
				var latencyText = _('Untested');
				if (s.latency_ms && s.latency_ms > 0) {
					latencyText = s.latency_ms + ' ms';
					if (s.latency_ms < 70) latencyBadge = 'os-badge-success';
					else if (s.latency_ms < 150) latencyBadge = 'os-badge-warning';
					else latencyBadge = 'os-badge-danger';
				}

				var isSelectedActive = (selectorMode === 'manual' && activeServer === (s.id || s.server));

				var card = E('div', {
					'class': 'os-entity-row',
					'style': isSelectedActive ? 'border-color: var(--os-primary); background: var(--os-primary-light);' : ''
				}, [
					E('div', { 'class': 'os-entity-main' }, [
						E('span', { 'style': 'font-size: 24px;' }, s.country_flag || '🌐'),
						E('div', { 'class': 'os-entity-details' }, [
							E('div', { 'style': 'display: flex; align-items: center; gap: 8px; flex-wrap: wrap;' }, [
								E('span', { 'class': 'os-entity-name' }, s.name || s.server),
								E('span', { 'class': 'os-badge ' + protoBadge }, s.protocol || 'node'),
								isSelectedActive ? E('span', { 'class': 'os-badge os-badge-success' }, '✓ ' + _('Active')) : null
							]),
							E('div', { 'class': 'os-entity-desc' }, [
								s.server + ':' + s.port,
								s.network ? ' • ' + s.network : '',
								s.security ? ' • ' + s.security : ''
							])
						])
					]),
					E('div', { 'class': 'os-entity-actions' }, [
						E('span', { 'class': 'os-badge ' + latencyBadge }, [ '⚡ ' + latencyText ]),
						selectorMode === 'manual' ? E('button', {
							'class': 'os-btn os-btn-sm ' + (isSelectedActive ? 'os-btn-success' : 'os-btn-secondary'),
							'click': function() {
								activeServer = s.id || s.server;
								updateSelectorVisibility();
								renderServerCards();
							}
						}, isSelectedActive ? _('Selected') : _('Select')) : null,
						E('button', {
							'class': 'os-btn os-btn-secondary os-btn-sm',
							'click': function(ev) {
								var b = ev.target;
								b.innerText = '⏳';
								callTestLatency(s.server, s.port).then(function(res) {
									b.innerText = '⚡ ' + _('Ping');
									if (res && res.latency_ms) {
										s.latency_ms = res.latency_ms;
										renderServerCards();
									}
								}).catch(function() {
									b.innerText = '⚡ ' + _('Ping');
								});
							}
						}, [ '⚡ ', _('Ping') ]),
						E('button', {
							'class': 'os-btn os-btn-danger os-btn-sm',
							'title': _('Delete server'),
							'click': function() {
								if (confirm(_('Delete server: ') + (s.name || s.server) + '?')) {
									servers.splice(idx, 1);
									updateSelectorVisibility();
									renderServerCards();
								}
							}
						}, [ '🗑️' ])
					])
				]);

				serversContainer.appendChild(card);
			});
		}

		updateSelectorVisibility();
		setTimeout(renderServerCards, 20);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
