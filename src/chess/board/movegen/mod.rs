mod magics;
mod magic_data;
mod movelist;

use crate::*;

pub use movelist::*;
pub use magics::*;


#[derive(Debug)]
pub struct MoveGenerator<'a> {
    board: &'a Board,
}

impl<'a> MoveGenerator<'a> {
    fn new(board: &'a Board) -> Self {
        Self {
            board
        }
    }

    fn target_squares<const IN_CHECK: bool>(&self) -> Bitboard {
        let color = self.board.side_to_move();
        let targets = if IN_CHECK {
            let checker = self.board.checkers.first_square_unchecked();
            let our_king = self.board.king_square(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            Bitboard::FULL
        };
        targets & !self.board.color_bitboard(color)
    }

    fn add_pawn_captures<const IN_CHECK: bool>() {

    }

    /// Generate all capture moves
    pub fn generate_captures(&self) -> MoveList {
        let board = self.board;
        let move_list = MoveList::new();

        let _in_check = !board.checkers.is_empty();
        
        move_list
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
        attackers |= get_bishop_attacks(sq, self.occupancy()) & (
            self.piece_bitboard(Piece::new(PieceType::Bishop, enemy_color))
            | self.piece_bitboard(Piece::new(PieceType::Queen, enemy_color))
        );

        // Rook-like attacks
        attackers |= get_rook_attacks(sq, self.occupancy()) & (
            self.piece_bitboard(Piece::new(PieceType::Rook, enemy_color))
            | self.piece_bitboard(Piece::new(PieceType::Queen, enemy_color))
        );

        // King attacks
        attackers |= get_king_attacks(sq) & self.piece_bitboard(Piece::new(PieceType::King, enemy_color));
        attackers
    }

    pub fn generator(&self) -> MoveGenerator<'_> {
        MoveGenerator {
            board: self
        }
    }
}