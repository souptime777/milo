mod fen;
mod make_unmake;
mod movegen;
mod zobrist_state;

use crate::*;
use std::{fmt};
pub use zobrist_state::*;

pub use movegen::*;

/// A chess board.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Board {
    // Inner values
    inner: ZobristState,

    // The amount moves since the last capture or pawn move.
    halfmove_clock: u16,

    // The amount of full moves that have been played by both players
    fullmove_clock: u16,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (sq, piece) in self.inner.mailbox().iter().enumerate() {
            // Newline every 8 squares
            if sq != 0 && sq % 8 == 0 {
                writeln!(f)?;
            }

            // Print piece
            let char = if let Some(piece) = piece {
                char::from(*piece)
            } else {
                '·'
            };

            write!(f, "{} ", char)?;
        }

        Ok(())
    }
}

impl Default for Board {
    fn default() -> Self {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse::<Self>()
            .unwrap()
    }
}

impl Board {
    /// Get the [Piece] on a [Square].
    #[inline(always)]
    pub const fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.inner.mailbox()[sq as usize]
    }

    /// Get the [Bitboard] of a [PieceType].
    #[inline(always)]
    pub fn piece_type_bitboard(&self, piece_type: PieceType) -> Bitboard {
        self.inner.piece_type_bitboard(piece_type)
    }

    /// Get the [Bitboard] of a [Color].
    #[inline(always)]
    pub const fn color_bitboard(&self, color: Color) -> Bitboard {
        self.inner.color_bitboard(color)
    }

    /// Get the [Bitboard] of a [Piece].
    #[inline(always)]
    pub const fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        self.inner.piece_bitboard(piece)
    }

    /// Get the [Bitboard] of all pieces.
    #[inline(always)]
    pub const fn occupancy(&self) -> Bitboard {
        self.inner.occupancy()
    }

    /// Get the [CastlingRights] of a [Board].
    #[inline(always)]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.inner.castling_rights()
    }

    /// Get the optional en passant [Square] of a [Board].
    #[inline(always)]
    pub const fn en_passant(&self) -> Option<File> {
        self.inner.en_passant()
    }

    /// Get the halfmove clock of a [Board]
    #[inline(always)]
    pub const fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    /// Get the fullmove clock of a [Board]
    #[inline(always)]
    pub const fn fullmove_clock(&self) -> u16 {
        self.fullmove_clock
    }

    /// Get the square of a the king
    pub const fn king_square(&self, color: Color) -> Square {
        unsafe {
            std::mem::transmute(
                self.inner.piece_bitboards[color as usize * 6 + 5]
                    .0
                    .trailing_zeros() as u8,
            )
        }
    }
}
