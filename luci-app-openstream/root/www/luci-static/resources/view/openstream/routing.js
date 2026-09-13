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

function ensureStylesheet() {
	var href = (window.L && L.resource) ? L.resource('openstream/openstream.css') : '/luci-static/resources/openstream/openstream.css';
	if (!document.querySelector('link[href*="openstream.css"]')) {
		document.head.appendChild(E('link', {
			'rel': 'stylesheet',
			'type': 'text/css',
			'href': href
		}));
	}
}

return view.extend({
	filterType: 'all',

	load: function() {
		ensureStylesheet();
		return callRoutingGet().catch(function() { return {}; });
	},

	render: function(data) {
		ensureStylesheet();
		var self = this;

		var rules = (data && data.rules) ? data.rules : [];
		var clients = (data && data.clients) ? data.clients : [];
		var zapret2Installed = (data && data.zapret2_installed) || false;
		var singboxInstalled = (data && data.singbox_installed) || false;

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Header Card
		var hero = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [
					E('span', {}, '🔀'),
					_('Policy Routing & Rules')
				]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Manage declarative traffic rules (.osrule.yaml), test domain resolution, and configure client policy routing.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function() { self.showRuleModal(null, rules); }
				}, [ E('span', {}, '➕'), _('Add Custom Rule') ]),
				E('button', {
					'class': 'os-btn os-btn-success',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						ui.showIndicator('os-save-rules', _('Applying routing rules…'));
						callRoutingSave(rules).then(function() {
							ui.hideIndicator('os-save-rules');
							ui.addNotification(null, E('p', {}, _('Routing rules applied successfully!')), 'info');
							btn.disabled = false;
						}).catch(function(e) {
							ui.hideIndicator('os-save-rules');
							ui.addNotification(null, E('p', {}, _('Failed to apply rules: ') + (e.message || e)), 'danger');
							btn.disabled = false;
						});
					}
				}, [ E('span', {}, '💾'), _('Apply Changes') ])
			])
		]);
		viewRoot.appendChild(hero);

		// Domain Tester Tool Card
		var testCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [
					E('span', {}, '🎯'),
					_('Real-time Domain Route Inspector')
				])
			]),
			E('div', { 'style': 'display: flex; gap: 10px; flex-wrap: wrap; align-items: center;' }, [
				E('input', {
					'type': 'text',
					'class': 'os-input',
					'placeholder': 'e.g. video.twitch.tv or discord.com',
					'id': 'os-test-domain-input',
					'style': 'flex: 1; min-width: 240px;',
					'keydown': function(ev) {
						if (ev.key === 'Enter') document.getElementById('os-test-domain-btn').click();
					}
				}),
				E('button', {
					'id': 'os-test-domain-btn',
					'class': 'os-btn os-btn-secondary',
					'click': function() {
						var input = document.getElementById('os-test-domain-input');
						var resultBox = document.getElementById('os-test-result');
						var domain = (input.value || '').trim();
						if (!domain) return;

						resultBox.innerHTML = '';
						resultBox.appendChild(E('span', { 'class': 'os-badge os-badge-info' }, _('Testing…')));

						callRoutingTest(domain).then(function(res) {
							resultBox.innerHTML = '';
							var route = (res && res.route) || 'direct';
							var badgeClass = 'os-badge-info';
							if (route === 'proxy' || route === 'streamproxy') badgeClass = 'os-badge-purple';
							else if (route === 'zapret2' || route === 'nfqws2') badgeClass = 'os-badge-warning';
							else if (route === 'block') badgeClass = 'os-badge-danger';
							else badgeClass = 'os-badge-success';

							resultBox.appendChild(E('span', { 'class': 'os-badge ' + badgeClass }, [
								E('strong', {}, route.toUpperCase()),
								' (' + (res.matched_rule || _('Default WAN')) + ')'
							]));
						}).catch(function(err) {
							resultBox.innerHTML = '';
							resultBox.appendChild(E('span', { 'class': 'os-badge os-badge-danger' }, _('Error: ') + (err.message || err)));
						});
					}
				}, [ E('span', {}, '🔍'), _('Inspect Route') ]),
				E('div', { 'id': 'os-test-result', 'style': 'display: flex; align-items: center;' })
			])
		]);
		viewRoot.appendChild(testCard);

		// Rules Filter & List Card
		var rulesCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [
					E('span', {}, '📋'),
					_('Active Declarative Rules'),
					E('span', { 'class': 'os-badge os-badge-muted' }, rules.length + ' ' + _('rules loaded'))
				]),
				E('div', { 'class': 'os-filter-bar', 'style': 'margin-bottom: 0;' }, [
					E('button', {
						'class': 'os-filter-item' + (self.filterType === 'all' ? ' active' : ''),
						'click': function() { self.filterType = 'all'; self.renderRulesList(rules, listContainer); }
					}, _('All')),
					E('button', {
						'class': 'os-filter-item' + (self.filterType === 'streaming' ? ' active' : ''),
						'click': function() { self.filterType = 'streaming'; self.renderRulesList(rules, listContainer); }
					}, _('Streaming & Media')),
					E('button', {
						'class': 'os-filter-item' + (self.filterType === 'privacy' ? ' active' : ''),
						'click': function() { self.filterType = 'privacy'; self.renderRulesList(rules, listContainer); }
					}, _('Privacy & AdBlock')),
					E('button', {
						'class': 'os-filter-item' + (self.filterType === 'custom' ? ' active' : ''),
						'click': function() { self.filterType = 'custom'; self.renderRulesList(rules, listContainer); }
					}, _('Custom'))
				])
			])
		]);

		var listContainer = E('div', { 'id': 'os-rules-list' });
		rulesCard.appendChild(listContainer);
		viewRoot.appendChild(rulesCard);

		self.renderRulesList(rules, listContainer);

		return viewRoot;
	},

	renderRulesList: function(rules, container) {
		var self = this;
		container.innerHTML = '';

		var filtered = rules.filter(function(r) {
			if (self.filterType === 'all') return true;
			var id = (r.id || r.name || '').toLowerCase();
			if (self.filterType === 'streaming') return id.includes('twitch') || id.includes('youtube') || id.includes('discord') || id.includes('crunchyroll');
			if (self.filterType === 'privacy') return id.includes('adblock') || id.includes('torrent') || id.includes('privacy');
			if (self.filterType === 'custom') return !id.includes('twitch') && !id.includes('youtube') && !id.includes('discord') && !id.includes('adblock') && !id.includes('torrent');
			return true;
		});

		if (filtered.length === 0) {
			container.appendChild(E('div', { 'style': 'padding: 24px; text-align: center; color: var(--os-text-muted);' },
				_('No rules matching selected filter.')
			));
			return;
		}

		filtered.forEach(function(rule, idx) {
			var isEnabled = rule.enabled !== false;
			var action = (rule.action || 'proxy').toLowerCase();
			var actionBadgeClass = 'os-badge-info';
			if (action === 'proxy' || action === 'streamproxy') actionBadgeClass = 'os-badge-purple';
			else if (action === 'zapret2' || action === 'nfqws2') actionBadgeClass = 'os-badge-warning';
			else if (action === 'direct' || action === 'bypass') actionBadgeClass = 'os-badge-success';
			else if (action === 'block') actionBadgeClass = 'os-badge-danger';

			var row = E('div', { 'class': 'os-entity-row' }, [
				E('div', { 'class': 'os-entity-main' }, [
					E('input', {
						'type': 'checkbox',
						'checked': isEnabled,
						'style': 'width: 18px; height: 18px; cursor: pointer; accent-color: var(--os-primary);',
						'change': function(ev) {
							rule.enabled = ev.target.checked;
						}
					}),
					E('div', { 'class': 'os-entity-details' }, [
						E('div', { 'style': 'display: flex; align-items: center; gap: 8px;' }, [
							E('span', { 'class': 'os-entity-name' }, rule.name || rule.id || _('Unnamed Rule')),
							E('span', { 'class': 'os-badge ' + actionBadgeClass }, action.toUpperCase()),
							rule.file ? E('span', { 'class': 'os-badge os-badge-muted' }, rule.file) : E('span')
						]),
						E('span', { 'class': 'os-entity-desc' },
							(rule.description || _('Declarative routing policy')) + ' • ' +
							((rule.domains && rule.domains.length) ? rule.domains.length + ' ' + _('domains') : _('no domains'))
						)
					])
				]),
				E('div', { 'class': 'os-entity-actions' }, [
					E('button', {
						'class': 'os-btn os-btn-secondary os-btn-sm',
						'click': function() { self.showRuleModal(rule, rules); }
					}, [ E('span', {}, '✏️'), _('Edit') ]),
					E('button', {
						'class': 'os-btn os-btn-danger os-btn-sm',
						'click': function() {
							if (confirm(_('Are you sure you want to delete this rule?'))) {
								var rIdx = rules.indexOf(rule);
								if (rIdx >= 0) rules.splice(rIdx, 1);
								self.renderRulesList(rules, container);
							}
						}
					}, [ E('span', {}, '🗑️') ])
				])
			]);

			container.appendChild(row);
		});
	},

	showRuleModal: function(rule, allRules) {
		var self = this;
		var isNew = !rule;
		var current = rule ? Object.assign({}, rule) : {
			id: 'custom_rule_' + Date.now(),
			name: '',
			description: '',
			action: 'proxy',
			enabled: true,
			domains: [],
			cidrs: []
		};

		var idInput = E('input', { 'class': 'os-input', 'value': current.id || '', 'disabled': !isNew });
		var nameInput = E('input', { 'class': 'os-input', 'value': current.name || '', 'placeholder': _('e.g. My Custom Service') });
		var descInput = E('input', { 'class': 'os-input', 'value': current.description || '', 'placeholder': _('Short rule purpose') });

		var actionSelect = E('select', { 'class': 'os-select' }, [
			E('option', { 'value': 'proxy', 'selected': current.action === 'proxy' }, _('Proxy (sing-box TPROXY)')),
			E('option', { 'value': 'streamproxy', 'selected': current.action === 'streamproxy' }, _('HLS/DASH Stream Proxy (Port 8888)')),
			E('option', { 'value': 'zapret2', 'selected': current.action === 'zapret2' }, _('Anti-DPI (Zapret2 NFQUEUE)')),
			E('option', { 'value': 'direct', 'selected': current.action === 'direct' }, _('Direct WAN (Bypass)')),
			E('option', { 'value': 'block', 'selected': current.action === 'block' }, _('Block (Reject/Drop)'))
		]);

		var domainsArea = E('textarea', {
			'class': 'os-textarea',
			'rows': 5,
			'placeholder': _('One domain per line, e.g.:\nexample.com\n*.cdn.example.org')
		}, (current.domains || []).join('\n'));

		var cidrsArea = E('textarea', {
			'class': 'os-textarea',
			'rows': 3,
			'placeholder': _('One IP or CIDR per line, e.g.:\n198.51.100.0/24\n203.0.113.5')
		}, (current.cidrs || []).join('\n'));

		var modalContent = E('div', { 'class': 'os-container', 'style': 'padding: 0;' }, [
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Rule Identifier (ID)')),
				idInput
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Display Name')),
				nameInput
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Description')),
				descInput
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Routing Target Action')),
				actionSelect
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Target Domains (Wildcards allowed)')),
				domainsArea,
				E('span', { 'class': 'os-help' }, _('Domains will be automatically populated into nftables dynamic target sets via dnsmasq.'))
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Target IP / CIDR Ranges (Optional)')),
				cidrsArea
			])
		]);

		ui.showModal(isNew ? _('Create Routing Rule') : _('Edit Routing Rule'), [
			modalContent,
			E('div', { 'class': 'right', 'style': 'margin-top: 16px; display: flex; justify-content: flex-end; gap: 8px;' }, [
				E('button', {
					'class': 'os-btn os-btn-secondary',
					'click': ui.hideModal
				}, _('Cancel')),
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function() {
						var parsedDomains = domainsArea.value.split('\n').map(function(s) { return s.trim(); }).filter(Boolean);
						var parsedCidrs = cidrsArea.value.split('\n').map(function(s) { return s.trim(); }).filter(Boolean);

						var updated = {
							id: idInput.value.trim() || current.id,
							name: nameInput.value.trim() || idInput.value.trim(),
							description: descInput.value.trim(),
							action: actionSelect.value,
							enabled: current.enabled !== false,
							domains: parsedDomains,
							cidrs: parsedCidrs,
							file: current.file || (idInput.value.trim() + '.osrule.yaml')
						};

						if (isNew) {
							allRules.push(updated);
						} else {
							var idx = allRules.indexOf(rule);
							if (idx >= 0) allRules[idx] = updated;
						}

						ui.hideModal();
						var listContainer = document.getElementById('os-rules-list');
						if (listContainer) self.renderRulesList(allRules, listContainer);
					}
				}, _('Save Rule'))
			])
		]);
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
