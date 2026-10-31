mod eval;
mod search;
mod uci;

use std::io;
use std::thread::sleep;

use uci::handle_uci_cmd;
use uci_parser::UciCommand;

use cozy_chess::{Board, util::display_uci_move};

use crate::search::PositionHistory;


fn main() {
    let mut board = Board::default();
    let mut position_history = PositionHistory::new();

    loop {
        let mut input = String::new();

        // 2. Read the line from stdin
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line"); // Handles potential I/O errors

        let cmd = input.parse::<UciCommand>().unwrap();
        let result = handle_uci_cmd(cmd, &mut board, false, &mut position_history);

        if let Some(search) = result {
            sleep(search.duration);
            search
                .stop_flag
                .store(true, std::sync::atomic::Ordering::Relaxed);
            println!(
                "bestmove {}",
                display_uci_move(&board, search.handle.join().unwrap())
            );
        }
    }
}
