// Генерация набора правил nftables для проверки синтаксиса (scripts/check-nft-syntax.sh).
//
// Путь вывода задаётся переменной окружения OPENSTREAM_NFT_OUT.
// Набор намеренно включает ВСЕ ветки генератора: bypass, zapret2, VPN/TPROXY,
// streamproxy (redirect) и блокировки QUIC/DoT (reject) — именно их сочетание
// ранее давало файл, который ядро отказывалось загружать.
use std::env;
use std::fs;

fn main() {
    let mut c = openstream_core::CompiledRuleSet::default();

    c.proxy_domains
        .push(("beta.crunchyroll.com".into(), "us_west".into()));
    c.proxy_domains
        .push(("gql.twitch.tv".into(), "vpn_adfree".into()));
    c.zapret2_domains
        .push(("googlevideo.com".into(), "youtube_4k".into()));
    c.streamproxy_domains.push("live-video.net".into());
    c.bypass_domains.push("gosuslugi.ru".into());
    c.bypass_clients.push("192.168.1.150".into());
    // Некорректное значение: обязано быть отфильтровано до попадания в файл.
    c.bypass_clients.push("; rm -rf /".into());
    c.disable_quic = true;
    c.block_doh = true;
    c.exclude_ntp = true;
    c.bypass_p2p = true;

    let nft = openstream_backend_openwrt::generate_nftables_rules(&c, "inet", "openstream");

    match env::var("OPENSTREAM_NFT_OUT") {
        Ok(path) => fs::write(&path, &nft).expect("не удалось записать файл правил"),
        Err(_) => print!("{}", nft),
    }
}
