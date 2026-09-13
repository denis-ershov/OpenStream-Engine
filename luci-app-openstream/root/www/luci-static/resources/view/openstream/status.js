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

var callApplyRules = rpc.declare({
	object: 'openstream',
	method: 'save_routing',
	params: [ 'rules' ],
	expect: { '': {} }
});

var callInitAction = rpc.declare({
	object: 'luci',
	method: 'setInitStatus',
	params: [ 'name', 'action' ],
	expect: { '': true }
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
	load: function() {
		ensureStylesheet();
		return callStatus().catch(function() { return {}; });
	},

	render: function(data) {
		ensureStylesheet();

		var isRunning = data && data.running;
		var zapret2Installed = data && data.zapret2_installed;
		var zapret2Running = data && data.zapret2_running;
		var singboxInstalled = data && data.singbox_installed;
		var singboxRunning = data && data.singbox_running;
		var singboxVariant = (data && data.singbox_variant) || 'stable';
		var version = (data && data.version) || '2.1.0-r37';

		var viewRoot = E('div', { 'class': 'os-container' }, [
			// Hero Header Card
			E('div', { 'class': 'os-hero' }, [
				E('div', { 'class': 'os-hero-title-wrap' }, [
					E('h2', { 'class': 'os-hero-title' }, [
						E('span', {}, '⚡'),
						_('OpenStream Engine'),
						E('span', { 'class': 'os-badge os-badge-info' }, 'v' + version)
					]),
					E('p', { 'class': 'os-hero-subtitle' },
						_('Universal Cross-Platform Traffic Orchestrator & Declarative Policy Routing Engine')
					)
				]),
				E('div', { 'class': 'os-hero-actions' }, [
					E('span', {
						'class': 'os-badge ' + (isRunning ? 'os-badge-success' : 'os-badge-danger')
					}, isRunning ? _('● Service Running') : _('○ Service Stopped')),
					E('button', {
						'class': 'os-btn os-btn-secondary',
						'click': function(ev) {
							var btn = ev.target;
							btn.disabled = true;
							ui.showIndicator('os-restart', _('Restarting streamproxyd…'));
							callInitAction('streamproxyd', 'restart').then(function() {
								setTimeout(function() {
									ui.hideIndicator('os-restart');
									window.location.reload();
								}, 1500);
							}).catch(function(e) {
								ui.hideIndicator('os-restart');
								ui.addNotification(null, E('p', {}, _('Failed to restart service: ') + (e.message || e)));
								btn.disabled = false;
							});
						}
					}, [ E('span', {}, '🔄'), _('Restart Service') ])
				])
			]),

			// Key Status Indicators Grid
			E('div', { 'class': 'os-grid' }, [
				// Rust Daemon
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('Core Daemon (Rust)')),
					E('div', { 'class': 'os-stat-value' }, isRunning ? 'streamproxyd' : _('Inactive')),
					E('div', {}, [
						E('span', {
							'class': 'os-badge ' + (isRunning ? 'os-badge-success' : 'os-badge-danger')
						}, isRunning ? _('Online (Port 8888)') : _('Stopped'))
					])
				]),

				// Zapret2 Anti-DPI
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('Anti-DPI (Zapret2 nfqws2)')),
					E('div', { 'class': 'os-stat-value' }, zapret2Running ? _('Active') : (zapret2Installed ? _('Ready') : _('Not Installed'))),
					E('div', {}, [
						E('span', {
							'class': 'os-badge ' + (zapret2Running ? 'os-badge-success' : (zapret2Installed ? 'os-badge-info' : 'os-badge-muted'))
						}, zapret2Running ? _('NFQUEUE 1088') : (zapret2Installed ? _('Installed') : _('Optional')))
					])
				]),

				// sing-box TPROXY
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('Proxy Core (sing-box)')),
					E('div', { 'class': 'os-stat-value' }, singboxRunning ? _('Active') : (singboxInstalled ? _('Standby') : _('Not Installed'))),
					E('div', {}, [
						E('span', {
							'class': 'os-badge ' + (singboxRunning ? 'os-badge-success' : (singboxInstalled ? 'os-badge-purple' : 'os-badge-muted'))
						}, singboxVariant.toUpperCase() + (singboxRunning ? ' (TPROXY 10888)' : ''))
					])
				]),

				// Policy Routing Engine
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('Orchestration Layer')),
					E('div', { 'class': 'os-stat-value' }, 'ucode / nftables'),
					E('div', {}, [
						E('span', { 'class': 'os-badge os-badge-cyan' }, _('Zero-Copy Routing'))
					])
				])
			]),

			// Architectural Overview Card
			E('div', { 'class': 'os-card' }, [
				E('div', { 'class': 'os-card-header' }, [
					E('h3', { 'class': 'os-card-title' }, [
						E('span', {}, '🛡️'),
						_('System Architecture & Network Interfaces')
					])
				]),
				E('div', { 'class': 'os-grid-2' }, [
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-label' }, _('Video Stream Proxy')),
						E('p', { 'class': 'os-help' },
							_('Transparent HTTP/HTTPS stream rewriter strips server-side inserted advertisements (SSAI) in HLS/DASH playlists without requiring root CA certificates on client devices.')
						)
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-label' }, _('Declarative Policy Routing')),
						E('p', { 'class': 'os-help' },
							_('Kernel-level fwmark packet dispatching via Linux nftables & IP rule sets routes targeted traffic across Direct, VPN, Anti-DPI, and Proxy egress interfaces.')
						)
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-label' }, _('Multi-DNS & DoH Shield')),
						E('p', { 'class': 'os-help' },
							_('Automatic domain population into nftables dynamic sets with failover resolvers and automated DNS leak mitigation.')
						)
					]),
					E('div', { 'class': 'os-stat-box' }, [
						E('div', { 'class': 'os-stat-label' }, _('Ecosystem Coexistence')),
						E('p', { 'class': 'os-help' },
							_('Idempotent table management preserving dynamic sets across reloads with mutual conflict isolation alongside Forkop, Podkop, and Passwall.')
						)
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
