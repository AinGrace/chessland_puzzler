use chessland_puzzle_generator::pgn::read_pgns;
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read pgn", |b| b.iter(|| read_pgns("Colle.pgn")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
