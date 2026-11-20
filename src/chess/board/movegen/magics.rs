use crate::*;

use super::magic_data::*;

const TOTAL_ROOK_BYTES: usize = ROOK_TABLE_SIZE * std::mem::size_of::<Bitboard>();
const TOTAL_BISHOP_BYTES: usize = BISHOP_TABLE_SIZE * std::mem::size_of::<Bitboard>();

#[repr(C, align(8))]
struct AlignedRookLut([u8; TOTAL_ROOK_BYTES]);
struct AlignedBishopLut([u8; TOTAL_BISHOP_BYTES]);

#[inline]
const fn sq_to_file_rank(sq: usize) -> (isize, isize) {
    let file = (sq % 8) as isize;
    let rank = 7 - (sq / 8) as isize;
    (file, rank)
}

#[inline]
const fn file_rank_to_sq(file: isize, rank: isize) -> usize {
    ((7 - rank) * 8 + file) as usize
}

const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0;

    const OFFSETS: [(isize, isize); 8] = [
        (1, 2),
        (2, 1),
        (-1, 2),
        (-2, 1),
        (-1, -2),
        (-2, -1),
        (1, -2),
        (2, -1)
    ];

    while sq < 64 {
        let (f, r) = sq_to_file_rank(sq);

        let mut offset_idx: usize = 0;
        while offset_idx < 8 {
            let offset = OFFSETS[offset_idx];

            if f + offset.0 <= 7
            && f + offset.0 >= 0
            && r + offset.1 <= 7
            && r + offset.1 >= 0 {
                attacks[sq].0 |= 1 << file_rank_to_sq(f + offset.0, r + offset.1) as u64;
            }
            offset_idx += 1;
        }
        sq += 1;
    }

    attacks
};

const KING_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0;

    const OFFSETS: [(isize, isize); 8] = [
        (1, 1),
        (1, 0),
        (1, -1),
        (0, 1),
        (0, -1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];

    while sq < 64 {
        let (f, r) = sq_to_file_rank(sq);

        let mut offset_idx: usize = 0;
        while offset_idx < 8 {
            let offset = OFFSETS[offset_idx];

            if f + offset.0 <= 7
            && f + offset.0 >= 0
            && r + offset.1 <= 7
            && r + offset.1 >= 0 {
                attacks[sq].0 |= 1 << file_rank_to_sq(f + offset.0, r + offset.1) as u64;
            }
            offset_idx += 1;
        }
        sq += 1;
    }

    attacks
};

const PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];
    let mut sq = 0;

    const WHITE_OFFSETS: [[(isize, isize); 2]; 2] = [
        [
            (1, 1),
            (-1, 1),
        ],
        [
            (1, -1),
            (-1, -1),
        ]
    ];

    while sq < 64 {
        let (f, r) = sq_to_file_rank(sq);

        let mut color_idx = 0;
        while color_idx < 2 {
            let mut offset_idx: usize = 0;
            while offset_idx < 2 {
                let offset = WHITE_OFFSETS[color_idx][offset_idx];

                if f + offset.0 <= 7
                && f + offset.0 >= 0
                && r + offset.1 <= 7
                && r + offset.1 >= 0 {
                    attacks[color_idx][sq].0 |= 1 << file_rank_to_sq(f + offset.0, r + offset.1) as u64;
                }
                offset_idx += 1;
            }
            color_idx += 1;
        }
        sq += 1;
    }

    attacks
};

static RAW_ROOK_LUT: AlignedRookLut = AlignedRookLut(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/rook_lut.bin"
)));

static RAW_BISHOP_LUT: AlignedBishopLut = AlignedBishopLut(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/bishop_lut.bin"
)));

#[inline(always)]
pub fn rook_lookup_table() -> &'static [Bitboard; ROOK_TABLE_SIZE] {

    // Compile time checks
    const _: () = assert!(std::mem::size_of::<Bitboard>() == 8);
    const _: () = assert!(std::mem::align_of::<Bitboard>() == 8);

    unsafe { &*(RAW_ROOK_LUT.0.as_ptr() as *const [Bitboard; ROOK_TABLE_SIZE])}
}

#[inline(always)]
pub fn bishop_lookup_table() -> &'static [Bitboard; BISHOP_TABLE_SIZE] {

    // Compile time checks
    const _: () = assert!(std::mem::size_of::<Bitboard>() == 8);
    const _: () = assert!(std::mem::align_of::<Bitboard>() == 8);

    unsafe { &*(RAW_BISHOP_LUT.0.as_ptr() as *const [Bitboard; BISHOP_TABLE_SIZE])}
}

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

#[inline(always)]
pub fn get_rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard{
    let idx = sq as usize;
    let blockers = occupancy.0 & ROOK_RELEVANT_OCCUPANCY[idx].0;
    let magic = ROOK_MAGICS[idx];

    let table_idx = ((blockers.wrapping_mul(magic.magic)) >> magic.shift) as usize + magic.offset as usize;

    unsafe {*rook_lookup_table().get_unchecked(table_idx)}
}

#[inline(always)]
pub fn get_bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard{
    let idx = sq as usize;
    let blockers = occupancy.0 & BISHOP_RELEVANT_OCCUPANCY[idx].0;
    let magic = BISHOP_MAGICS[idx];

    let table_idx = ((blockers.wrapping_mul(magic.magic)) >> magic.shift) as usize + magic.offset as usize;

    unsafe {*bishop_lookup_table().get_unchecked(table_idx)}
}

#[inline(always)]
pub fn get_queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    get_rook_attacks(sq, occupancy) | get_bishop_attacks(sq, occupancy)
}

#[inline(always)]
pub fn get_knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq as usize]
}

#[inline(always)]
pub fn get_king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq as usize]
}

#[inline(always)]
pub fn get_pawn_attacks(sq: Square, color: Color) -> Bitboard {
    PAWN_ATTACKS[color as usize][sq as usize]
}