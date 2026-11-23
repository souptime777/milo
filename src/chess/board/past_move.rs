use crate::*;

/// Contains all information to reconstruct a previous position.
/// 
/// 48 Total bits used out of 64:
/// - 16 bits: the [Move],
/// - 8 bits: the captured [Piece],
/// - 8 bits: previous [CastlingRights],
/// - 8 bits: previous optional en passant [File],
/// - 8 bits: previous halfmove clock,
/// - (16 bits: padding)
/// Since we use u64 anyways, we prefer to byte-align.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PastMove(u64);

impl PastMove {
    #[inline(always)]
    pub fn new(
        mv: &Move,
        captured: Option<Piece>,
        rights: CastlingRights,
        en_passant: Option<File>,
        halfmove_clock: u8,
    ) -> Self {
        // Compile time check to ensure Ferris did his job.
        const _: () = assert!(std::mem::size_of::<Option<File>>() == 1);
        const _: () = assert!(std::mem::size_of::<Option<Piece>>() == 1);
        Self(
            
            (mv.0 as u64)
            | (( unsafe{ std::mem::transmute::<Option<Piece>, u8>(captured) } as u64) << 16)
            | ((rights.0 as u64) << 24)
            | (( unsafe{ std::mem::transmute::<Option<File>, u8>(en_passant) } as u64) << 32)
            | ((halfmove_clock as u64) << 40)
        )
    }

    #[inline(always)]
    pub fn from(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline(always)]
    pub const fn to(self) -> Square {
        Square::from_index((self.0 & 0x3F) as u8)
    }

    #[inline(always)]
    pub const fn flag(&self) -> MoveFlag {
        unsafe { std::mem::transmute(((self.0 >> 12) & 0xF) as u8) }
    }

    #[inline(always)]
    pub const fn is_capture(&self) -> bool {
        (self.0 & (1 << 14)) != 0
    }

    #[inline(always)]
    pub const fn is_promotion(&self) -> bool {
        (self.0 & (1 << 15)) != 0
    }

    #[inline(always)]
    pub const fn promotion_piece(&self) -> PieceType {
        PieceType::ALL[((self.0 >> 12) & 0b11) as usize + 1]
    }

    #[inline(always)]
    pub const fn captured_piece(self) -> Option<Piece> {
        unsafe { std::mem::transmute(((self.0) >> 16) as u8) }
    }

    #[inline(always)]
    pub const fn castling_rights(self) -> CastlingRights {
        CastlingRights(((self.0 >> 24) & 0b1111) as u8)
    }

    #[inline(always)]
    pub const fn en_passant(&self) -> Option<File> {
        unsafe { std::mem::transmute((self.0 >> 32) as u8) }
    }

    #[inline(always)]
    pub const fn halfmove_clock(&self) -> u8 {
        (self.0 >> 40) as u8
    }
}

