use crate::*;

pub const ROOK_RELEVANT_OCCUPANCY: [Bitboard; 64] = {
    let mut relevant_occupancy = [Bitboard::EMPTY; 64];

    let mut idx = 0_u8;

    while idx < 64 {
        let sq: Square = unsafe { std::mem::transmute(idx) };

        relevant_occupancy[sq as usize] = Bitboard(
            ((sq.file().bitboard().0 & !Bitboard::EDGES_RANKS.0) 
            | (sq.rank().bitboard().0 & !Bitboard::EDGES_FILES.0))
            & !sq.bitboard().0
        );
        
        idx += 1;
    }

    relevant_occupancy
};