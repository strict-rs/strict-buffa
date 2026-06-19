#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{GoogleMessage1, __buffa::view::GoogleMessage1View};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/google_message1_proto3.pb");
    benchmark_decode::<GoogleMessage1>(c, "buffa/google_message1_proto3", data);
    benchmark_json::<GoogleMessage1>(c, "buffa/google_message1_proto3", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/google_message1_proto3");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| for p in &ds.payload { criterion::black_box(GoogleMessage1View::decode_view(p).unwrap()); })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
