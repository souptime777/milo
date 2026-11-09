use core::fmt;

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Move(u16);
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

// #[derive(Debug)]
// pub enum ParseMoveError {
//     InvalidFromSquare,
//     InvalidToSquare,
//     InvalidPromotionPiece,
//     InvalidUciFormat,
// }

// impl FromStr for Move {
//     type Err = ParseMoveError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if !s.is_ascii() || s.len() < 4 || s.len() > 5 {
//             return Err(ParseMoveError::InvalidUciFormat);
//         }
//         Ok(Move {
//             from: s[0..2].parse::<Square>().map_err(|_| ParseMoveError::InvalidFromSquare)?,
//             to: s[2..4].parse::<Square>().map_err(|_| ParseMoveError::InvalidToSquare)?,
//             promotion: if s.len() == 5 {
//                 Some(s[4..5].parse::<Piece>().map_err(|_| ParseMoveError::InvalidPromotionPiece)?)
//             } else {
//                 None
//             },
//         })
//     }
// }

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;
        if self.is_promotion() {
            write!(f, "{}", self.promotion_piece())?;
        }
        
        Ok(())
    }
}

impl Move {


    /// Unpack the [Move]'s from square.
    #[inline(always)]
    pub const fn from(self) -> Square {
        Square::from_index((self.0 >> 6) as u8)
    }

    /// Unpack the [Move]'s to square.
    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::from_index((self.0) as u8)
    }

    /// Unpack the [Move]'s flag.
    #[inline(always)]
    pub const fn flag(self) -> MoveFlag {
        unsafe {std::mem::transmute((self.0 >> 12) as u8)}
    }

    /// Return whether the [Move] is a capture.
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        (self.0 & (1 << 14)) != 0 
    }

    /// Return whether the [Move] is a promotion.
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.0 & (1 << 15)) != 0
    }
    /// Return the [Move]'s promotion piece.
    /// 
    /// If the [Move] is not a promotion, it will return a garbage value.
    #[inline(always)]
    pub const fn promotion_piece(self) -> PieceType {
        PieceType::ALL[((self.0 >> 12) & 0b11) as usize + 1]
    }
}