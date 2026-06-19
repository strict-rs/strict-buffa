#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{AnalyticsEvent, AnalyticsEventView};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/analytics_event.pb");
    benchmark_decode::<AnalyticsEvent>(c, "buffa/analytics_event", data);
    benchmark_json::<AnalyticsEvent>(c, "buffa/analytics_event", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/analytics_event");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| {
            for p in &ds.payload {
                criterion::black_box(AnalyticsEventView::decode_view(p).unwrap());
            }
        })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
