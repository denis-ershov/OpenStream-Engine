use std::path::PathBuf;
use openstream_core::{EgressGateway, PolicyEngine, RoutingVerdict};
use openstream_rule::load_rule_file;

fn get_rules_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().join("rules")
}

#[test]
fn test_load_all_reference_rules() {
    let rules_dir = get_rules_dir();
    let engine = PolicyEngine::new();

    // Шлюзы
    engine.update_gateways(vec![
        EgressGateway {
            id: "gw_albania".into(),
            tag: "geo:al".into(),
            protocol: "wireguard".into(),
            endpoint: "1.1.1.1:51820".into(),
            healthy: true,
        },
        EgressGateway {
            id: "gw_us_west".into(),
            tag: "proxy:us_west".into(),
            protocol: "vless".into(),
            endpoint: "2.2.2.2:443".into(),
            healthy: true,
        },
    ]);

    // 1. Twitch Rule
    let twitch_rule = load_rule_file(rules_dir.join("streaming").join("twitch.osrule.yaml"))
        .expect("Twitch rule должен загрузиться без ошибок");
    assert_eq!(twitch_rule.id, "org.openstream.rules.twitch");
    engine.add_rule(twitch_rule);

    // 2. Crunchyroll Rule
    let crunchy_rule = load_rule_file(rules_dir.join("streaming").join("crunchyroll.osrule.yaml"))
        .expect("Crunchyroll rule должен загрузиться без ошибок");
    assert_eq!(crunchy_rule.id, "org.openstream.rules.crunchyroll");
    engine.add_rule(crunchy_rule);

    // 3. YouTube Rule
    let yt_rule = load_rule_file(rules_dir.join("streaming").join("youtube.osrule.yaml"))
        .expect("YouTube rule должен загрузиться без ошибок");
    assert_eq!(yt_rule.id, "org.openstream.rules.youtube");
    engine.add_rule(yt_rule);

    // 4. Adblock Rule
    let adblock_rule = load_rule_file(rules_dir.join("privacy").join("adblock.osrule.yaml"))
        .expect("Adblock rule должен загрузиться без ошибок");
    assert_eq!(adblock_rule.id, "org.openstream.rules.adblock");
    engine.add_rule(adblock_rule);

    // Проверка маршрутизации Twitch:
    // gql.twitch.tv -> geo:al -> Proxy { gateway_id: "gw_albania" }
    assert_eq!(
        engine.resolve_domain("gql.twitch.tv", None),
        RoutingVerdict::Proxy {
            gateway_id: "gw_albania".into()
        }
    );
    // usher.ttvnw.net -> quality_unlock -> fallback to direct (нет geo:de/smartdns:eu)
    assert_eq!(
        engine.resolve_domain("usher.ttvnw.net", None),
        RoutingVerdict::Direct
    );
    // video cdn -> direct
    assert_eq!(
        engine.resolve_domain("video-edge-fra.live-video.net", None),
        RoutingVerdict::Direct
    );
    // ads -> block
    assert_eq!(
        engine.resolve_domain("edge.ads.twitch.tv", None),
        RoutingVerdict::Block { reason: None }
    );

    // Проверка маршрутизации Crunchyroll:
    assert_eq!(
        engine.resolve_domain("beta-api.crunchyroll.com", None),
        RoutingVerdict::Proxy {
            gateway_id: "gw_us_west".into()
        }
    );
    assert_eq!(
        engine.resolve_domain("v.vrv.co", None),
        RoutingVerdict::Direct
    );

    // Проверка YouTube:
    // googlevideo -> dpi_evasive_direct
    assert_eq!(
        engine.resolve_domain("rr1---sn-abc.googlevideo.com", None),
        RoutingVerdict::DpiEvasiveDirect
    );
    // ads -> block
    assert_eq!(
        engine.resolve_domain("youtubeads.g.doubleclick.net", None),
        RoutingVerdict::Block { reason: None }
    );

    // Проверка Adblock:
    assert_eq!(
        engine.resolve_domain("mc.yandex.ru", None),
        RoutingVerdict::Block { reason: None }
    );
    assert_eq!(
        engine.resolve_domain("app-measurement.com", None),
        RoutingVerdict::Block { reason: None }
    );

    // Любой не перехваченный домен -> Direct
    assert_eq!(
        engine.resolve_domain("github.com", None),
        RoutingVerdict::Direct
    );
}
