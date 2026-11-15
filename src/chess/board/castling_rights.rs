use crate::Square;


const WHITE_KINGSIDE: u8 = 0b0001;
const WHITE_QUEENSIDE: u8 = 0b0010;
const BLACK_KINGSIDE: u8 = 0b0100;
const BLACK_QUEENSIDE: u8 = 0b1000;

const CASTLING_MASKS: [u8; 64] = [
    0b0111, 0b1111, 0b1111, 0b1111, 0b0011, 0b1111, 0b1111, 0b1011,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111, 0b1111,
    0b1101, 0b1111, 0b1111, 0b1111, 0b1100, 0b1111, 0b1111, 0b1110,
];

/// A small struct to hold a board's castling rights.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        if white_kingside { inner |= WHITE_KINGSIDE };
        if white_queenside { inner |= WHITE_QUEENSIDE };
        if black_kingside { inner |= BLACK_KINGSIDE };
        if black_queenside { inner |= BLACK_QUEENSIDE };

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