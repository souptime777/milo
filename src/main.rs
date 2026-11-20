use milo_core::*;

fn main() {
    let board = Board::default();
    println!("{}", board.attackers_to(Square::E4, Color::White))
}
