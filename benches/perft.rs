use milo_core::{Board, perft::perft};
use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

fn bench_perft(c: &mut Criterion) {
    let positions = [
        ("startpos", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5),
        ("kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/2pP4/1p2P3/2N2N2/PPPBBPPP/R2Q1RK1 w kq - 0 1", 4),
    ];

    let mut group = c.benchmark_group("perft");

    for (name, fen, depth) in positions {
        let board = Board::from_fen(fen).unwrap();
        let nodes = perft(&mut board.clone(), depth);

        group.throughput(Throughput::Elements(nodes));

        group.bench_with_input(
            BenchmarkId::new(name, depth),
            &board,
            |b, board| {
                b.iter(|| {
                    let mut board = black_box(board.clone());
                    black_box(perft(&mut board, depth));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_perft);
criterion_main!(benches);
