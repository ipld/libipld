use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libipld::cbor::DagCborCodec;
use libipld::cid::Cid;
use libipld::codec::Codec;
use libipld::ipld;

fn bench_codec(c: &mut Criterion) {
    c.bench_function("roundtrip", |b| {
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": Cid::random(),
        });
        b.iter(|| {
            for _ in 0..1000 {
                let bytes = DagCborCodec::encode(&ipld).unwrap();
                let ipld2 = DagCborCodec::decode(&bytes).unwrap();
                black_box(ipld2);
            }
        });
    });
}

criterion_group! {
    name = codec;
    config = Criterion::default();
    targets = bench_codec
}

criterion_main!(codec);
