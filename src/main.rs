mod eval;
mod search;

use cozy_chess::{Board, Move};
use eval::evaluate;
use search::*;
use uci_parser::{UciCommand, UciInfo, UciOption, UciResponse};
use std::{io, time::Instant};

fn main() {
    let mut board = Board::default();
    let mv = Engine::iterative_search(board.clone(), 7, 1024, true);
    println!("{}", mv);
    board.play_unchecked(mv);
}
