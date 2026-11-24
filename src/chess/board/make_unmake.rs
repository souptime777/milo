use crate::*;
use config::MAX_SEARCH_PLIES;

impl Board {
    #[inline(always)]
    fn quiet_promotion_helper(
        &mut self,
        promo_piece: Piece,
        moved_piece: Piece,
        from: Square,
        to: Square,
    ) {
        // Pick up pawn
        self.inner.xor_square(moved_piece, from);

        // Drop new piece
        self.inner.xor_square(promo_piece, to);

        // Remove en passant square
        self.inner.set_en_passant(None);

        // Update halfmove clock
        self.halfmove_clock = 0;
    }
    #[inline(always)]
    fn capture_promotion_helper(
        &mut self,
        promo_piece: Piece,
        moved_piece: Piece,
        from: Square,
        to: Square,
    ) {
        let captured_piece = self
            .piece_on(to)
            .expect("Move is capture promo but to square is empty.");

        self.inner.xor_square(captured_piece, to);

        // Remove castling rights
        self.inner.update_castling_rights(from, to);

        // Promote as normal
        self.quiet_promotion_helper(promo_piece, moved_piece, from, to);
    }
    /// Push a move to the board
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let color = self.inner.side_to_move;
        let moved_piece = self
            .piece_on(from)
            .expect("No piece on move's from square.");

        let past_move = PastMove::new(
            &mv, 
            self.piece_on(to), 
            self.castling_rights(), 
            self.en_passant(), 
            self.halfmove_clock());

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

        match flag {
            MoveFlag::Quiet => {
                // Update bitboards.
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Remove en passant square
                self.inner.set_en_passant(None);

                // Update castling rights
                self.inner.update_castling_rights(from, to);

                // Update halfmove clock
                if moved_piece.piece_type() == PieceType::Pawn {
                    self.halfmove_clock = 0;
                } else {
                    self.halfmove_clock += 1;
                }
            }

            MoveFlag::DoublePawn => {
                // Update bitboards.
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Remove en passant square
                self.inner.set_en_passant(None);

                // Update castling rights
                self.inner.update_castling_rights(from, to);

                self.inner.set_en_passant(Some(to.file()));

                // Update halfmove clock
                self.halfmove_clock = 0;
            }

            MoveFlag::KingCastle => {
                // Update king
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Update rook
                let rook = Piece::new(PieceType::Rook, color);
                self.inner
                    .xor_square(rook, Square::H1.from_perspective(color));
                self.inner
                    .xor_square(rook, Square::F1.from_perspective(color));

                // Remove castling rights
                self.inner.update_castling_rights(from, to);

                // Remove en passant square
                self.inner.set_en_passant(None);

                // Update halfmove clock
                self.halfmove_clock += 1;
            }

            MoveFlag::QueenCastle => {
                // Update king
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Update rook
                let rook = Piece::new(PieceType::Rook, color);
                self.inner
                    .xor_square(rook, Square::A1.from_perspective(color));
                self.inner
                    .xor_square(rook, Square::D1.from_perspective(color));

                // Remove castling rights
                self.inner.update_castling_rights(from, to);

                // Remove en passant square
                self.inner.set_en_passant(None);

                // Update halfmove clock
                self.halfmove_clock += 1;
            }

            MoveFlag::Capture => {
                
                let captured = self
                    .piece_on(to)
                    .expect("move is capture but no captured piece");

                // Remove captured
                self.inner.xor_square(captured, to);

                // Move Piece
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Update ep square
                self.inner.set_en_passant(None);

                // Update castling rights
                self.inner.update_castling_rights(from, to);

                // Update halfmove clock
                self.halfmove_clock = 0;
            }

            MoveFlag::EnPassant => {
                // Move pawn
                self.inner.xor_square(moved_piece, from);
                self.inner.xor_square(moved_piece, to);

                // Remove pawn
                let ep_pawn_sq = Square::new(
                    self.en_passant().expect("ep move but no ep file"),
                    Rank::Fifth.from_perspective(color),
                );
                self.inner
                    .xor_square(Piece::new(PieceType::Pawn, !color), ep_pawn_sq);

                // Remove en passant square
                self.inner.set_en_passant(None);

                // Update halfmove clock
                self.halfmove_clock = 0;
            }

            MoveFlag::KnightPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Knight, color),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::BishopPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Bishop, color),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::RookPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Rook, color),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::QueenPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Queen, color),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::KnightPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Knight, color),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::BishopPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Bishop, color),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::RookPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Rook, color),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::QueenPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Queen, color),
                    moved_piece,
                    from,
                    to,
                );
            }
        }

        // Update turn 
        self.inner.toggle_side_to_move();

        // Update checkers (crucially after updating side to move!)
        self.update_checkers_and_pinned();

        // Increment fullmove clock if black (branchless)
        self.fullmove_clock += (color == Color::Black) as u16;
    }

    /// Undo the previous move.
    pub fn unmake_move(&mut self) {
        
        let past_move = self.move_history.pop().expect("Attempted to unmake with no move history.");

        let from = past_move.from();
        let to = past_move.to();
        let flag = past_move.flag();
        self.inner.toggle_side_to_move();
        self.inner.set_castling_rights(past_move.castling_rights());
        self.inner.set_en_passant(past_move.en_passant());
        self.halfmove_clock = past_move.halfmove_clock();
        
        let color = self.side_to_move();
        self.fullmove_clock -= (color == Color::Black) as u16;

        if past_move.is_promotion() {
            let promo_piece = Piece::new(past_move.promotion_piece(), color);
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

        if let Some(captured) = past_move.captured_piece() {
            self.inner.xor_square(captured, to);
        }

        
        (self.pinned, self.checkers) = self.pin_check_history.pop().unwrap();
        
    }
}
