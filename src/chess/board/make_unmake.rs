use crate::*;
use config::MAX_SEARCH_PLIES;
use crate::perft::perft;

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let color = self.inner.side_to_move;
        let moved = self
            .piece_on(from)
            .expect("No piece on move's from square.");
        let captured = self.piece_on(to);

        let past_move = PastMove::new(
            &mv, 
            captured,
            self.castling_rights(), 
            self.en_passant(), 
            self.halfmove_clock()
        );

        debug_assert!(self.move_history.len() < MAX_SEARCH_PLIES as usize);
        // SAFETY: Engine should never play beyond `MAX_SEARCH_PLIES`
        unsafe {
            self.move_history.push_unchecked(past_move);
            self.pin_check_history.push_unchecked(
                (
                    self.pinned(),
                    self.checkers(),
                )  
            ); 
        }

        // Default to none, but can be reset by double pawn push.
        let mut new_en_passant: Option<File> = None;
        // Remove the piece at from square
        self.inner.xor_square(moved, from);

        if mv.is_capture() {
            if flag == MoveFlag::EnPassant {
                // Remove enemy pawn
                let ep_file = self.en_passant().unwrap();
                let ep_square = Square::new(ep_file, Rank::Fifth.from_perspective(color));
                let victim_pawn = Piece::new(PieceType::Pawn, !color);
                self.inner.xor_square(victim_pawn, ep_square);
            } else {
                // Remove captured piece
                let victim = captured.unwrap();
                self.inner.xor_square(victim, to);
            }
            // Reset halfmove clock
            self.halfmove_clock = 0;
        } else {
            if moved.piece_type() == PieceType::Pawn {
                // Reset halfmove clock
                self.halfmove_clock = 0;
                if flag == MoveFlag::DoublePawn {
                    new_en_passant = Some(to.file());
                }
            } else {
                self.halfmove_clock += 1;
                if flag == MoveFlag::KingCastle {
                    let rook = Piece::new(PieceType::Rook, color);

                    self.inner.xor_square(rook, Square::H1.from_perspective(color));
                    self.inner.xor_square(rook, Square::F1.from_perspective(color));

                } else if flag == MoveFlag::QueenCastle {
                    let rook = Piece::new(PieceType::Rook, color);

                    self.inner.xor_square(rook, Square::A1.from_perspective(color));
                    self.inner.xor_square(rook, Square::D1.from_perspective(color));
                }
            }
        }

        if mv.is_promotion() {
            let promo = Piece::new(mv.promotion_piece(), color);
            self.inner.xor_square(promo, to);
        } else {
            // Only set new square once we know it's not promo
            self.inner.xor_square(moved, to);
        }

        // Update en passant
        self.inner.set_en_passant(new_en_passant);
        // Update turn 
        self.inner.toggle_side_to_move();
        // Update castling rights
        self.inner.update_castling_rights(from, to);
        // Update checkers (crucially after updating side to move!)
        self.update_checkers_and_pinned();
        // Increment fullmove clock if black (branchless)
        self.fullmove_clock += (color == Color::Black) as u16;
    }

    /// Undo the previous move.
    pub fn unmake_move(&mut self) {
        
        let p_mv = self.move_history.pop().expect("Attempted to unmake with no move history.");

        let from = p_mv.from();
        let to = p_mv.to();
        let flag = p_mv.flag();
        self.inner.toggle_side_to_move();
        self.inner.set_castling_rights(p_mv.castling_rights());
        self.inner.set_en_passant(p_mv.en_passant());
        self.halfmove_clock = p_mv.halfmove_clock();
        
        let color = self.side_to_move();
        self.fullmove_clock -= (color == Color::Black) as u16;

        if p_mv.is_promotion() {
            let promo_piece = Piece::new(p_mv.promotion_piece(), color);
            let pawn = Piece::new(PieceType::Pawn, color);
            self.inner.xor_square(promo_piece, to);
            self.inner.xor_square(pawn, from);
        } else {
            let moved_piece = self
                .piece_on(to)
                .expect("No piece on move's to square.");
            self.inner.xor_square(moved_piece, to);
            self.inner.xor_square(moved_piece, from);

            if flag == MoveFlag::KingCastle {
                let rook = Piece::new(PieceType::Rook, color);
                
                self.inner.xor_square(rook, Square::F1.from_perspective(color));
                self.inner.xor_square(rook, Square::H1.from_perspective(color));
            } else if flag == MoveFlag::QueenCastle {
                let rook = Piece::new(PieceType::Rook, color);
                
                self.inner.xor_square(rook, Square::A1.from_perspective(color));
                self.inner.xor_square(rook, Square::D1.from_perspective(color));
            } else if flag == MoveFlag::EnPassant {
                let their_pawn = Piece::new(PieceType::Pawn, !color);
                let ep_file = self.en_passant().unwrap(); // We've already set it!
                let victim_square = Square::new(ep_file, Rank::Fifth.from_perspective(color));

                self.inner.xor_square(their_pawn, victim_square);
            }
        }

        if let Some(captured) = p_mv.captured_piece() {
            self.inner.xor_square(captured, to);
        }

        
        (self.pinned, self.checkers) = self.pin_check_history.pop().unwrap();
    }
}

mod tests {
    use super::*;
    #[test]
    fn perft_test() {
        let expected: [u64; 5] = [
            20,
            400,
            8_902,
            197_281,
            4_865_609,
        ];
        for (depth, expected) in expected.into_iter().enumerate() {
            let mut board = Board::default();
            assert_eq!(perft(&mut board, (depth + 1) as u8), expected);
        }
    }
}
