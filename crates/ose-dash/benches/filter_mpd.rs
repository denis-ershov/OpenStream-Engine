use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ose_dash::{filter_ad_nodes, DashFilterRules, Mpd};

const MPD: &str = r#"<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" type="static">
  <Period id="content_0">
    <AdaptationSet contentType="video" mimeType="video/mp4">
      <Representation id="v1" bandwidth="1000000"/>
    </AdaptationSet>
  </Period>
  <Period id="midroll_ad_1">
    <AssetIdentifier schemeIdUri="urn:scte:dash:ad" value="break"/>
    <AdaptationSet contentType="video">
      <Representation id="ad" bandwidth="500000"/>
    </AdaptationSet>
  </Period>
  <Period id="content_1">
    <AdaptationSet contentType="video">
      <Representation id="v2" bandwidth="1000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#;

fn bench_filter(c: &mut Criterion) {
    let rules = DashFilterRules::default();
    c.bench_function("dash_filter_ad_period", |b| {
        b.iter(|| {
            let mut mpd = Mpd::parse(black_box(MPD)).unwrap();
            filter_ad_nodes(black_box(&mut mpd), &rules)
        })
    });
}

criterion_group!(benches, bench_filter);
criterion_main!(benches);
