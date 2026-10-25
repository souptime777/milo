mod eval;
mod search;

use cozy_chess::{Board, GameStatus};
use search::*;

fn main() {
    let mut board = Board::from_fen("4r1k1/pR4bp/3N2p1/3b4/3ppp2/1P5P/5PP1/5RK1 w - - 1 2", true).unwrap();


    let mv = Engine::iterative_search(board.clone(), 7, 4096, true);
    println!("{}", mv);
    board.play_unchecked(mv);

    println!("{}", board);
}
