use std::time::Instant;
use crate::*;
pub fn nps_test(depth: u8) -> f64 {
    let mut board = Board::default();
    let start = Instant::now();
    let nodes = full_perft(&mut board, depth);
    let elapsed = start.elapsed().as_secs_f64();
    nodes as f64 / elapsed
}
pub fn perft(board: &mut Board, depth: u8) -> u64 {
    let mut nodes = 0;
    let mut moves = MoveList::new();
    board.generate_moves_into(&mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    for mv in moves {
        board.make_move(mv);
        nodes += perft(board, depth - 1);
        board.unmake_move();
    }

    nodes
}

pub fn full_perft(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1
    }
    let mut nodes = 0;
    let mut moves = MoveList::new();
    board.generate_moves_into(&mut moves);

    for mv in moves {
        board.make_move(mv);
        nodes += full_perft(board, depth - 1);
        board.unmake_move();
    }

    nodes
}

