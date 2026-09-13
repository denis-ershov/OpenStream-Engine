'use strict';
'require view';
'require dom';
'require rpc';
'require ui';

/*
 * OpenStream Engine 2.1 - Services, Multi-DNS Failover & Security Hub
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

var callGetDnsConfig = rpc.declare({
	object: 'openstream',
	method: 'get_dns_config',
	expect: { success: true }
});

var callSaveDnsConfig = rpc.declare({
	object: 'openstream',
	method: 'save_dns_config',
	params: [ 'dns_servers', 'bootstrap_dns', 'failover_enabled', 'prefer_ipv4', 'failover_threshold', 'failover_recovery_sec', 'doh_client_cert', 'doh_client_key', 'dns_hosts' ],
	expect: { success: true }
});

var callGetSecurity = rpc.declare({
	object: 'openstream',
	method: 'get_security_settings',
	expect: { success: true }
});

var callSaveSecurity = rpc.declare({
	object: 'openstream',
	method: 'save_security_settings',
	params: [ 'disable_quic', 'block_doh', 'exclude_ntp', 'torrent_bypass', 'download_via_proxy', 'bypass_cidrs', 'bypass_clients' ],
	expect: { success: true }
});

var callCreateBackup = rpc.declare({
	object: 'openstream',
	method: 'create_backup',
	expect: { success: true }
});

var callRestoreBackup = rpc.declare({
	object: 'openstream',
	method: 'restore_backup',
	params: [ 'data_base64' ],
	expect: { success: true }
});

return view.extend({
	load: function() {
		return Promise.all([
			callGetDnsConfig().catch(function() { return {}; }),
			callGetSecurity().catch(function() { return {}; })
		]);
	},

	render: function(results) {
		ensureStylesheet();

		var dnsCfg = results[0] || {};
		var secCfg = results[1] || {};

		var viewRoot = E('div', { 'class': 'os-container' });

		// Hero Card
		var heroNode = E('div', { 'class': 'os-hero' }, [
			E('div', { 'class': 'os-hero-title-wrap' }, [
				E('h2', { 'class': 'os-hero-title' }, [ '⚙️ ', _('Network Services & Security') ]),
				E('p', { 'class': 'os-hero-subtitle' },
					_('Configure cascading DNS failover, DoH bootstrap, QUIC/DoH blocking, BitTorrent WAN bypass, and system backup.')
				)
			]),
			E('div', { 'class': 'os-hero-actions' }, [
				E('button', {
					'class': 'os-btn os-btn-primary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = _('Saving settings...');

						var dnsRaw = document.getElementById('os-dns-servers') ? document.getElementById('os-dns-servers').value.trim() : '';
						var bootRaw = document.getElementById('os-dns-bootstrap') ? document.getElementById('os-dns-bootstrap').value.trim() : '';
						var dnsList = dnsRaw ? dnsRaw.split(/[\r\n]+/).map(function(s) { return s.trim(); }).filter(Boolean) : [];
						var bootList = bootRaw ? bootRaw.split(/[\r\n]+/).map(function(s) { return s.trim(); }).filter(Boolean) : [];

						var failover = document.getElementById('os-dns-failover') ? document.getElementById('os-dns-failover').checked : true;
						var preferIpv4 = document.getElementById('os-dns-ipv4') ? document.getElementById('os-dns-ipv4').checked : true;
						var threshold = document.getElementById('os-dns-threshold') ? parseInt(document.getElementById('os-dns-threshold').value) : 3;
						var recovery = document.getElementById('os-dns-recovery') ? parseInt(document.getElementById('os-dns-recovery').value) : 30;
						var cert = document.getElementById('os-dns-cert') ? document.getElementById('os-dns-cert').value.trim() : '';
						var key = document.getElementById('os-dns-key') ? document.getElementById('os-dns-key').value.trim() : '';
						var hostsRaw = document.getElementById('os-dns-hosts') ? document.getElementById('os-dns-hosts').value.trim() : '';
						var hostsList = hostsRaw ? hostsRaw.split(/[\r\n]+/).map(function(s) { return s.trim(); }).filter(Boolean) : [];

						var disQuic = document.getElementById('os-sec-quic') ? document.getElementById('os-sec-quic').checked : true;
						var blkDoh = document.getElementById('os-sec-doh') ? document.getElementById('os-sec-doh').checked : true;
						var exNtp = document.getElementById('os-sec-ntp') ? document.getElementById('os-sec-ntp').checked : true;
						var torrentBypass = document.getElementById('os-sec-torrent') ? document.getElementById('os-sec-torrent').checked : true;
						var dlProxy = document.getElementById('os-sec-dlproxy') ? document.getElementById('os-sec-dlproxy').checked : false;
						var cidrsRaw = document.getElementById('os-sec-cidrs') ? document.getElementById('os-sec-cidrs').value.trim() : '';
						var clientsRaw = document.getElementById('os-sec-clients') ? document.getElementById('os-sec-clients').value.trim() : '';
						var cidrsList = cidrsRaw ? cidrsRaw.split(/[\r\n,]+/).map(function(s) { return s.trim(); }).filter(Boolean) : [];
						var clientsList = clientsRaw ? clientsRaw.split(/[\r\n,]+/).map(function(s) { return s.trim(); }).filter(Boolean) : [];

						Promise.all([
							callSaveDnsConfig(dnsList, bootList, failover, preferIpv4, threshold, recovery, cert, key, hostsList),
							callSaveSecurity(disQuic, blkDoh, exNtp, torrentBypass, dlProxy, cidrsList, clientsList)
						]).then(function() {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save All Settings');
							ui.addNotification(null, E('p', {}, '✓ ' + _('DNS and network security settings applied successfully!')), 'info');
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '💾 ' + _('Save All Settings');
							ui.addNotification(null, E('p', {}, '❌ ' + _('RPC Error: ') + err), 'error');
						});
					}
				}, [ '💾 ', _('Save All Settings') ])
			])
		]);
		viewRoot.appendChild(heroNode);

		// Section 1: Multi-DNS Failover, Hysteresis, mTLS & Bootstrap DNS
		var dnsServersStr = (dnsCfg.dns_servers || [
			'https://1.1.1.1/dns-query',
			'https://dns.google/dns-query',
			'tls://dns.quad9.net'
		]).join('\n');

		var bootstrapStr = (dnsCfg.bootstrap_dns || [
			'77.88.8.8',
			'1.1.1.1'
		]).join('\n');

		var dnsHostsStr = (dnsCfg.dns_hosts || []).join('\n');

		var dnsCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🛡️ ', _('Multi-DNS Failover & Bootstrap Resolver') ]),
				E('span', { 'class': 'os-badge os-badge-info' }, _('Resilient DNS'))
			]),
			E('div', { 'class': 'os-grid-2' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Upstream DNS Servers (DoH / DoT / UDP - one per line)')),
					E('textarea', {
						'class': 'os-textarea',
						'id': 'os-dns-servers',
						'rows': 4
					}, dnsServersStr),
					E('div', { 'class': 'os-help' }, _('Cascading upstream resolvers queried through the secure engine.'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Bootstrap DNS (Static plain IPs for resolving DoH endpoints)')),
					E('textarea', {
						'class': 'os-textarea',
						'id': 'os-dns-bootstrap',
						'rows': 4
					}, bootstrapStr),
					E('div', { 'class': 'os-help' }, _('Plain IP resolvers used to bootstrap initial DoH server domain names.'))
				])
			]),
			E('div', { 'class': 'os-grid-2', 'style': 'margin-top: 10px;' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Failover Failure Threshold')),
					E('input', {
						'type': 'number',
						'class': 'os-input',
						'id': 'os-dns-threshold',
						'min': '1',
						'max': '10',
						'value': dnsCfg.failover_threshold || 3
					}),
					E('div', { 'class': 'os-help' }, _('Number of consecutive failed queries before switching to secondary DNS (anti-flapping).'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Primary Recovery Interval (seconds)')),
					E('input', {
						'type': 'number',
						'class': 'os-input',
						'id': 'os-dns-recovery',
						'min': '5',
						'max': '300',
						'value': dnsCfg.failover_recovery_sec || 30
					}),
					E('div', { 'class': 'os-help' }, _('Time to wait before probing and returning to the primary DNS server.'))
				])
			]),
			E('div', { 'class': 'os-grid-2', 'style': 'margin-top: 10px;' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('mTLS DoH: Client Certificate (PEM Path)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-dns-cert',
						'placeholder': '/etc/ssl/certs/client.crt',
						'value': dnsCfg.doh_client_cert || ''
					}),
					E('div', { 'class': 'os-help' }, _('Optional client certificate for mutual TLS DoH resolvers.'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('mTLS DoH: Client Private Key (PEM Path)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-dns-key',
						'placeholder': '/etc/ssl/private/client.key',
						'value': dnsCfg.doh_client_key || ''
					}),
					E('div', { 'class': 'os-help' }, _('Optional private key matching the client certificate.'))
				])
			]),
			E('div', { 'class': 'os-form-group', 'style': 'margin-top: 10px;' }, [
				E('label', { 'class': 'os-label' }, _('Local DNS Overrides (Hosts format: domain IP)')),
				E('textarea', {
					'class': 'os-textarea',
					'id': 'os-dns-hosts',
					'rows': 2,
					'placeholder': 'router.local 192.168.1.1\nnas.home 192.168.1.200'
				}, dnsHostsStr),
				E('div', { 'class': 'os-help' }, _('Static domain mappings evaluated before querying upstream DNS.'))
			]),
			E('div', { 'style': 'display: flex; gap: 24px; flex-wrap: wrap; margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--os-border);' }, [
				E('label', { 'style': 'display: flex; align-items: center; gap: 10px; cursor: pointer;' }, [
					E('label', { 'class': 'os-switch' }, [
						E('input', { 'type': 'checkbox', 'id': 'os-dns-failover', 'checked': dnsCfg.failover_enabled !== false }),
						E('span', { 'class': 'os-slider' })
					]),
					E('span', { 'class': 'os-label' }, _('Automatic DNS Failover'))
				]),
				E('label', { 'style': 'display: flex; align-items: center; gap: 10px; cursor: pointer;' }, [
					E('label', { 'class': 'os-switch' }, [
						E('input', { 'type': 'checkbox', 'id': 'os-dns-ipv4', 'checked': dnsCfg.prefer_ipv4 !== false }),
						E('span', { 'class': 'os-slider' })
					]),
					E('span', { 'class': 'os-label' }, _('Prefer IPv4 for DNS Queries'))
				])
			])
		]);
		viewRoot.appendChild(dnsCard);

		// Section 2: Network Security & P2P Bypass
		var cidrsStr = (secCfg.bypass_cidrs || []).join(', ');
		var clientsStr = (secCfg.bypass_clients || []).join(', ');

		var secCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '🔒 ', _('Network Security & Bypass Rules') ]),
				E('span', { 'class': 'os-badge os-badge-purple' }, _('nftables Kernel Rules'))
			]),
			E('div', { 'class': 'os-grid' }, [
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('BitTorrent WAN Bypass')),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-torrent', 'checked': secCfg.torrent_bypass !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Ports 6881–6889 & 51413 bypass VPN directly to WAN, preventing VPN provider bans.'))
				]),
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('Block QUIC (UDP 443)')),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-quic', 'checked': secCfg.disable_quic !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Forces TLS 1.3 TCP fallback in browsers, essential for YouTube DPI bypass in Zapret2.'))
				]),
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('Block Unauthorized DoH/DoT')),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-doh', 'checked': secCfg.block_doh !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Blocks rogue browser DoH to prevent DNS leaks and enforce router policy routing.'))
				]),
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('Exclude NTP (UDP 123)')),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-ntp', 'checked': secCfg.exclude_ntp !== false }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Direct WAN routing for time synchronization without tunnel jitter.'))
				]),
				E('div', { 'class': 'os-stat-box' }, [
					E('div', { 'style': 'display: flex; justify-content: space-between; align-items: center;' }, [
						E('span', { 'class': 'os-label' }, _('Download Updates via VPN')),
						E('label', { 'class': 'os-switch' }, [
							E('input', { 'type': 'checkbox', 'id': 'os-sec-dlproxy', 'checked': secCfg.download_via_proxy === true }),
							E('span', { 'class': 'os-slider' })
						])
					]),
					E('div', { 'class': 'os-help' }, _('Use active sing-box tunnel when downloading GitHub release packages if blocked.'))
				])
			]),
			E('div', { 'class': 'os-grid-2', 'style': 'margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--os-border);' }, [
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Excluded Subnets and IPs (CIDR list, comma-separated)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-sec-cidrs',
						'placeholder': '192.168.1.0/24, 1.1.1.1, 91.108.4.0/22',
						'value': cidrsStr
					}),
					E('div', { 'class': 'os-help' }, _('IP prefixes that are always bypassed directly to WAN without proxying.'))
				]),
				E('div', { 'class': 'os-form-group' }, [
					E('label', { 'class': 'os-label' }, _('Excluded LAN Clients (IP or MAC, comma-separated)')),
					E('input', {
						'type': 'text',
						'class': 'os-input',
						'id': 'os-sec-clients',
						'placeholder': '192.168.1.150, AA:BB:CC:DD:EE:FF',
						'value': clientsStr
					}),
					E('div', { 'class': 'os-help' }, _('Specific local devices completely excluded from all proxying.'))
				])
			])
		]);
		viewRoot.appendChild(secCard);

		// Section 3: Backup & Restore
		var bakCard = E('div', { 'class': 'os-card' }, [
			E('div', { 'class': 'os-card-header' }, [
				E('h3', { 'class': 'os-card-title' }, [ '💾 ', _('Backup & Restore') ]),
				E('span', { 'class': 'os-badge os-badge-muted' }, _('Archive .tar.gz'))
			]),
			E('p', { 'class': 'os-help', 'style': 'margin-bottom: 16px;' },
				_('Export all custom routing rules, UCI configuration, and server credentials to a portable archive, or restore on a freshly flashed router.')
			),
			E('div', { 'style': 'display: flex; gap: 12px; flex-wrap: wrap;' }, [
				E('button', {
					'class': 'os-btn os-btn-secondary',
					'click': function(ev) {
						var btn = ev.target;
						btn.disabled = true;
						btn.innerText = '⏳ ' + _('Creating archive...');
						callCreateBackup().then(function(res) {
							btn.disabled = false;
							btn.innerText = '📥 ' + _('Download Backup (.tar.gz)');
							if (res && res.data_base64) {
								var link = document.createElement('a');
								link.href = 'data:application/gzip;base64,' + res.data_base64;
								link.download = res.backup_filename || 'openstream-backup.tar.gz';
								document.body.appendChild(link);
								link.click();
								document.body.removeChild(link);
								ui.addNotification(null, E('p', {}, '✓ ' + _('Backup archive downloaded successfully!')), 'info');
							}
						}).catch(function(err) {
							btn.disabled = false;
							btn.innerText = '📥 ' + _('Download Backup (.tar.gz)');
							ui.addNotification(null, E('p', {}, '❌ ' + _('Backup error: ') + err), 'error');
						});
					}
				}, [ '📥 ', _('Download Backup (.tar.gz)') ]),
				E('label', {
					'class': 'os-btn os-btn-secondary',
					'style': 'cursor: pointer;'
				}, [
					'📤 ', _('Restore from Archive'),
					E('input', {
						'type': 'file',
						'style': 'display: none;',
						'accept': '.tar.gz,.tgz',
						'change': function(ev) {
							var file = ev.target.files[0];
							if (!file) return;
							var reader = new FileReader();
							reader.onload = function() {
								var b64 = reader.result.split(',')[1];
								ui.showModal(_('Restoring Configuration'), [
									E('p', { 'class': 'spinning' }, _('Unpacking archive and restarting services...'))
								]);
								callRestoreBackup(b64).then(function(r) {
									setTimeout(function() {
										ui.hideModal();
										ui.addNotification(null, E('p', {}, '✓ ' + (r.message || _('Configuration restored successfully!'))), 'info');
										window.location.reload();
									}, 1500);
								}).catch(function(err) {
									ui.hideModal();
									ui.addNotification(null, E('p', {}, '❌ ' + _('Restore error: ') + err), 'error');
								});
							};
							reader.readAsDataURL(file);
						}
					})
				])
			])
		]);
		viewRoot.appendChild(bakCard);

		return viewRoot;
	},

	handleSaveApply: null,
	handleSave: null,
	handleReset: null
});
