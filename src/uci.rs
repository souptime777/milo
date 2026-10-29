use std::{sync::{Arc, atomic::AtomicBool}, thread};

use cozy_chess::{Board, Color, Move, util::parse_uci_move};
use uci_parser::{UciCommand, UciSearchOptions};
use std::time::{Instant, Duration};

use crate::search::Engine;

pub struct ActiveSearch {
    pub stop_flag: Arc<AtomicBool>,
    pub handle: thread::JoinHandle<Move>,
    pub start: Instant,
    pub duration: Duration,
}

#[inline]
fn calculate_movetime(
    options: UciSearchOptions,
    color: Color,
) -> Duration {
    if options.movetime.is_some() {
        options.movetime.unwrap()
    } else if 
    options.wtime.is_some()
    && options.btime.is_some()
    && options.winc.is_some()
    && options.binc.is_some() {
        // Actual calculation

        let time = match color {
            Color::White => options.wtime.unwrap().as_millis(),
            Color::Black => options.btime.unwrap().as_millis(),
        } as u64;

        let inc = match color {
            Color::White => options.winc.unwrap().as_millis(),
            Color::Black => options.binc.unwrap().as_millis(),
        } as u64;
        
        Duration::from_millis(
            std::cmp::min(
                time / 30 + inc / 2,
                50,
            )
        )
        

    } else {
        Duration::from_millis(5000)
    }
    
}

pub fn handle_uci_cmd(cmd: UciCommand, board: &mut Board, search_active: bool) -> Option<ActiveSearch> {
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
        }

        UciCommand::Position { fen, moves } => {
            if let Some(position_fen) = fen {
                *board = Board::from_fen(&position_fen, false).expect("invalid fen string");
            } else {
                *board = Board::default();
            }

            for mv in moves {
                let mv = parse_uci_move(board, &mv).expect("invalid uci string");
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

            let worker_flag = Arc::clone(&stop_flag);
            let search_thread_handle = thread::spawn(move || {
                Engine::iterative_search(b, depth, 2usize.pow(21), true, worker_flag)
            });

            return Some(ActiveSearch {
                stop_flag,
                handle: search_thread_handle,
                start: Instant::now(),
                duration: movetime,
            })
        }
        
        _ => {},
    }
    None
}