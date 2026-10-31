use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};

use cozy_chess::{Board, Color, Move, util::parse_uci_move};
use std::time::{Duration, Instant};
use uci_parser::{UciCommand, UciSearchOptions};

use crate::search::{Engine, PositionHistory};

pub struct ActiveSearch {
    pub stop_flag: Arc<AtomicBool>,
    pub handle: thread::JoinHandle<Move>,
    pub start: Instant,
    pub duration: Duration,
}

#[inline]
fn calculate_movetime(options: UciSearchOptions, color: Color) -> Duration {
    if let Some(movetime) = options.movetime {
        movetime
    } else if let Some(wtime) = options.wtime
        && let Some(btime) = options.btime
    {
        // Actual calculation
        let time = match color {
            Color::White => wtime.as_millis(),
            Color::Black => btime.as_millis(),
        } as u64;

        let inc = match color {
            Color::White => if let Some(winc) = options.winc {winc.as_millis()} else {0},
            Color::Black => if let Some(binc) = options.binc {binc.as_millis()} else {0},
        } as u64;

        Duration::from_millis(std::cmp::min(time / 30 + inc / 2, 15000))
    } else {
        Duration::from_millis(5000)
    }
}

pub fn handle_uci_cmd(
    cmd: UciCommand,
    board: &mut Board,
    search_active: bool,
    position_history: &mut PositionHistory,
) -> Option<ActiveSearch> {
    match cmd {
        UciCommand::Uci => {
            println!("id name Milo");
            println!("id author souptime777");
            println!("uciok");
        }

        UciCommand::IsReady => {
            println!("readyok");
        }

        UciCommand::UciNewGame => {
            *board = Board::default();
            *position_history = PositionHistory::new();
            
        }

        UciCommand::Position { fen, moves } => {
            if let Some(position_fen) = fen {
                *board = Board::from_fen(&position_fen, false).expect("invalid fen string");
            } else {
                *board = Board::default();
            }
            position_history.push(board, None);

            for mv in moves {
                let mv = parse_uci_move(board, &mv).expect("invalid uci string");
                position_history.push(board, Some(&mv));
                board.try_play(mv).expect("illegal uci move");
            }
        }

        UciCommand::Go(search_options) => {
            // Return early if there is already an ongoing search

            if search_active {
                return None;
            }
            let depth: u8;
            if let Some(d) = search_options.depth {
                depth = d as u8;
            } else {
                depth = u8::MAX;
            };

            let movetime = calculate_movetime(search_options, board.side_to_move());
            let b = board.clone();
            let stop_flag = Arc::new(AtomicBool::new(false));
            let hist = position_history.clone();

            let worker_flag = Arc::clone(&stop_flag);
            let search_thread_handle = thread::spawn(move || {
                Engine::iterative_search(b, depth, 2usize.pow(21), true, hist, worker_flag)
            });

            return Some(ActiveSearch {
                stop_flag,
                handle: search_thread_handle,
                start: Instant::now(),
                duration: movetime,
            });
        }

        _ => {}
    }
    None
}
