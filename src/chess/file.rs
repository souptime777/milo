use crate::*;
crate::chess::helpers::define_enum! {
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

crate::chess::helpers::enum_char_conv! {
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