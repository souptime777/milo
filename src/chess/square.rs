use crate::*;
use std::fmt;
use std::str::FromStr;

crate::chess::helpers::define_enum! {
    /// An enum representing any one square on a board.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Square {
        A8, B8, C8, D8, E8, F8, G8, H8,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A1, B1, C1, D1, E1, F1, G1, H1,
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", char::from(self.file()), char::from(self.rank()))
    }
}

#[derive(Debug)]
pub enum SquareParseError {
    InvalidFile,
    InvalidRank,
    MissingChar,
}

impl FromStr for Square {
    type Err = SquareParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        
        let file = File::try_from(
            chars
            .next()
            .ok_or(SquareParseError::MissingChar)?
        ).map_err(|_| SquareParseError::InvalidFile)?;

        let rank = Rank::try_from(
            chars
            .next()
            .ok_or(SquareParseError::MissingChar)?
        ).map_err(|_| SquareParseError::InvalidRank)?;

        Ok(Self::new(file, rank))
    }
}

impl Square {
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        let idx = (rank as usize * 8) + file as usize;
        Self::ALL[idx]
    }

    /// Get a bitboard with only the square occupied.
    #[inline(always)]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(1 << self as u64)
    }

    /// Get the file that the square is on.
    #[inline(always)]
    pub fn file(self) -> File {
        File::from(self)
    }

    /// Get the rank that the square is on.
    #[inline(always)]
    pub fn rank(self) -> Rank {
        Rank::from(self)
    }

    /// Get a square from an index. Uses only first 6 bits.
    pub const fn from_index(idx: u8) -> Self {
        Self::ALL[(idx & 63) as usize]
    }
}
