use milo_core::*;

fn main() {
    for square in Rank::Fourth.bitboard() {
        println!("{}", square)
    }
}
