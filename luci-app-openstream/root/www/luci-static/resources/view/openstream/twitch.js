'use strict';
'require view';
'require dom';
'require uci';
'require ui';

/*
 * OpenStream Engine 2.1 - Twitch Live Stream Optimizer
 * Unified Linear/Apple Design System, Mobile-First, Zero-CA, 100% i18n
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

return view.extend({
	load: function() {
		return uci.load('openstream').catch(function() {
			return null;
		});
	},

	render: function() {
		ensureStylesheet();

		var enabled = uci.get('openstream', 'twitch', 'enabled');
		if (enabled === null || enabled === undefined) enabled = '1';

		var preset = uci.get('openstream', 'twitch', 'preset') || 'clean_proxy_geosplit';
		var routeToken = uci.get('openstream', 'twitch', 'route_token') || 'vpn_adfree';
		var routeMaster = uci.get('openstream', 'twitch', 'route_master') || 'smartdns_comss';
		var routeSegments = uci.get('openstream', 'twitch', 'route_segments') || 'direct';

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '🟣 ', _('Twitch Live Stream Optimizer') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Server-side token split routing and SSAI ad-stripping without installing client SSL root certificates (Zero-CA).')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = _('Saving...');

						var isEnabled = document.getElementById('os-twitch-enabled').checked ? '1' : '0';
						var selPreset = document.getElementById('os-twitch-preset').value;
						var selToken = document.getElementById('os-twitch-token') ? document.getElementById('os-twitch-token').value : 'vpn_adfree';
						var selMaster = document.getElementById('os-twitch-master') ? document.getElementById('os-twitch-master').value : 'smartdns_comss';
						var selSegments = document.getElementById('os-twitch-segments') ? document.getElementById('os-twitch-segments').value : 'direct';

						// Ensure section exists
						var sections = uci.sections('openstream', 'twitch');
						if (!sections || !sections.length) {
							uci.add('openstream', 'twitch', 'twitch');
						}

						uci.set('openstream', 'twitch', 'enabled', isEnabled);
						uci.set('openstream', 'twitch', 'preset', selPreset);
						uci.set('openstream', 'twitch', 'route_token', selToken);
						uci.set('openstream', 'twitch', 'route_master', selMaster);
						uci.set('openstream', 'twitch', 'route_segments', selSegments);

						uci.save().then(function() {
							return uci.apply();
						}).then(function() {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save & Apply');
							ui.addNotification(null, E('p', {}, '✓ ' + _('Twitch Optimizer configuration saved and applied!')), 'info');
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save & Apply');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Save error: ') + err), 'error');
						});
					}
				}, [ '💾 ', _('Save & Apply') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// Preset Selection Card
		var customSettingsBox;
		var presetCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '⚡ ', _('Optimizer Mode & Routing Presets') ]),
				E('label', { 'style': 'display: flex; align-items: center; gap: 8px; cursor: pointer;' }, [
					E('label', { 'class': 'os-switch' }, [
						E('input', {
							'type': 'checkbox',
							'id': 'os-twitch-enabled',
							'checked': enabled === '1'
						}),
						E('span', { 'class': 'os-slider' })
					]),
					E('span', { 'class': 'os-label' }, _('Module Enabled'))
				])
			]),
			E('div', { 'class': 'os-form-group' }, [
				E('label', { 'class': 'os-label' }, _('Operational Preset')),
				E('select', {
					'class': 'os-select',
					'id': 'os-twitch-preset',
					'change': function(ev) {
						if (customSettingsBox) {
							customSettingsBox.style.display = (ev.target.value === 'custom') ? 'block' : 'none';
						}
					}
				}, [
					E('option', { 'value': 'clean_proxy_geosplit', 'selected': preset === 'clean_proxy_geosplit' },
						'🛡️ ' + _('Geo-Split via Clean-Proxy (Zero-CA for SmartTV/PC) [Recommended]')
					),
					E('option', { 'value': 'manifest_strip_edge', 'selected': preset === 'manifest_strip_edge' },
						'⚡ ' + _('Playlist Edge: Local SSAI ad-stripping in streamproxyd (:18080)')
					),
					E('option', { 'value': 'smartdns_quality_unlock', 'selected': preset === 'smartdns_quality_unlock' },
						'🌍 ' + _('Quality Unlock: 1080p60 unthrottled via SmartDNS')
					),
					E('option', { 'value': 'custom', 'selected': preset === 'custom' },
						'⚙️ ' + _('Custom: Fine-grained matrix split routing')
					),
					E('option', { 'value': 'off', 'selected': preset === 'off' },
						'⏹️ ' + _('Disabled')
					)
				]),
				E('div', { 'class': 'os-help' },
					_('Geo-Split requests stream playback access tokens from an ad-free region (e.g. Ukraine, Kazakhstan, Albania) while delivering raw 1080p video chunks directly from your local ISP CDN.')
				)
			]),

			// Custom Settings (Displayed when preset is 'custom')
			customSettingsBox = E('div', {
				'id': 'os-twitch-custom-box',
				'style': 'margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--os-border); display: ' + (preset === 'custom' ? 'block' : 'none') + ';'
			}, [
				E('h4', { 'style': 'margin: 0 0 12px 0; font-size: 14px; font-weight: 700; color: var(--os-text-primary);' },
					_('Matrix Routing Destinations')
				),
				E('div', { 'class': 'os-grid' }, [
					E('div', { 'class': 'os-form-group' }, [
						E('label', { 'class': 'os-label' }, _('Access Token Route (gql.twitch.tv)')),
						E('select', { 'class': 'os-select', 'id': 'os-twitch-token' }, [
							E('option', { 'value': 'vpn_adfree', 'selected': routeToken === 'vpn_adfree' }, _('Ad-Free VPN / Clean-Proxy (UA/AL/KZ)')),
							E('option', { 'value': 'vpn_eu', 'selected': routeToken === 'vpn_eu' }, _('Primary VPN (Europe)')),
							E('option', { 'value': 'direct', 'selected': routeToken === 'direct' }, _('Direct WAN (Provider Default)'))
						]),
						E('div', { 'class': 'os-help' }, _('Controls region-based SSAI ad-injection rules.'))
					]),
					E('div', { 'class': 'os-form-group' }, [
						E('label', { 'class': 'os-label' }, _('Master Playlist Route (usher.ttvnw.net)')),
						E('select', { 'class': 'os-select', 'id': 'os-twitch-master' }, [
							E('option', { 'value': 'smartdns_comss', 'selected': routeMaster === 'smartdns_comss' }, _('SmartDNS Comss.one (1080p60 Unlock)')),
							E('option', { 'value': 'vpn_eu', 'selected': routeMaster === 'vpn_eu' }, _('Primary VPN (Europe)')),
							E('option', { 'value': 'direct', 'selected': routeMaster === 'direct' }, _('Direct WAN (Provider Default)'))
						]),
						E('div', { 'class': 'os-help' }, _('Controls resolution availability and transcoder endpoints.'))
					]),
					E('div', { 'class': 'os-form-group' }, [
						E('label', { 'class': 'os-label' }, _('Video Stream CDN Route (live-video.net)')),
						E('select', { 'class': 'os-select', 'id': 'os-twitch-segments' }, [
							E('option', { 'value': 'direct', 'selected': routeSegments === 'direct' }, _('Direct WAN (Max ISP Bandwidth)')),
							E('option', { 'value': 'vpn_eu', 'selected': routeSegments === 'vpn_eu' }, _('Primary VPN (Europe)'))
						]),
						E('div', { 'class': 'os-help' }, _('Transports multi-gigabyte video segments.'))
					])
				])
			])
		]);
		viewRoot.appendChild(presetCard);

		// Architecture Info Card (Zero-CA Explanation)
		var infoCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '💡 ', _('Zero-CA Architectural Blueprint') ]),
				E('span', { 'class': 'os-badge os-badge-success' }, _('Client Certificate Free'))
			]),
			E('div', { 'class': 'os-grid-2' }, [
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('SmartTV, Consoles & Mobile')),
					E('div', { 'style': 'font-size: 13px; color: var(--os-text-secondary); line-height: 1.5; margin-top: 4px;' },
						_('Works out of the box on Apple TV, Android TV, LG webOS, Samsung Tizen, PlayStation, Xbox, iOS and Android without modifying device certificates or running mitmproxy.')
					)
				]),
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'class': 'os-stat-label' }, _('Bandwidth Efficiency')),
					E('div', { 'style': 'font-size: 13px; color: var(--os-text-secondary); line-height: 1.5; margin-top: 4px;' },
						_('Heavy video traffic (~8-12 Mbps per stream) never overloads your VPN tunnel. Only lightweight JSON token handshakes (~2 KB) are routed through geo-split outbounds.')
					)
				])
			])
		]);
		viewRoot.appendChild(infoCard);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
