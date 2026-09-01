'use strict';
'require view';
'require form';
'require uci';
'require rpc';

return view.extend({
	render: function() {
		var m, s, o;

		m = new form.Map('openstream', _('OpenStream Engine 2.1 Services Hub'),
			_('Управление сервисами, глобальными настройками и компиляцией политик маршрутизации (ucode).'));

		s = m.section(form.NamedSection, 'main', 'main', _('Глобальные параметры'));
		s.anonymous = true;

		o = s.option(form.Flag, 'enabled', _('Главный переключатель службы (OpenStream)'));
		o.default = '1';
		o.rmempty = false;

		o = s.option(form.Flag, 'expert_mode', _('Экспертный режим отладки'));
		o.default = '0';
		o.description = _('Включает расширенную диагностику и низкоуровневые метрики ядра.');

		s = m.section(form.NamedSection, 'rules', 'rules', _('Сервисные политики OpenStream 2.1 (.osrule)'));
		s.anonymous = true;
		s.description = _('Декларативные кроссплатформенные правила, компилируемые напрямую в nftables и dnsmasq без накладных расходов.');

		o = s.option(form.Flag, 'rule_twitch', _('Twitch Live Optimizer'));
		o.default = '1';
		o.description = _('Удаление SSAI рекламы через чистые токены, разблокировка 1080p60/1440p, прямое видео с CDN.');

		o = s.option(form.Flag, 'rule_youtube', _('YouTube Anti-DPI & Clean'));
		o.default = '1';
		o.description = _('Локальная десинхронизация TLS ClientHello для googlevideo CDN через Zapret2 (nfqws2) без ограничений VPN.');

		o = s.option(form.Flag, 'rule_discord', _('Discord Voice & Media'));
		o.default = '1';
		o.description = _('Обход блокировок голосовых серверов Discord через UDP-пресет Zapret2.');

		o = s.option(form.Flag, 'rule_crunchyroll', _('Crunchyroll Smart Route'));
		o.default = '1';
		o.description = _('Разблокировка библиотеки США с прямой скоростью локального провайдера.');

		o = s.option(form.Flag, 'rule_adblock', _('Privacy & AdBlock Sinkhole'));
		o.default = '1';
		o.description = _('Блокировка телеметрии, трекеров и рекламных сетей на уровне DNS (0.0.0.0).');

		return m.render();
	}
});
