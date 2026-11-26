use std::fmt;

use crate::*;

/// A chess move.
///
/// Note that it is only valid for one board at one time, as it contains a [MoveFlag].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Move(pub u16);
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePawn = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    KnightPromo = 8,
    BishopPromo = 9,
    RookPromo = 10,
    QueenPromo = 11,
    KnightPromoCapture = 12,
    BishopPromoCapture = 13,
    RookPromoCapture = 14,
    QueenPromoCapture = 15,
}

#[derive(Debug)]
pub enum MoveParseError {
    InvalidFromSquare,
    InvalidToSquare,
    InvalidPromotionPiece,
    InvalidUciFormat,
    IllegalMove,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;
        if self.is_promotion() {
            write!(f, "{}", self.promotion_piece())?;
        }

        Ok(())
    }
}

impl Move {
    /// Create a new [Move]. Only to be used internally, as flag must be valid.
    #[inline(always)]
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Self((to as u16) | (from as u16) << 6 | (flag as u16) << 12)
    }
    #[inline(always)]
    pub fn from_uci(s: &str, board: &Board) -> Result<Self, MoveParseError> {
        if !(4..=5).contains(&s.len()) || !s.is_ascii() {
            return Err(MoveParseError::InvalidUciFormat);
        }

        let from: Square = s[0..=1].parse().map_err(|_| MoveParseError::InvalidFromSquare)?;
        let to: Square = s[2..=3].parse().map_err(|_| MoveParseError::InvalidToSquare)?;
        let promo  = s.chars()
            .nth(4)
            .map(|c| PieceType::try_from(c))
            .transpose()
            .map_err(|_| MoveParseError::InvalidPromotionPiece)?;

        let mut legals = MoveList::new();
        board.generate_moves_into(&mut legals);

        for legal in legals {
            if legal.from() == from
                && legal.to() == to {
                
                if !legal.is_promotion() && promo.is_some() {
                    continue
                }
                
                if legal.is_promotion() {
                    if let Some(promo) = promo
                    && legal.promotion_piece() == promo {
                        return Ok(legal);
                    }
                    continue;
                }
                return Ok(legal);
            }
        }

        Err(MoveParseError::IllegalMove)
    }
    /// Unpack the [Move]'s from square.
    #[inline(always)]
    pub const fn from(self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3F) as u8)
    }

    /// Unpack the [Move]'s to square.
    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::from_index((self.0 & 0x3F) as u8)
    }

    /// Unpack the [Move]'s flag.
    #[inline(always)]
    pub const fn flag(&self) -> MoveFlag {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    /// Return whether the [Move] is a capture.
    #[inline(always)]
    pub const fn is_capture(&self) -> bool {
        (self.0 & (1 << 14)) != 0
    }

    /// Return whether the [Move] is a promotion.
    #[inline(always)]
    pub const fn is_promotion(&self) -> bool {
        (self.0 & (1 << 15)) != 0
    }
    /// Return the [Move]'s promotion piece.
    ///
    /// If the [Move] is not a promotion, it will return a garbage value.
    #[inline(always)]
    pub const fn promotion_piece(&self) -> PieceType {
        PieceType::ALL[((self.0 >> 12) & 0b11) as usize + 1]
    }
}
