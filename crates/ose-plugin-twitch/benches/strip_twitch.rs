use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ose_manifest::parse;
use ose_plugin::strip_ad_segments;
use ose_plugin_twitch::TwitchPlugin;
use ose_detector::default_twitch_rules;

fn bench_strip(c: &mut Criterion) {
    let raw = include_str!("../fixtures/midroll.m3u8");
    let rules = default_twitch_rules();
    c.bench_function("twitch_strip_midroll", |b| {
        b.iter(|| {
            let mut m = parse(black_box(raw)).unwrap();
            strip_ad_segments(black_box(&mut m), &rules, true).unwrap()
        })
    });
    let _ = TwitchPlugin::default();
}

criterion_group!(benches, bench_strip);
criterion_main!(benches);
