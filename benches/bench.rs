use std::{fs::File, hint::black_box, io::BufReader};
use criterion::{criterion_group, criterion_main, Criterion};
use row_col_major::{Matrix, DataType, SortOrder};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("group");

    group.sample_size(30);

    group.bench_function("row-major", |b| b.iter(|| {
        let file = File::open("data/qc2534.mtx").unwrap();
        let mut rdr = BufReader::new(file);
        let mut m = Matrix::from_reader(&mut rdr, DataType::Real);
        m.sort(SortOrder::RowMajor);
        black_box(m);
    }));

    group.bench_function("col-major", |b| b.iter(|| {
        let file = File::open("data/qc2534.mtx").unwrap();
        let mut rdr = BufReader::new(file);
        let mut m = Matrix::from_reader(&mut rdr, DataType::Real);
        m.sort(SortOrder::ColMajor);
        black_box(m);
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
