use criterion::{black_box, criterion_group, criterion_main, Criterion};
use traybattery::hidpp::battery::decipher_battery_unified;

fn benchmark_battery_parsing(c: &mut Criterion) {
    c.bench_function("parse valid battery", |b| {
        b.iter(|| {
            let response = black_box(vec![47, 30, 0x00]);
            decipher_battery_unified(&response)
        });
    });

    c.bench_function("reject invalid battery", |b| {
        b.iter(|| {
            let response = black_box(vec![255, 255, 0xFF]);
            decipher_battery_unified(&response)
        });
    });
}

criterion_group!(benches, benchmark_battery_parsing);
criterion_main!(benches);