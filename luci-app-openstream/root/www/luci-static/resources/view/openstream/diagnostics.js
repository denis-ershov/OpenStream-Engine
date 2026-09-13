'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Self-Diagnostics & System Health
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
		ensureStylesheet();

		var initial = results[0] || {};
		var checks = initial.checks || [];

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '🩺 ', _('Self-Diagnostics & Health Hub') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Comprehensive automated verification of nftables chains, sing-box sockets, Zapret2 DPI queues, and DNS integrity.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ ' + _('Diagnosing...');
						callRunDiagnostics().then(function(res) {
							btn.disabled = false;
							btn.innerText = '▶️ ' + _('Run Diagnostics');
							if (res && res.checks) {
								checks = res.checks;
								renderDiagCards();
								ui.addNotification(null, E('p', {}, '✓ ' + _('Self-diagnostics completed!')), 'info');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '▶️ ' + _('Run Diagnostics');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Diagnostics error: ') + err), 'error');
						});
					}
				}, [ '▶️ ', _('Run Diagnostics') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// Diagnostics Results Card
		var diagCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🔬 ', _('Diagnostic Checks') ]),
				E('span', { 'class': 'os-badge os-badge-info', 'id': 'os-diag-count' }, checks.length + ' ' + _('checks'))
			]),
			E('div', { 'id': 'os-diag-container' })
		]);
		viewRoot.appendChild(diagCard);

		function renderDiagCards() {
			var container = document.getElementById('os-diag-container');
			if (!container) return;

			var countEl = document.getElementById('os-diag-count');
			if (countEl) countEl.textContent = checks.length + ' ' + _('checks');

			dom.content(container, null);

			if (!checks.length) {
				container.appendChild(E('div', { 'style': 'text-align: center; padding: 40px 16px; color: var(--os-text-secondary);' }, [
					E('div', { 'style': 'font-size: 32px; margin-bottom: 8px;' }, '🩺'),
					E('div', { 'style': 'font-size: 14px; font-weight: 600;' }, _('No diagnostic results yet')),
					E('div', { 'style': 'font-size: 12px; margin-top: 4px;' }, _('Click "Run Diagnostics" to inspect system components.'))
				]));
				return;
			}

			checks.forEach(function(c) {
				var badgeClass = 'os-badge-success';
				var badgeText = '✓ ' + _('HEALTHY');
				if (c.status === 'warn') {
					badgeClass = 'os-badge-warning';
					badgeText = '⚠️ ' + _('WARNING');
				} else if (c.status === 'fail' || c.status === 'error') {
					badgeClass = 'os-badge-danger';
					badgeText = '❌ ' + _('FAIL');
				}

				var row = E('div', { 'class': 'os-entity-row' }, [
					E('div', { 'class': 'os-entity-main' }, [
						E('div', { 'class': 'os-entity-details' }, [
							E('span', { 'class': 'os-entity-name' }, c.name),
							E('span', { 'class': 'os-entity-desc' }, c.details || '')
						])
					]),
					E('div', { 'class': 'os-entity-actions' }, [
						E('span', { 'class': 'os-badge ' + badgeClass }, badgeText)
					])
				]);

				container.appendChild(row);
			});
		}

		setTimeout(renderDiagCards, 20);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
