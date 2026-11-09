use crate::*;
crate::types::helpers::define_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// An enum representing any one file (column) on a board.
    pub enum File {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
    }
}

impl From<Square> for File {
    #[inline(always)]
    fn from(value: Square) -> Self {
        match value as u64 % 8 {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => unreachable!()
        }
    }
}

impl TryFrom<u64> for File {
    type Error = &'static str;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(File::A),
            1 => Ok(File::B),
            2 => Ok(File::C),
            3 => Ok(File::D),
            4 => Ok(File::E),
            5 => Ok(File::F),
            6 => Ok(File::G),
            7 => Ok(File::H),
            _ => Err("File index must be between 0 and 7"),
        }
    }
}

crate::types::helpers::enum_char_conv! {
    File, FileParseError {
        A = 'a',
        B = 'b',
        C = 'c',
        D = 'd',
        E = 'e',
        F = 'f',
        G = 'g',
        H = 'h'
    }
}

impl File {
    #[inline(always)]
    /// Get a bitboard with all squares on this file.
    pub const fn bitboard(self) -> Bitboard {
        Bitboard(u64::from_ne_bytes([
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
            0b00000001,
        ]) << self as u8)
    }
}