use crate::*;

crate::types::helpers::define_enum! {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// A chess piece that does not hold the color.
    pub enum PieceType {
        Pawn,
        Knight,
        Bishop,
        Rook,
        Queen,
        King
    }
}

crate::types::helpers::enum_char_conv! {
    PieceType, PieceTypeParseError {
        Pawn = 'p',
        Knight = 'n',
        Bishop = 'b',
        Rook = 'r',
        Queen = 'q',
        King = 'k'
    }
}

crate::types::helpers::define_enum! {
    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    /// A chess piece that holds the color.
    pub enum Piece {
        WhitePawn,
        WhiteKnight,
        WhiteBishop,
        WhiteRook,
        WhiteQueen,
        WhiteKing,
        BlackPawn,
        BlackKnight,
        BlackBishop,
        BlackRook,
        BlackQueen,
        BlackKing,
    }
}

crate::types::helpers::enum_char_conv! {
    Piece, PieceParseError {
        WhitePawn = 'P',
        WhiteKnight = 'N',
        WhiteBishop = 'B',
        WhiteRook = 'R',
        WhiteQueen = 'Q',
        WhiteKing = 'K',
        BlackPawn = 'p',
        BlackKnight = 'n',
        BlackBishop = 'b',
        BlackRook = 'r',
        BlackQueen = 'q',
        BlackKing = 'k'
    }
}

impl Piece {
    /// Get a new [Piece] from [PieceType] and [Color].
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        unsafe { std::mem::transmute((color as u8) * 6 + (piece_type as u8)) }
    }

    /// Get the underlying [PieceType].
    pub const fn piece_type(self) -> PieceType {
        unsafe { std::mem::transmute((self as u8) % 6) }
    }

    /// Get the underlying [Color].
    pub const fn color(self) -> Color {
        unsafe { std::mem::transmute((self as u8) / 6) }
    }
}