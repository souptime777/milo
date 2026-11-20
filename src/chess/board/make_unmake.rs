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
        self.piece_bitboards[moved_piece as usize] ^= from.bitboard();
        self.color_bitboards[moved_piece.color() as usize] ^= from.bitboard();
        self.mailbox[from as usize] = None;

        // Drop new piece
        self.piece_bitboards[promo_piece as usize] ^= to.bitboard();
        self.color_bitboards[self.side_to_move as usize] ^= to.bitboard();
        self.mailbox[to as usize] = Some(promo_piece);

        // Remove en passant square
        self.ep_square = None;
        self.ep_pawn_square = None;

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
        let to_bb = to.bitboard();

        // Remove captured piece
        self.piece_bitboards[captured_piece as usize] ^= to_bb;
        self.color_bitboards[captured_piece.color() as usize] ^= to_bb;

        // Remove castling rights
        self.castling_rights.update(from, to);

        // Promote as normal
        self.quiet_promotion_helper(promo_piece, moved_piece, from, to);
    }
    /// Push a move to the board
    #[inline(always)]
    pub fn make_move(&mut self, mv: Move) {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        let moved_piece = self
            .piece_on(from)
            .expect("No piece on move's from square.");
        let move_mask = from.bitboard() | to.bitboard();

        match flag {
            MoveFlag::Quiet => {
                // Update bitboards.
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;

                // Update mailbox.
                self.mailbox.swap(from as usize, to as usize);

                // Remove en passant square
                self.ep_square = None;
                self.ep_pawn_square = None;

                // Update castling rights
                self.castling_rights.update(from, to);

                // Update halfmove clock
                if moved_piece.piece_type() == PieceType::Pawn {
                    self.halfmove_clock = 0;
                } else {
                    self.halfmove_clock += 1;
                }
            }

            MoveFlag::DoublePawn => {
                // Update bitboards.
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;

                // Update mailbox.
                self.mailbox.swap(from as usize, to as usize);

                // Update ep square
                self.ep_square = Some(match self.side_to_move {
                    Color::White => Square::new(to.file(), Rank::Third),
                    Color::Black => Square::new(to.file(), Rank::Sixth),
                });

                self.ep_pawn_square = Some(to);

                // Update halfmove clock
                if moved_piece.piece_type() == PieceType::Pawn {
                    self.halfmove_clock = 0;
                } else {
                    self.halfmove_clock += 1;
                }
            }

            MoveFlag::KingCastle => {
                // Update king bitboard.
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;

                // Update rook bitboard.
                let from_n = from.bitboard().0;
                let rook_mask = Bitboard(from_n << 1 | from_n << 3);
                self.piece_bitboards[Piece::new(PieceType::Rook, self.side_to_move) as usize] ^=
                    rook_mask;
                self.color_bitboards[self.side_to_move as usize] ^= rook_mask;

                // Update mailbox.
                let from_idx = from as usize;
                self.mailbox.swap(from_idx, to as usize);
                self.mailbox.swap(from_idx + 3, from_idx + 1);

                // Remove castling rights
                self.castling_rights.update(from, to);

                // Remove en passant square
                self.ep_square = None;
                self.ep_pawn_square = None;

                // Update halfmove clock
                self.halfmove_clock += 1;
            }

            MoveFlag::QueenCastle => {
                // Update King bitboard.
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;

                // Update rook bitboard.
                let from_n = from.bitboard().0;
                let rook_mask = Bitboard(from_n >> 1 | from_n >> 4);
                self.piece_bitboards[Piece::new(PieceType::Rook, self.side_to_move) as usize] ^=
                    rook_mask;
                self.color_bitboards[self.side_to_move as usize] ^= rook_mask;

                // Update mailbox.
                let from_idx = from as usize;
                self.mailbox.swap(from_idx, to as usize);
                self.mailbox.swap(from_idx - 4, from_idx - 1);

                // Remove castling rights
                self.castling_rights.update(from, to);

                // Remove en passant square
                self.ep_square = None;
                self.ep_pawn_square = None;

                // Update halfmove clock
                self.halfmove_clock += 1;
            }

            MoveFlag::Capture => {
                let captured_piece = self
                    .piece_on(to)
                    .expect("Move is capture but to square is empty.");
                let to_bb = to.bitboard();

                // Update bitboards
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;
                self.piece_bitboards[captured_piece as usize] ^= to_bb;
                self.color_bitboards[captured_piece.color() as usize] ^= to_bb;

                // Update mailbox
                self.mailbox[from as usize] = None;
                self.mailbox[to as usize] = Some(moved_piece);

                // Update castling rights
                self.castling_rights.update(from, to);

                // Remove en passant square
                self.ep_square = None;
                self.ep_pawn_square = None;

                // Update halfmove clock
                self.halfmove_clock = 0;
            }

            MoveFlag::EnPassant => {
                let ep_pawn_square = self
                    .ep_pawn_square
                    .expect("Move is en passant, but ep_square is none");
                // Move piece
                self.piece_bitboards[moved_piece as usize] ^= move_mask;
                self.color_bitboards[moved_piece.color() as usize] ^= move_mask;
                self.mailbox.swap(from as usize, to as usize);

                // Remove Pawn
                self.piece_bitboards[(!self.side_to_move) as usize * 6] ^=
                    ep_pawn_square.bitboard();
                self.color_bitboards[(!self.side_to_move) as usize] ^= ep_pawn_square.bitboard();
                self.mailbox[ep_pawn_square as usize] = None;

                // Remove en passant square
                self.ep_square = None;
                self.ep_pawn_square = None;

                // Update halfmove clock
                self.halfmove_clock = 0;
            }

            MoveFlag::KnightPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Knight, self.side_to_move),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::BishopPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Bishop, self.side_to_move),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::RookPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Rook, self.side_to_move),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::QueenPromo => self.quiet_promotion_helper(
                Piece::new(PieceType::Queen, self.side_to_move),
                moved_piece,
                from,
                to,
            ),

            MoveFlag::KnightPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Knight, self.side_to_move),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::BishopPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Bishop, self.side_to_move),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::RookPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Rook, self.side_to_move),
                    moved_piece,
                    from,
                    to,
                );
            }

            MoveFlag::QueenPromoCapture => {
                self.capture_promotion_helper(
                    Piece::new(PieceType::Queen, self.side_to_move),
                    moved_piece,
                    from,
                    to,
                );
            }
        }

        // Update misc data
        self.side_to_move = !self.side_to_move;

        // Increment fullmove clock if black (branchless)
        self.fullmove_clock += (self.side_to_move == Color::Black) as u16;

        // Update occupancy
        self.occupancy = self.color_bitboards[0] & self.color_bitboards[1];
    }
}
