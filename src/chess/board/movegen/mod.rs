mod magics;
mod magic_data;

use crate::*;

pub use magics::*;
#[derive(Debug)]
pub struct MoveGenerator {
    occupancy: Bitboard,
    piece_bitboards: [Bitboard; 12],
    color_bitboards: [Bitboard; 2],
}

impl Board {
    pub fn generate_moves() -> MoveGenerator {
        todo!()
    }
}