#![allow(dead_code)]
include!("common.rs");
use bench_buffa::bench::{PackedTile, __buffa::view::PackedTileView};

fn run(c: &mut Criterion) {
    let data = include_bytes!("../../datasets/packed_tile.pb");
    benchmark_decode::<PackedTile>(c, "buffa/packed_tile", data);
    benchmark_json::<PackedTile>(c, "buffa/packed_tile", data);
    let ds = load_dataset(data);
    let bytes = total_payload_bytes(&ds);
    let mut g = c.benchmark_group("buffa/packed_tile");
    g.throughput(Throughput::Bytes(bytes));
    g.bench_function("decode_view", |b| {
        b.iter(|| for p in &ds.payload { criterion::black_box(PackedTileView::decode_view(p).unwrap()); })
    });
    g.finish();
}
criterion::criterion_group!(grp, run);
criterion::criterion_main!(grp);
