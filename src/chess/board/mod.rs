mod castling_rights;
mod fen;
mod make_unmake;
mod movegen;

use crate::*;
use castling_rights::CastlingRights;
use std::fmt;

pub use movegen::*;

/// A chess board.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Board {
    // Bitboard for each PieceType and Color.
    piece_bitboards: [Bitboard; 12],
    color_bitboards: [Bitboard; 2],

    /*
    An array that holds the same information as the bitboards.
    It is technically redundant information, but is faster to index in some cases
    */
    mailbox: [Option<Piece>; 64],

    // The color of the player who will play the next turn.
    side_to_move: Color,

    // Holds kingside + queenside castling rights for both players.
    castling_rights: CastlingRights,

    // The square behind a pawn if it were pushed to squares in the last move.
    ep_square: Option<Square>,

    // The square of a pawn that was pushed two squares in the last move.
    ep_pawn_square: Option<Square>,

    // The amount moves since the last capture or pawn move.
    halfmove_clock: u16,

    // The amount of full moves that have been played by both players
    fullmove_clock: u16,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (sq, piece) in self.mailbox.iter().enumerate() {
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
        self.mailbox[sq as usize]
    }

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

    /// Get the [CastlingRights] of a [Board].
    #[inline(always)]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Get the optional en passant [Square] of a [Board].
    #[inline(always)]
    pub const fn ep_square(&self) -> Option<Square> {
        self.ep_square
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
                self.piece_bitboards[color as usize * 6 + 5]
                    .0
                    .trailing_zeros() as u8,
            )
        }
    }

    /// Put a piece on the board
    #[inline(always)]
    pub fn put_peice(&mut self, piece: Piece, sq: Square) {
        let sq_idx = sq as usize;
        let sq_bb = sq.bitboard();

        if let Some(captured) = self.mailbox[sq_idx] {
            self.piece_bitboards[captured as usize] ^= sq_bb;
            self.color_bitboards[captured.color() as usize] ^= sq_bb;
        }

        self.piece_bitboards[piece as usize] |= sq_bb;
        self.color_bitboards[piece.color() as usize] |= sq_bb;
        self.mailbox[sq_idx] = Some(piece)
    }

    /// Remove a piece
    #[inline(always)]
    pub fn remove_piece(&mut self, sq: Square) {
        let sq_bb = sq.bitboard();

        if let Some(piece) = self.piece_on(sq) {
            self.piece_bitboards[piece as usize] ^= sq_bb;
            self.color_bitboards[piece.color() as usize] ^= sq_bb;
            self.mailbox[sq as usize] = None;
        }
    }
}
