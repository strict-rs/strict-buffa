#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{MediaFrame, MediaFrameView};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/media_frame.pb");
    benchmark_decode::<MediaFrame>(c, "buffa/media_frame", data);
    benchmark_json::<MediaFrame>(c, "buffa/media_frame", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/media_frame");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| {
            for p in &ds.payload {
                criterion::black_box(MediaFrameView::decode_view(p).unwrap());
            }
        })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
