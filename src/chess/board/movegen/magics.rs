use crate::*;



fn ray_march_attacks(blockers: Bitboard, sq: Square, dir: (isize, isize)) -> Bitboard {
    let mut f = sq.file() as isize;
    let mut r = sq.rank() as isize;

    let mut bb = Bitboard::EMPTY;

    loop {
        f += dir.0;
        r += dir.1;

        if !(0..=7).contains(&f) || !(0..=7).contains(&r) {
            break;
        }

        bb |= Bitboard(1 << (r * 8 + f));

        if !(bb & blockers).is_empty() || !(1..=6).contains(&f) || !(1..=6).contains(&r) {
            break;
        }
    }
    bb
}

pub fn get_rook_attacks_slow(blockers: Bitboard, sq: Square) -> Bitboard {
    const ROOK_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

    let mut bb = Bitboard::EMPTY;

    for dir in ROOK_DIRECTIONS {
        bb |= ray_march_attacks(blockers, sq, dir);
    }

    bb
}

pub fn get_bishop_attacks_slow(blockers: Bitboard, sq: Square) -> Bitboard {
    const BISHOP_DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    let mut bb = Bitboard::EMPTY;

    for dir in BISHOP_DIRECTIONS {
        bb |= ray_march_attacks(blockers, sq, dir);
    }

    bb
}

pub fn enumerate_occupancies(mask: Bitboard) -> Vec<Bitboard> {
    let mut occupancies = Vec::new();
    let mut subset = 0_u64;

    loop {
        occupancies.push(Bitboard(subset));

        subset = subset.wrapping_sub(mask.0) & mask.0;

        if subset == 0 {
            break;
        }
    }
    occupancies
}
