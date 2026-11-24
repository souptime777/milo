use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

#[path = "src/chess/board/movegen/magic_data.rs"]
mod magic_data;
use magic_data::*;

const EDGES_FILES: u64 = 0x8181_8181_8181_8181; // File A | File H
const EDGES_RANKS: u64 = 0xFF00_0000_0000_00FF; // Rank 1 | Rank 8


fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/chess/board/movegen/magic_data.rs");
    println!("cargo::rerun-if-env-changed=RUSTFLAGS");
    println!("cargo::rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    let mut rook_table = vec![0_u64; ROOK_TABLE_SIZE];
    let mut bishop_table = vec![0_u64; BISHOP_TABLE_SIZE];

    let use_bmi2 = is_bmi2_enabled();

    for sq in 0..64 {
        // 1. Populate Rook attacks
        let rook_mask = rook_relevant_occupancy(sq);

        for blockers in enumerate_occupancies(rook_mask) {
            let idx = rook_idx(blockers, sq, use_bmi2);
            rook_table[idx] = get_rook_attacks_slow(blockers, sq);
        }

        // 2. Populate Bishop attacks
        let bishop_mask = bishop_relevant_occupancy(sq);

        for blockers in enumerate_occupancies(bishop_mask) {
            let idx = bishop_idx(blockers, sq, use_bmi2);
            bishop_table[idx] = get_bishop_attacks_slow(blockers, sq);
        }
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let rook_path = Path::new(&out_dir).join("rook_lut.bin");
    let mut file = BufWriter::new(File::create(rook_path).unwrap());

    for entry in rook_table {
        file.write_all(&entry.to_ne_bytes()).unwrap();
    }
    file.flush().unwrap();

    let bishop_path = Path::new(&out_dir).join("bishop_lut.bin");
    let mut file = BufWriter::new(File::create(bishop_path).unwrap());

    for entry in bishop_table {
        file.write_all(&entry.to_ne_bytes()).unwrap();
    }
    file.flush().unwrap();
}

// Software PEXT, slow but works regardless of system.
const fn pext_u64_soft(val: u64, mut mask: u64) -> u64 {
    let mut res = 0;
    let mut bb = 1;
    while mask != 0 {
        if val & mask & mask.wrapping_neg() != 0 {
            res |= bb;
        }
        mask &= mask - 1;
        bb <<= 1;
    }
    res
}

fn is_bmi2_enabled() -> bool {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    arch == "x86_64" && features.split(',').any(|f| f == "bmi2")
}

#[inline]
fn rook_idx(relevant_blockers: u64, sq: usize, use_bmi2: bool) -> usize {
    if use_bmi2 {
        let idx = pext_u64_soft(relevant_blockers, rook_relevant_occupancy(sq)) as usize;
        ROOK_MAGICS[sq].offset + idx
    } else {
        let m = &ROOK_MAGICS[sq];
        let idx = ((relevant_blockers).wrapping_mul(m.magic) >> m.shift) as usize;
        m.offset + idx
    }
}

#[inline]
fn bishop_idx(relevant_blockers: u64, sq: usize, use_bmi2: bool) -> usize {
    if use_bmi2 {
        let idx = pext_u64_soft(relevant_blockers, bishop_relevant_occupancy(sq)) as usize;
        BISHOP_MAGICS[sq].offset + idx
    } else {
        let m = &BISHOP_MAGICS[sq];
        let idx = ((relevant_blockers).wrapping_mul(m.magic) >> m.shift) as usize;
        m.offset + idx
    }
}

#[inline]
fn sq_to_file_rank(sq: usize) -> (isize, isize) {
    let file = (sq % 8) as isize;
    let rank = 7 - (sq / 8) as isize;
    (file, rank)
}

#[inline]
fn file_rank_to_sq(file: isize, rank: isize) -> usize {
    ((7 - rank) * 8 + file) as usize
}

fn rook_relevant_occupancy(sq: usize) -> u64 {
    let file_bb = 0x0101_0101_0101_0101_u64 << (sq % 8);
    let rank_bb = 0x0000_0000_0000_00FF_u64 << ((sq / 8) * 8);
    ((file_bb & !EDGES_RANKS) | (rank_bb & !EDGES_FILES)) & !(1_u64 << sq)
}

fn bishop_relevant_occupancy(sq: usize) -> u64 {
    const DIRS: [(isize, isize); 4] = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
    let (start_f, start_r) = sq_to_file_rank(sq);
    let mut mask = 0_u64;

    for (df, dr) in DIRS {
        let mut f = start_f + df;
        let mut r = start_r + dr;
        while (1..=6).contains(&f) && (1..=6).contains(&r) {
            mask |= 1_u64 << file_rank_to_sq(f, r);
            f += df;
            r += dr;
        }
    }
    mask
}

fn ray_march_attacks(blockers: u64, sq: usize, df: isize, dr: isize) -> u64 {
    let (mut f, mut r) = sq_to_file_rank(sq);
    let mut attacks = 0_u64;

    loop {
        f += df;
        r += dr;
        if !(0..=7).contains(&f) || !(0..=7).contains(&r) {
            break;
        }
        let bit = 1_u64 << file_rank_to_sq(f, r);
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
