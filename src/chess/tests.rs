use crate::*;
#[test]

pub fn perft_test() {
    let expected: [u64; 7] = [
        1, 
        20,
        400,
        8_902,
        197_281,
        4_865_609,
        119_060_324,
    ];
    for (depth, expected) in expected.into_iter().enumerate() {
        let mut board = Board::default();
        assert_eq!(perft(&mut board, depth as u8), expected);
    }
    
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