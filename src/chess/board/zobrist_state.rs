use crate::{chess::square, *};

const WHITE_KINGSIDE: u8 = 0b0001;
const WHITE_QUEENSIDE: u8 = 0b0010;
const BLACK_KINGSIDE: u8 = 0b0100;
const BLACK_QUEENSIDE: u8 = 0b1000;

const CASTLING_MASKS: [u8; 64] = [
    0b0111, 0b1111, 0b1111, 0b1111, 0b0011, 0b1111, 0b1111, 0b1011, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1101, 0b1111, 0b1111, 0b1111,
    0b1100, 0b1111, 0b1111, 0b1110,
];

/// A small struct to hold a board's castling rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// Create new castling rights
    #[inline(always)]
    pub const fn new(
        white_kingside: bool,
        white_queenside: bool,
        black_kingside: bool,
        black_queenside: bool,
    ) -> Self {
        let mut inner: u8 = 0;
        if white_kingside {
            inner |= WHITE_KINGSIDE
        };
        if white_queenside {
            inner |= WHITE_QUEENSIDE
        };
        if black_kingside {
            inner |= BLACK_KINGSIDE
        };
        if black_queenside {
            inner |= BLACK_QUEENSIDE
        };

        Self(inner)
    }
    /// Update castling rights
    #[inline(always)]
    pub const fn update(&mut self, from: Square, to: Square) {
        self.0 &= CASTLING_MASKS[from as usize];
        self.0 &= CASTLING_MASKS[to as usize];
    }
    /// Get the white kingside castling rights
    #[inline(always)]
    pub const fn white_kingside(&self) -> bool {
        self.0 & WHITE_KINGSIDE != 0
    }

    /// Get the white queenside castling rights
    #[inline(always)]
    pub const fn white_queenside(&self) -> bool {
        self.0 & WHITE_QUEENSIDE != 0
    }

    /// Get the black kingside castling rights
    #[inline(always)]
    pub const fn black_kingside(&self) -> bool {
        self.0 & BLACK_KINGSIDE != 0
    }

    /// Get the black queenside castling rights
    #[inline(always)]
    pub const fn black_queenside(&self) -> bool {
        self.0 & BLACK_QUEENSIDE != 0
    }
}

#[derive(Debug)]
pub struct ColorZobristConstants {
    pieces: [[u64; 64]; 6],
}

#[derive(Debug)]
pub struct ZobristConstants {
    color: [ColorZobristConstants; 2],
    en_passant: [u64; 8],
    castle_rights: [u64; 16],
    black_to_move: u64
}

const ZOBRIST: ZobristConstants = {
    // Simple Pcg64Mcg impl
    let mut state = 0x7369787465656E2062797465206E756Du128 | 1;
    macro_rules! rand {
        () => {{
            state = state.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
            let rot = (state >> 122) as u32;
            let xsl = (state >> 64) as u64 ^ state as u64;
            xsl.rotate_right(rot)
        }};
    }

    macro_rules! fill_array {
        ($array:ident: $expr:expr) => {{
            let mut i = 0;
            while i < $array.len() {
                $array[i] = $expr;
                i += 1;
            }
        }};
    }

    macro_rules! color_zobrist_constant {
        () => {{
            let mut pieces = [[0; 64]; 6];
            fill_array!(pieces: {
                let mut squares = [0; 64];
                fill_array!(squares: rand!());
                squares
            });
            
            ColorZobristConstants {
                pieces,
            }
        }};
    }

    let mut en_passant = [0; 8];
    fill_array!(en_passant: rand!());

    let mut castle_rights = [0; 16];
    fill_array!(castle_rights: rand!());

    let white = color_zobrist_constant!();
    let black = color_zobrist_constant!();

    let black_to_move = rand!();

    ZobristConstants {
        color: [white, black],
        en_passant,
        black_to_move,
        castle_rights,
    }
};

