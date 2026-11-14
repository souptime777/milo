use milo_core::*;

fn main() {
    let mut board = Board::default();
    board.make_move(&Move::new(
        Square::E2,
        Square::E4, 
        MoveFlag::Quiet));
    println!("{}", board)
}