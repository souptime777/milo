mod eval;
mod search;

use cozy_chess::Board;
use search::*;

fn main() {
    let mut board = Board::default();
    let mv = Engine::iterative_search(board.clone(), 7, 1024, true);
    println!("{}", mv);
    board.play_unchecked(mv);
}
