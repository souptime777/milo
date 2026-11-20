
#[path = "src/chess/board/movegen/magic_data.rs"]
mod magic_data;
use magic_data::{BISHOP_MAGICS, LOOKUP_TABLE_SIZE, ROOK_MAGICS};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/src/chess/board/movegen/magic_data.rs");
}

const EDGES_FILES: u64 = 0x8181_8181_8181_8181; // File A | File H
const EDGES_RANKS: u64 = 0xFF00_0000_0000_00FF; // Rank 1 | Rank 8

fn rook_relevant_occupancy(sq: usize) -> u64 {
    let file = 0x0101_0101_0101_0101_u64 << (sq % 8);
    let rank = 0x0000_0000_0000_00FF_u64 << ((sq / 8) * 8);

    (file & !EDGES_RANKS) | (rank & !EDGES_FILES)
}

fn bishop_relevant_occupancy(sq: usize) -> u64 {
    const DIRS: [(isize, isize); 4] = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
    let mut mask = 0_u64;
    for (df, dr) in DIRS {
        let mut f = (sq % 8) as isize + df;
        let mut r = (sq / 8) as isize + dr;
        while (1..=6).contains(&f) && (1..=6).contains(&r) {
            mask |= 1_u64 << (r * 8 + f);
            f += df;
            r += dr;
        }
    }
    mask
}

fn ray_march_attacks(blockers: u64, sq: usize, df: isize, dr: isize) -> u64 {
    let mut f = (sq % 8) as isize;
    let mut r = (sq / 8) as isize;
    let mut attacks = 0_u64;

    loop {
        f += df;
        r += dr;
        if !(0..=7).contains(&f) || !(0..=7).contains(&r) {
            break;
        }
        let bit = 1_u64 << (r * 8 + f);
        attacks |= bit;
        if (blockers & bit) != 0 {
            break;
        }
    }
    attacks
}

fn get_rook_attacks_slow(blockers: u64, sq: usize) -> u64 {
    ray_march_attacks(blockers, sq, 1, 0)
        | ray_march_attacks(blockers, sq, 0, 1)
        | ray_march_attacks(blockers, sq, -1, 0)
        | ray_march_attacks(blockers, sq, 0, -1)
}

fn get_bishop_attacks_slow(blockers: u64, sq: usize) -> u64 {
    ray_march_attacks(blockers, sq, 1, 1)
        | ray_march_attacks(blockers, sq, 1, -1)
        | ray_march_attacks(blockers, sq, -1, 1)
        | ray_march_attacks(blockers, sq, -1, -1)
}

fn enumerate_occupancies(mask: u64) -> Vec<u64> {
    let mut occupancies = Vec::new();
    let mut subset = 0_u64;
    loop {
        occupancies.push(subset);
        subset = subset.wrapping_sub(mask) & mask;
        if subset == 0 {
            break;
        }
    }
    occupancies
}