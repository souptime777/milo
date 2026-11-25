mod fen;
mod make_unmake;
mod movegen;
mod past_move;
mod zobrist_state;

use crate::*;
use config::{HALFMOVE_RULE, MAX_SEARCH_PLIES};
use std::{fmt};
use movegen::*;
use arrayvec::ArrayVec;

pub use past_move::PastMove;
pub use zobrist_state::*;
pub use movegen::MoveList;
pub use fen::FenParseError;

/// A chess board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    // Inner values
    inner: ZobristState,

    // A bitboard of the pieces checking the king.
    checkers: Bitboard,

    // A bitboard of pinned pieces
    pinned: Bitboard,

    // The amount moves since the last capture or pawn move.
    halfmove_clock: u8,

    // The amount of full moves that have been played by both players
    fullmove_clock: u16,

    // The history required to rebuild previous positions
    move_history: ArrayVec<PastMove, {MAX_SEARCH_PLIES as usize}>,
    pin_check_history: ArrayVec<(Bitboard, Bitboard), {MAX_SEARCH_PLIES as usize}>
    
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::ALL {
            for file in File::ALL {
                let square = Square::new(file, rank);
                let char = match self.piece_on(square) {
                    Some(piece) => {
                        piece.image_char()
                    }
                    None => '.'
                };
                write!(f, "{} ", char)?;
            }
            writeln!(f)?;
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

    /// Get the side to move.
    #[inline(always)]
    pub const fn side_to_move(&self) -> Color {
        self.inner.side_to_move
    }

    /// Get the halfmove clock of a [Board]
    #[inline(always)]
    pub const fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    /// Get the fullmove clock of a [Board]
    #[inline(always)]
    pub const fn fullmove_clock(&self) -> u16 {
        self.fullmove_clock
    }

    /// Get the incrementally updated hash of the board
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.inner.hash()
    }

    /// Check whether a board is exactly equal 
    /// 
    /// Slower that comparing hash, but guarantees no collisions.
    /// Only counts position, not clocks.
    #[inline(always)]
    pub fn is_equal(&self, other: &Board) -> bool {
        self.inner.state_is_equal(&other.inner)
    }

    /// Get a Bitboard of the checking pieces.
    #[inline(always)]
    pub fn checkers(&self) -> Bitboard {
        self.checkers
    }

    /// Get a Bitboard of the pinned pieces
    #[inline(always)]
    pub fn pinned(&self) -> Bitboard {
        self.pinned
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

    #[inline(always)]
    pub const fn is_halfmove_rule(&self) -> bool {
        self.halfmove_clock >= HALFMOVE_RULE
    }
    /// Update checker bitboard
    #[inline(always)]
    fn update_checkers_and_pinned(&mut self) {
        self.checkers = Bitboard::EMPTY;
        self.pinned = Bitboard::EMPTY;

        let their_color = !self.side_to_move();

        let our_king = self.king_square(!their_color);
        let their_attackers = (get_bishop_rays(our_king)
            & (self.piece_bitboard(Piece::new(PieceType::Bishop, their_color))
                | self.piece_bitboard(Piece::new(PieceType::Queen, their_color))))
            | (get_rook_rays(our_king)
                & (self.piece_bitboard(Piece::new(PieceType::Rook, their_color))
                    | self.piece_bitboard(Piece::new(PieceType::Queen, their_color))));

        for square in their_attackers {
            let between = get_between_rays(square, our_king) & self.occupancy();

            match between.0.count_ones() {
                0 => self.checkers |= square.bitboard(),
                1 => self.pinned |= between,
                _ => {}
            }
        }

        self.checkers |= get_pawn_attacks(our_king, !their_color) & self.piece_bitboard(Piece::new(PieceType::Pawn, their_color));
        self.checkers |= get_knight_attacks(our_king) & self.piece_bitboard(Piece::new(PieceType::Knight, their_color));
    }
}