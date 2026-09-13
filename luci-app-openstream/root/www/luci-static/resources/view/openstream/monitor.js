'use strict';
'require view';
'require rpc';
'require ui';
'require dom';
'require poll';

/*
 * OpenStream Engine 2.1 - Live Traffic & Flow Monitor
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

	load: function() {
		return callMonitorFlows().catch(function() {
			return { flows: [] };
		});
	},

	render: function(data) {
		ensureStylesheet();
		var self = this;
		var flows = (data && data.flows) ? data.flows : [];

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '📡 ', _('Live Traffic & Routing Monitor') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Real-time inspection of active network flows, core engine classification, and client traffic mapping.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-secondary',
					'click': function() {
						callClearFlows().then(function() {
							ui.addNotification(null, E('p', {}, '✓ ' + _('Flow monitor history cleared.')), 'info');
							flows = [];
							self.renderStats(flows);
							self.renderFlowList(flows);
						});
					}
				}, [ '🗑️ ', _('Clear History') ]),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ ' + _('Refreshing...');
						callMonitorFlows().then(function(res) {
							btn.disabled = false;
							btn.innerText = '🔄 ' + _('Refresh Now');
							flows = (res && res.flows) ? res.flows : [];
							self.renderStats(flows);
							self.renderFlowList(flows);
						}).catch(function() {
							btn.disabled = false;
							btn.innerText = '🔄 ' + _('Refresh Now');
						});
					}
				}, [ '🔄 ', _('Refresh Now') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// Stats Grid Card
		var statsCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '📊 ', _('Flow Distribution Metrics') ]),
				E('span', { 'class': 'os-badge os-badge-success', 'id': 'os-mon-live-badge' }, [
					E('span', { 'style': 'display: inline-block; width: 6px; height: 6px; border-radius: 50%; background: var(--os-success); margin-right: 4px;' }),
					_('Engine Active')
				])
			]),
			E('div', { 'class': 'os-grid', 'id': 'os-mon-stats-grid' })
		]);
		viewRoot.appendChild(statsCard);

		// Live Domain Inspector Card
		var inspectorResultBox;
		var inspectorCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🔍 ', _('Live Domain Routing Inspector') ])
			]),
			E('p', { 'class': 'os-help', 'style': 'margin-bottom: 12px;' },
				_('Query the active routing table in real-time to inspect which core subsystem will handle traffic for a specific domain:')
			),
			E('div', { 'style': 'display: flex; gap: 10px; flex-wrap: wrap;' }, [
				E('input', {
					'type': 'text',
					'class': 'os-input',
					'style': 'flex: 1; min-width: 240px;',
					'id': 'os-inspector-input',
					'placeholder': 'e.g. googlevideo.com, discord.gg, twitch.tv, rutracker.org'
				}),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function() {
						var input = document.getElementById('os-inspector-input');
						var val = input ? input.value.trim() : '';
						if (!val) return;

						callTestRoute(val).then(function(res) {
							if (res && res.result) {
								var r = res.result;
								dom.content(inspectorResultBox, null);
								inspectorResultBox.style.display = 'block';

								var badgeClass = 'os-badge-muted';
								if (r.route === 'zapret2') badgeClass = 'os-badge-success';
								else if (r.route === 'streamproxy') badgeClass = 'os-badge-info';
								else if (r.route === 'vpn') badgeClass = 'os-badge-purple';
								else if (r.route === 'block') badgeClass = 'os-badge-danger';

								inspectorResultBox.appendChild(E('div', { 'class': 'os-entity-row', 'style': 'margin-bottom: 0;' }, [
									E('div', { 'class': 'os-entity-main' }, [
										E('span', { 'style': 'font-size: 20px;' }, '🎯'),
										E('div', { 'class': 'os-entity-details' }, [
											E('span', { 'class': 'os-entity-name' }, r.domain),
											E('span', { 'class': 'os-entity-desc' }, r.details || _('Matched routing rule'))
										])
									]),
									E('div', { 'class': 'os-entity-actions' }, [
										E('span', { 'class': 'os-badge ' + badgeClass }, [ '● ' + (r.engine || r.route) ])
									])
								]));
							}
						});
					}
				}, [ '🔍 ', _('Inspect Route') ])
			]),
			inspectorResultBox = E('div', {
				'id': 'os-inspector-result',
				'style': 'display: none; margin-top: 14px;'
			})
		]);
		viewRoot.appendChild(inspectorCard);

		// Active Flows Card
		var flowsCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '⚡ ', _('Active Flow Classifications') ]),
				E('span', { 'class': 'os-badge os-badge-info', 'id': 'os-flows-count-badge' }, flows.length + ' ' + _('flows'))
			]),

			// Filter Bar
			E('div', { 'class': 'os-filter-bar', 'id': 'os-filter-bar' }, [
				E('button', {
					'class': 'os-filter-item active',
					'click': function(ev) { self.setFilter('all', ev.target, flows); }
				}, _('All Subsystems')),
				E('button', {
					'class': 'os-filter-item',
					'click': function(ev) { self.setFilter('zapret2', ev.target, flows); }
				}, '⚡ ' + _('Zapret2 (nfqws2)')),
				E('button', {
					'class': 'os-filter-item',
					'click': function(ev) { self.setFilter('streamproxy', ev.target, flows); }
				}, '🛡️ ' + _('StreamProxy (:18080)')),
				E('button', {
					'class': 'os-filter-item',
					'click': function(ev) { self.setFilter('singbox', ev.target, flows); }
				}, '🌐 ' + _('sing-box / VPN')),
				E('button', {
					'class': 'os-filter-item',
					'click': function(ev) { self.setFilter('block', ev.target, flows); }
				}, '🚫 ' + _('Sinkhole (Block)'))
			]),

			// Container for Flows
			E('div', { 'id': 'os-flows-container' })
		]);
		viewRoot.appendChild(flowsCard);

		this.renderStats = function(dataFlows) {
			var grid = document.getElementById('os-mon-stats-grid');
			if (!grid) return;
			dom.content(grid, null);

			var total = dataFlows.length;
			var zapretCount = dataFlows.filter(function(f) { return f.section === 'zapret2'; }).length;
			var streamCount = dataFlows.filter(function(f) { return f.section === 'streamproxy'; }).length;
			var vpnCount = dataFlows.filter(function(f) { return f.section === 'singbox' || f.section === 'vpn'; }).length;
			var blockCount = dataFlows.filter(function(f) { return f.section === 'block'; }).length;

			grid.appendChild(E('div', { 'class': 'os-stat-box' }, [
				E('div', { 'class': 'os-stat-label' }, _('Total Tracked Flows')),
				E('div', { 'class': 'os-stat-value', 'style': 'color: var(--os-primary);' }, String(total))
			]));
			grid.appendChild(E('div', { 'class': 'os-stat-box' }, [
				E('div', { 'class': 'os-stat-label' }, _('Zapret2 (nfqws2)')),
				E('div', { 'class': 'os-stat-value', 'style': 'color: var(--os-success);' }, String(zapretCount))
			]));
			grid.appendChild(E('div', { 'class': 'os-stat-box' }, [
				E('div', { 'class': 'os-stat-label' }, _('StreamProxy (:18080)')),
				E('div', { 'class': 'os-stat-value', 'style': 'color: var(--os-cyan);' }, String(streamCount))
			]));
			grid.appendChild(E('div', { 'class': 'os-stat-box' }, [
				E('div', { 'class': 'os-stat-label' }, _('sing-box Outbound')),
				E('div', { 'class': 'os-stat-value', 'style': 'color: var(--os-purple);' }, String(vpnCount))
			]));
			grid.appendChild(E('div', { 'class': 'os-stat-box' }, [
				E('div', { 'class': 'os-stat-label' }, _('DNS Sinkhole Block')),
				E('div', { 'class': 'os-stat-value', 'style': 'color: var(--os-danger);' }, String(blockCount))
			]));
		};

		this.setFilter = function(filterName, btnTarget, currentFlows) {
			self.filterSection = filterName;
			var filterBar = document.getElementById('os-filter-bar');
			if (filterBar) {
				var btns = filterBar.querySelectorAll('.os-filter-item');
				btns.forEach(function(b) { b.classList.remove('active'); });
				if (btnTarget) btnTarget.classList.add('active');
			}
			self.renderFlowList(currentFlows);
		};

		this.renderFlowList = function(currentFlows) {
			var container = document.getElementById('os-flows-container');
			if (!container) return;

			var badgeCount = document.getElementById('os-flows-count-badge');
			if (badgeCount) badgeCount.textContent = currentFlows.length + ' ' + _('flows');

			var filtered = currentFlows;
			if (self.filterSection !== 'all') {
				filtered = currentFlows.filter(function(f) {
					if (self.filterSection === 'singbox') {
						return f.section === 'singbox' || f.section === 'vpn';
					}
					return f.section === self.filterSection;
				});
			}

			dom.content(container, null);

			if (!filtered.length) {
				container.appendChild(E('div', {
					'style': 'text-align: center; padding: 40px 16px; color: var(--os-text-secondary);'
				}, [
					E('div', { 'style': 'font-size: 32px; margin-bottom: 8px;' }, '🔍'),
					E('div', { 'style': 'font-size: 14px; font-weight: 600;' }, _('No flows matched the current filter')),
					E('div', { 'style': 'font-size: 12px; margin-top: 4px;' }, _('Traffic in this category currently passes through standard default routes.'))
				]));
				return;
			}

			filtered.forEach(function(flow) {
				var badgeClass = 'os-badge-muted';
				if (flow.section === 'zapret2') badgeClass = 'os-badge-success';
				else if (flow.section === 'streamproxy') badgeClass = 'os-badge-info';
				else if (flow.section === 'singbox' || flow.section === 'vpn') badgeClass = 'os-badge-purple';
				else if (flow.section === 'block') badgeClass = 'os-badge-danger';

				var card = E('div', { 'class': 'os-entity-row' }, [
					E('div', { 'class': 'os-entity-main' }, [
						E('span', { 'style': 'display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--os-success); box-shadow: 0 0 6px var(--os-success); margin-right: 4px;' }),
						E('div', { 'class': 'os-entity-details' }, [
							E('span', { 'class': 'os-entity-name' }, flow.domain),
							E('span', { 'class': 'os-entity-desc' }, _('Applied to all LAN clients'))
						])
					]),
					E('div', { 'class': 'os-entity-actions' }, [
						E('span', { 'class': 'os-badge ' + badgeClass }, flow.details || flow.section)
					])
				]);

				container.appendChild(card);
			});
		};

		setTimeout(function() {
			self.renderStats(flows);
			self.renderFlowList(flows);
		}, 20);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
