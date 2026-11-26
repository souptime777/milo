use milo_core::{*, tests::*};

fn main() {
    let mut board = Board::default();
    let start = std::time::Instant::now();
    let nodes = full_perft(&mut board, 6);
    let elapsed = start.elapsed().as_secs_f64();
    let rate = nodes as f64 / elapsed;
    println!("{} nodes in {:.2}s ({}nps) ", nodes, elapsed, rate);
}
