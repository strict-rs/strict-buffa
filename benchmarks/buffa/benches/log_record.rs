#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{LogRecord, LogRecordView};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/log_record.pb");
    benchmark_decode::<LogRecord>(c, "buffa/log_record", data);
    benchmark_json::<LogRecord>(c, "buffa/log_record", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/log_record");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| {
            for p in &ds.payload {
                criterion::black_box(LogRecordView::decode_view(p).unwrap());
            }
        })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
