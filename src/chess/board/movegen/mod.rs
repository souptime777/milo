mod relevant_occupancy;
mod precomputing;
mod magics;

pub use relevant_occupancy::*;
pub use precomputing::*;
use magics::*;

use crate::*;

static ROOK_TABLE: [Bitboard; ROOK_TABLE_SIZE] = {
    let mut table = [Bitboard::EMPTY; ROOK_TABLE_SIZE];

    let data = get_sq_rook_data();

    table
};

impl Board {
    /// Generate rook sliders
    pub const fn get_rook_sliders(blockers: Bitboard) {

    }
}