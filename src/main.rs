mod eval;
mod search;
mod uci;

use uci::handle_uci_cmd;
use uci_parser::{UciCommand, UciSearchOptions};
use std::thread::available_parallelism;

use cozy_chess::{Board, GameStatus};
use search::*;


fn main() {
    let mut board = Board::from_fen("rn2kbR1/ppp1p3/8/3P4/5q2/2N4b/PPP4P/R2QK3 w Q - 0 2", false).unwrap();
    let cmd = "go depth 7".parse::<UciCommand>().unwrap();
    let search = handle_uci_cmd(cmd, &mut board, false);
    println!("bestmove {}", search.unwrap().handle.join().unwrap());
    Engine::iterative_search(board, 7, 4096, true);
}
