#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{ApiResponse, __buffa::view::ApiResponseView};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/api_response.pb");
    benchmark_decode::<ApiResponse>(c, "buffa/api_response", data);
    benchmark_json::<ApiResponse>(c, "buffa/api_response", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/api_response");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| for p in &ds.payload { criterion::black_box(ApiResponseView::decode_view(p).unwrap()); })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
