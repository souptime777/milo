use crate::*;

pub const ROOK_RELEVANT_OCCUPANCY: [Bitboard; 64] = {
    let mut relevant_occupancy = [Bitboard::EMPTY; 64];

    let mut idx = 0_u8;

    while idx < 64 {
        let sq = Square::from_index(idx);

        relevant_occupancy[sq as usize] = Bitboard(
            ((sq.file().bitboard().0 & !Bitboard::EDGES_RANKS.0) 
            | (sq.rank().bitboard().0 & !Bitboard::EDGES_FILES.0))
            & !sq.bitboard().0
        );
        
        idx += 1;
    }

    relevant_occupancy
};

pub const BISHOP_RELEVANT_OCCUPANCY: [Bitboard; 64] = {
    const BISHOP_DIRECTIONS: [(isize, isize); 4] = [
        (1, 1),
        (-1, 1),
        (-1, -1),
        (1, -1),
    ];
    let mut relevant_occupancy = [Bitboard::EMPTY; 64];

    let mut idx = 0_u8;

    while idx < 64 {
        let sq = Square::from_index(idx);

        let mut inner_num = 0_u64;

        let mut dir_idx = 0;

        while dir_idx < 4 {
            let mut file_idx = sq.file() as isize;
            let mut rank_idx = sq.rank() as isize;
            loop {
                file_idx += BISHOP_DIRECTIONS[dir_idx].0;
                rank_idx += BISHOP_DIRECTIONS[dir_idx].1;

                if file_idx < 1 || file_idx > 6
                || rank_idx < 1 || rank_idx > 6 {
                    break;
                }
                
                inner_num |= 1 << Square::new(
                    File::ALL[file_idx as usize], 
                    Rank::ALL[rank_idx as usize],
                ) as u64;
            }
            dir_idx += 1;
        }
            relevant_occupancy[sq as usize] = Bitboard(inner_num);

        idx += 1;
    }

    relevant_occupancy
};
