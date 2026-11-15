use crate::*;

const ROOK_TABLE_LEN: usize = 102_400;
const ROOK_TABLE_BYTES: &[u8; ROOK_TABLE_LEN * 8] =
    include_bytes!("generated_rook_table.bin");

const fn decode_rook_table() -> [Bitboard; ROOK_TABLE_LEN] {
    let mut table = [Bitboard::EMPTY; ROOK_TABLE_LEN];
    let mut i = 0;

    while i < ROOK_TABLE_LEN {
        let j = i * 8;
        table[i] = Bitboard(u64::from_le_bytes([
            ROOK_TABLE_BYTES[j],
            ROOK_TABLE_BYTES[j + 1],
            ROOK_TABLE_BYTES[j + 2],
            ROOK_TABLE_BYTES[j + 3],
            ROOK_TABLE_BYTES[j + 4],
            ROOK_TABLE_BYTES[j + 5],
            ROOK_TABLE_BYTES[j + 6],
            ROOK_TABLE_BYTES[j + 7],
        ]));
        i += 1;
    }

    table
}

pub static ROOK_TABLE: [Bitboard; ROOK_TABLE_LEN] = decode_rook_table();