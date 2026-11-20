mod magics;
mod magic_data;
mod movelist;

use crate::*;

use movelist::*;
use magics::*;


#[derive(Debug)]
pub struct MoveGenerator {
    piece_bitboards: [Bitboard; 12],
    occupancy: Bitboard,
    side_to_move: Color,
}

impl MoveGenerator {
    pub fn generate_captures() {
        
    }
}

impl Board {
    /// Gets a bitboard of the locations of attacking pieces
    pub fn attackers_to(&self, sq: Square, enemy_color: Color) -> Bitboard {
        let mut attackers = Bitboard::EMPTY;

        // Pawn attacks
        attackers |= get_pawn_attacks(sq, !enemy_color) & self.piece_bitboard(Piece::new(PieceType::Pawn, enemy_color));

        // Knight attacks
        attackers |= get_knight_attacks(sq) & self.piece_bitboard(Piece::new(PieceType::Knight, enemy_color));

        // Bishop-like attacks
        attackers |= get_bishop_attacks(sq, self.occupancy) & (
            self.piece_bitboard(Piece::new(PieceType::Bishop, enemy_color))
            | self.piece_bitboard(Piece::new(PieceType::Queen, enemy_color))
        );

        // Rook-like attacks
        attackers |= get_bishop_attacks(sq, self.occupancy) & (
            self.piece_bitboard(Piece::new(PieceType::Rook, enemy_color))
            | self.piece_bitboard(Piece::new(PieceType::Queen, enemy_color))
        );

        // King attacks
        attackers |= get_king_attacks(sq) & self.piece_bitboard(Piece::new(PieceType::King, enemy_color));
        attackers
    }

    pub fn generator() -> MoveGenerator {
        todo!()
    }
}