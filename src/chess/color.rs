use core::ops::Not;

crate::chess::helpers::define_enum! {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// The color assigned to either player.
    pub enum Color {
        White,
        Black,
    }
}

crate::chess::helpers::enum_char_conv! {
    Color, ColorParseError {
        White = 'w',
        Black = 'b'
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}