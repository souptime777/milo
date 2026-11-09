use crate::*;

crate::types::helpers::define_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// An enum representing any one file (column) on a board.
    pub enum Rank {
        Eighth,
        Seventh,
        Sixth,
        Fifth,
        Fourth,
        Third,
        Second,
        First,
    }
}

impl From<Square> for Rank {
    #[inline(always)]
    fn from(value: Square) -> Self {
        match value as u64 / 8 {
            0 => Rank::Eighth,
            1 => Rank::Seventh,
            2 => Rank::Sixth,
            3 => Rank::Fifth,
            4 => Rank::Fourth,
            5 => Rank::Third,
            6 => Rank::Second,
            7 => Rank::First,
            _ => unreachable!()
        }
    }
}

impl TryFrom<u64> for Rank {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rank::Eighth),
            1 => Ok(Rank::Seventh),
            2 => Ok(Rank::Sixth),
            3 => Ok(Rank::Fifth),
            4 => Ok(Rank::Fourth),
            5 => Ok(Rank::Third),
            6 => Ok(Rank::Second),
            7 => Ok(Rank::First),
            _ => Err("Rank index must be between 0 and 7"),
        }
    }
}

crate::types::helpers::enum_char_conv! {
    Rank, RankParseError {
        First = '1',
        Second = '2',
        Third = '3',
        Fourth = '4',
        Fifth = '5',
        Sixth = '6',
        Seventh = '7',
        Eighth = '8'
    }
}

impl Rank {
    /// Get a bitboard with all squares on this rank.
    #[inline(always)]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(0b11111111 << (self as u8 * 8))
    }
}
