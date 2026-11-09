use milo_core::*;

fn main() {
    let board = Board::from_fen("rnbqk1nr/2ppbp1p/pp4p1/1B2p3/4P1Q1/2N5/PPPP1PPP/R1B1K1NR w KQkq - 0 1").unwrap();
    println!("{}", board);
    println!("{}", board.to_fen());
}
