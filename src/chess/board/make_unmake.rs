use crate::*;

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
        self.checkers = Bitboard::EMPTY;
        self.pinned = Bitboard::EMPTY;

        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let color = self.inner.side_to_move;
        let moved_piece = self
            .piece_on(from)
            .expect("No piece on move's from square.");

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
}
