use crate::*;

use rand::{rng, RngExt};
use std::fs::File;
use std::io::{self, BufWriter, Write};

#[derive(Debug)]
pub struct Magic {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

pub fn generate_magic_file() -> io::Result<()> {
    let rook_magics = generate_rook_magics();
    let bishop_magics = generate_bishop_magics();

    let file = File::create("src/chess/board/movegen/magics.rs")?;
    let mut writer = BufWriter::new(file);
    
    let mut table_size: usize = 0;
    writeln!(writer, "use crate::*;\n")?;
    writeln!(writer, "pub const ROOK_MAGICS: [Magic; 64] = [")?;
    for magic in rook_magics {
        writeln!(writer, "    {:?},", magic)?;
        table_size += magic.offset;
    }
    writeln!(writer, "];")?;
    writeln!(writer, "pub const ROOK_TABLE_SIZE: usize = {};", table_size)?;

    let mut table_size: usize = 0;
    writeln!(writer)?;
    writeln!(writer, "pub const BISHOP_MAGICS: [Magic; 64] = [")?;
    for magic in bishop_magics {
        writeln!(writer, "    {:?},", magic)?;
        table_size += magic.offset;
    }
    writeln!(writer, "];")?;
    writeln!(writer, "pub const BISHOP_TABLE_SIZE: usize = {};", table_size)?;

    Ok(())
}

fn generate_rook_magics() -> [Magic; 64] {
    let rook_data = get_sq_rook_data();
    let mut data_iter = tqdm::tqdm(rook_data.iter());
    
    std::array::from_fn(|_| {
        if let Some(data) = data_iter.next() {
            find_magic(data)
        } else {
            panic!();
        }
    })
}

fn generate_bishop_magics() -> [Magic; 64] {
    let bishop_data = get_sq_bishop_data();
    let mut data_iter = tqdm::tqdm(bishop_data.iter());
    
    std::array::from_fn(|_| {
        if let Some(data) = data_iter.next() {
            find_magic(data)
        } else {
            panic!();
        }
    })
}

fn find_magic(data: &SquareMoveData) -> Magic {
    let mut rng = rng();

    let relevant_bits = data.mask.0.count_ones() as u8;
    let shift = 64 - relevant_bits;
    let table_size = 1_usize << relevant_bits;

    loop {
        let magic = rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>();

        // Quality Heuristic
        if (data.mask.0.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
            continue;
        }

        let mut used: Vec<Option<Bitboard>> = vec![None; table_size];
        let mut valid = true;

        for (&occupancy, &attack) in data.possible_blockers.iter().zip(&data.moves) {
            let idx = (
                occupancy.0.wrapping_mul(magic) >> shift
            ) as usize;
            match used[idx] {
                Some(existing) => {
                    if existing != attack {
                        valid = false;
                        break;
                    }
                }
                None => {
                    used[idx] = Some(attack);
                }
            }
        }
        
        if valid {
            return Magic { mask: data.mask, magic, shift, offset: table_size};
        }
    }
}

fn ray_march_sliders(blockers: Bitboard, sq: Square, dir: (isize, isize)) -> Bitboard {
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

        if !(bb & blockers).is_empty()
        || !(1..=6).contains(&f) 
        || !(1..=6).contains(&r) {
            break;
        }
    }
    bb
}

pub fn get_rook_sliders_slow(blockers: Bitboard, sq: Square) -> Bitboard {
    const ROOK_DIRECTIONS: [(isize, isize); 4] = [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
    ];

    let mut bb = Bitboard::EMPTY;

    for dir in ROOK_DIRECTIONS {
        bb |= ray_march_sliders(blockers, sq, dir);
    }

    bb
}

pub fn get_bishop_sliders_slow(blockers: Bitboard, sq: Square) -> Bitboard {
    const BISHOP_DIRECTIONS: [(isize, isize); 4] = [
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    let mut bb = Bitboard::EMPTY;

    for dir in BISHOP_DIRECTIONS {
        bb |= ray_march_sliders(blockers, sq, dir);
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

pub struct SquareMoveData {
    pub mask: Bitboard,
    pub possible_blockers: Vec<Bitboard>,
    pub moves: Vec<Bitboard>,
}

impl SquareMoveData {
    const EMPTY: Self = Self {
        mask: Bitboard::EMPTY,
        possible_blockers: Vec::new(),
        moves: Vec::new(),
    };
}

pub const fn get_sq_rook_data() -> [SquareMoveData; 64] {
    let mut data = [SquareMoveData::EMPTY; 64];
    for (sq_idx, &sq) in Square::ALL.iter().enumerate() {
        let mask = ROOK_RELEVANT_OCCUPANCY[sq_idx];
        let possible_blockers = enumerate_occupancies(mask);

        let moves = possible_blockers
            .iter()
            .map(|&bb| get_rook_sliders_slow(bb, sq))
            .collect();

        data[sq_idx] = SquareMoveData {
            mask,
            possible_blockers,
            moves,
        }
    }
    data
}

pub const fn get_sq_bishop_data() -> [SquareMoveData; 64] {
    let mut data = [SquareMoveData::EMPTY; 64];
    for (sq_idx, &sq) in Square::ALL.iter().enumerate() {
        let mask = BISHOP_RELEVANT_OCCUPANCY[sq_idx];
        let possible_blockers = enumerate_occupancies(mask);

        let moves = possible_blockers
            .iter()
            .map(|&bb| get_bishop_sliders_slow(bb, sq))
            .collect();

        data[sq_idx] = SquareMoveData {
            mask,
            possible_blockers,
            moves,
        }
    }
    data
}

