use milo_core::*;
use crate::chess::board::ROOK_RELEVANT_OCCUPANCY;

fn main() {
    for sq in Square::ALL {
        println!("{}", sq);
        println!("{}", ROOK_RELEVANT_OCCUPANCY[sq as usize]);
    }
}