/// A struct to hold the boards inner state required to hash it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ZobristState {
    // Bitboard for each PieceType and Color.
    pub piece_bitboards: [Bitboard; 12],
    pub color_bitboards: [Bitboard; 2],

    // Bitboard to hold colors.
    pub occupancy: Bitboard,

    /*
    An array that holds the same information as the bitboards.
    It is technically redundant information, but is faster to index in some cases
    */
    pub mailbox: [Option<Piece>; 64],

    // The color of the player who will play the next turn.
    pub side_to_move: Color,

    // Holds kingside + queenside castling rights for both players.
    pub castling_rights: CastlingRights,

    // The square behind a pawn if it were pushed two squares in the last move.
    pub en_passant: Option<File>,

    // The incrmentally updated hash.
    pub hash: u64,
}

impl ZobristState {
    /// Get the [Bitboard] of a [PieceType].
    #[inline(always)]
    pub fn piece_type_bitboard(&self, piece_type: PieceType) -> Bitboard {
        self.piece_bitboards[piece_type as usize] | self.piece_bitboards[piece_type as usize + 6]
    }

    /// Get the [Bitboard] of a [Color].
    #[inline(always)]
    pub const fn color_bitboard(&self, color: Color) -> Bitboard {
        self.color_bitboards[color as usize]
    }

    /// Get the [Bitboard] of a [Piece].
    #[inline(always)]
    pub const fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        self.piece_bitboards[piece as usize]
    }

    /// Get the mailbox.
    #[inline(always)]
    pub const fn mailbox(&self) -> [Option<Piece>; 64] {
        self.mailbox
    }

    /// Get the [Bitboard] of all pieces.
    #[inline(always)]
    pub const fn occupancy(&self) -> Bitboard {
        self.occupancy
    }

    /// Get the [CastlingRights] of a [Board].
    #[inline(always)]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Get the optional en passant [Square] of a [Board].
    #[inline(always)]
    pub const fn en_passant(&self) -> Option<File> {
        self.en_passant
    }

    /// Get the optional en passant [Square] of a [Board].
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Check whether this zobrist state is equal to another.
    #[inline(always)]
    pub fn state_is_equal(&self, other: &Self) -> bool {
        self.piece_bitboards == other.piece_bitboards
            && self.color_bitboards == other.color_bitboards
            && self.side_to_move == other.side_to_move
            && self.castling_rights == other.castling_rights
    }

    /// Run the XOR operation on relevant bb, updating hash and mailbox.
    pub fn xor_square(&mut self, piece: Piece, square: Square) {
        // debug check, no release overhead.
        debug_assert!(
            self.mailbox[square as usize].is_none() || self.mailbox[square as usize] == Some(piece),
            "xor_square mismatch at {:?}: mailbox had {:?}, tried to xor {:?}",
            square,
            self.mailbox[square as usize],
            piece
        );

        let square_bb = square.bitboard();

        self.piece_bitboards[piece as usize] ^= square_bb;
        self.color_bitboards[piece.color() as usize] ^= square_bb;
        self.occupancy ^= square_bb;

        let slot = &mut self.mailbox[square as usize];
        *slot = slot.xor(Some(piece));
    }

    /// Set the castling right, updating hash.
    pub fn update_castling_rights(&mut self, from: Square, to: Square) {
        let prev = self.castling_rights.0;
        self.castling_rights.update(from, to);
        
        self.hash ^= ZOBRIST.castle_rights[prev as usize];
        self.hash ^= ZOBRIST.castle_rights[self.castling_rights.0 as usize];
    }

    /// Set en passant value.
    pub fn set_en_passant(&mut self, new_en_passant: Option<File>) {
        if let Some(file) = core::mem::replace(&mut self.en_passant, new_en_passant) {
            self.hash ^= ZOBRIST.en_passant[file as usize];
        }

        if let Some(file) = self.en_passant {
            self.hash ^= ZOBRIST.en_passant[file as usize];
        }
    }

    #[inline(always)]
    pub fn toggle_side_to_move(&mut self) {
        self.side_to_move = !self.side_to_move;
        self.hash ^= ZOBRIST.black_to_move
    }
}