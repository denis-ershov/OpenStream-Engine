'use strict';
'require view';
'require form';

return view.extend({
	render: function() {
		var m, s, o;

		m = new form.Map('openstream', _('Модуль Twitch Live Stream Optimizer'),
			_('Маршрутизация токенов и очистка видеопотоков без установки клиентских SSL-сертификатов.'));

		s = m.section(form.NamedSection, 'twitch', 'module', _('Пресеты маршрутизации Twitch'));
		s.anonymous = true;

		o = s.option(form.Flag, 'enabled', _('Включить модуль Twitch'));
		o.default = '1';
		o.rmempty = false;

		o = s.option(form.ListValue, 'preset', _('Режим работы'));
		o.value('clean_proxy_geosplit', _('🛡️ Geo-Split через Clean-Proxy (Zero-CA для SmartTV/ПК) [Рекомендуется]'));
		o.value('manifest_strip_edge', _('⚡ Playlist Edge: Локальное удаление рекламы в streamproxyd (:18080)'));
		o.value('smartdns_quality_unlock', _('🌍 Quality Unlock: Разблокировка 1080p60 через SmartDNS'));
		o.value('custom', _('⚙️ Custom: Матричная раздельная маршрутизация'));
		o.value('off', _('⏹️ Отключено'));
		o.default = 'clean_proxy_geosplit';
		o.description = _('Geo-Split запрашивает авторизационный токен в стране без рекламы, а видео транслирует с прямого CDN провайдера.');

		// Custom options
		o = s.option(form.ListValue, 'route_token', _('Маршрут токена (gql.twitch.tv)'));
		o.depends('preset', 'custom');
		o.value('vpn_adfree', _('Ad-Free VPN / Clean-Proxy (UA/AL/KZ)'));
		o.value('vpn_eu', _('Основной VPN (Европа)'));
		o.value('direct', _('Прямой интернет (WAN)'));
		o.default = 'vpn_adfree';

		o = s.option(form.ListValue, 'route_master', _('Мастер-плейлист (usher.ttvnw.net)'));
		o.depends('preset', 'custom');
		o.value('smartdns_comss', _('SmartDNS Comss.one (1080p60)'));
		o.value('vpn_eu', _('Основной VPN (Европа)'));
		o.value('direct', _('Прямой интернет (WAN)'));
		o.default = 'smartdns_comss';

		o = s.option(form.ListValue, 'route_segments', _('Видеопоток CDN (live-video.net)'));
		o.depends('preset', 'custom');
		o.value('direct', _('Прямой интернет WAN (Максимальная скорость)'));
		o.value('vpn_eu', _('Основной VPN'));
		o.default = 'direct';

		return m.render();
	}
});
