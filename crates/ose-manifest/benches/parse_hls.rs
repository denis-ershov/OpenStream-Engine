use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ose_manifest::{parse, serialize};

const MEDIA: &str = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-TARGETDURATION:2
#EXT-X-MEDIA-SEQUENCE:100
#EXTINF:2.000,live
seg100.ts
#EXTINF:2.000,live
seg101.ts
#EXTINF:2.000,
ad1.ts
#EXTINF:2.000,
ad2.ts
#EXTINF:2.000,live
seg102.ts
#EXTINF:2.000,live
seg103.ts
#EXTINF:2.000,live
seg104.ts
#EXTINF:2.000,live
seg105.ts
"#;

fn bench_parse(c: &mut Criterion) {
    c.bench_function("hls_parse_media", |b| {
        b.iter(|| parse(black_box(MEDIA)).unwrap())
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    let m = parse(MEDIA).unwrap();
    c.bench_function("hls_serialize", |b| {
        b.iter(|| serialize(black_box(&m)))
    });
}

criterion_group!(benches, bench_parse, bench_roundtrip);
criterion_main!(benches);
