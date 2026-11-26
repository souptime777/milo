use milo_core::{*, tests::*};

fn main() {
    let mv = Move::new(
        Square::E5,
        Square::D6,
        MoveFlag::EnPassant,
    );
    println!("{}", mv.is_capture())
}
