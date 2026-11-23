use milo_core::*;
use chess::tests::perft;

fn main() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let start = std::time::Instant::now();
    let nodes = perft(&mut board, 5);
    let elapsed = start.elapsed().as_secs_f64();
    let rate = (nodes as f64) / elapsed;
    println!("{} nodes in {} seconds ({} nps)", nodes, elapsed, rate.round());

}
