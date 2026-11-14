use milo_core::*;

fn main() {
    let mut board = Board::default();
    board.make_move(&Move::new(
        Square::B8,
        Square::F6, 
        MoveFlag::Quiet));
    board.make_move(&Move::new(
        Square::C8,
        Square::C5, 
        MoveFlag::Quiet));
    board.make_move(&Move::new(
        Square::D8,
        Square::D5, 
        MoveFlag::Quiet));
    board.make_move(&Move::new(
        Square::E8,
        Square::C8,
        MoveFlag::QueenCastle));
    println!("{}", board);
    for piece in Piece::ALL {
        println!("{:?}", piece);
        println!("{}", board.piece_bitboard(piece))
    }
}