use crate::*;

#[derive(Debug)]
pub struct Magic {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub size: usize,
}

pub fn generate_rook_magics() -> [Magic; 64] {
    let rook_data = get_sq_rook_data();
    let mut data_iter = rook_data.iter();
    
    std::array::from_fn(|_| {
        if let Some(data) = data_iter.next() {
            find_magic(data)
        } else {
            panic!();
        }
    })
}

pub fn generate_bishop_magics() -> [Magic; 64] {
    let bishop_data = get_sq_bishop_data();
    let mut data_iter = bishop_data.iter();
    
    std::array::from_fn(|_| {
        if let Some(data) = data_iter.next() {
            find_magic(data)
        } else {
            panic!();
        }
    })
}

pub struct Rng(u64);

impl Rng {
    pub fn new() -> Self {
        Self(0xFFAAB58C5833FE89)
    }

    /// Returns a random unsigned 64-bit integer.
    pub fn next_u64(&mut self) -> u64 {
        let mut number = self.0;

        number ^= number << 13;
        number ^= number >> 17;
        number ^= number << 5;

        self.0 = number;

        number
    }
}

fn find_magic(data: &SquareMoveData) -> Magic {
    let mut rng = Rng::new();

    let relevant_bits = data.mask.0.count_ones() as u8;
    let shift = 64 - relevant_bits;
    let table_size = 1_usize << relevant_bits;

    for _ in 0..1_000_000 {
        let magic = rng.next_u64() & rng.next_u64() & rng.next_u64();

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
            return Magic { mask: data.mask, magic, shift, size: table_size};
        }
    }

    panic!("Failed to find magic!")
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

pub fn get_sq_rook_data() -> [SquareMoveData; 64] {
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

pub fn get_sq_bishop_data() -> [SquareMoveData; 64] {
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

