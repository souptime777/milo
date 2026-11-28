mod moves;
mod magic_data;

use crate::*;

pub use crate::chess::movelist::*;
pub use moves::*;

#[repr(u8)]
#[derive(Debug)]
pub enum GenType {
    All = 0,
    Captures = 1,
    Quiets = 2,
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
    #[inline(always)]
    fn target_squares<const IN_CHECK: bool, const GEN_TYPE: u8>(&self) -> Bitboard {
        let color = self.side_to_move();

        let targets = if IN_CHECK {
            let checker = self.checkers.first_square_unchecked();
            let our_king = self.king_square(color);
            get_between_rays(checker, our_king) | checker.bitboard()
        } else {
            Bitboard::FULL
        };

        let gen_type_mask = if GEN_TYPE == GenType::Captures as u8 {
            self.color_bitboard(!color)
        } else if GEN_TYPE == GenType::Quiets as u8 {
            !self.occupancy()
        } else {
            !self.color_bitboard(color)
        };

        targets & gen_type_mask
    }

    fn add_pawn_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let piece = Piece::new(PieceType::Pawn, color);
        let pieces = self.piece_bitboard(piece);
        let our_king = self.king_square(color);
        let their_pieces = self.color_bitboard(!color);
        let pinned = self.pinned;
        let blockers = self.occupancy();
        let targets = self.target_squares::<IN_CHECK, GEN_TYPE>();
        let our_seventh = Rank::Seventh.from_perspective(color).bitboard();

        // Generate non-promotions for non-pinned pawns
        for piece in pieces & !pinned & !our_seventh {
            if GEN_TYPE == GenType::Captures as u8
            || GEN_TYPE == GenType::All as u8 {
                let targets = targets & their_pieces;
                let to_squares = get_pawn_attacks(piece, color) & targets;

                for to in to_squares {
                    move_list.push_move(Move::new(piece, to, MoveFlag::Capture));
                }

            }
            if GEN_TYPE == GenType::Quiets as u8 
            || GEN_TYPE == GenType::All as u8 {
                let targets = targets & !their_pieces;
                let to_squares = get_pawn_quiets(piece, color, blockers) & targets;
                for to in to_squares {
                    let flag = if (piece as u8).abs_diff(to as u8) == 16 {
                        MoveFlag::DoublePawn
                    } else {
                        MoveFlag::Quiet
                    };
                    move_list.push_move(Move::new(piece, to, flag));
                }
            }
        }

        // Generate promotions for non-pinned pawns
        for piece in pieces & !pinned & our_seventh {
            if GEN_TYPE == GenType::Captures as u8
            || GEN_TYPE == GenType::All as u8 {
                let targets = targets & their_pieces;
                let to_squares = get_pawn_attacks(piece, color) & targets;

                for to in to_squares {
                    move_list.push_move(Move::new(piece, to, MoveFlag::KnightPromoCapture));
                    move_list.push_move(Move::new(piece, to, MoveFlag::BishopPromoCapture));
                    move_list.push_move(Move::new(piece, to, MoveFlag::RookPromoCapture));
                    move_list.push_move(Move::new(piece, to, MoveFlag::QueenPromoCapture));
                    
                }

            }
            if GEN_TYPE == GenType::Quiets as u8 
            || GEN_TYPE == GenType::All as u8 {
                let targets = targets & !their_pieces;
                let to_squares = get_pawn_quiets(piece, color, blockers) & targets;
                for to in to_squares {
                    move_list.push_move(Move::new(piece, to, MoveFlag::KnightPromo));
                    move_list.push_move(Move::new(piece, to, MoveFlag::BishopPromo));
                    move_list.push_move(Move::new(piece, to, MoveFlag::RookPromo));
                    move_list.push_move(Move::new(piece, to, MoveFlag::QueenPromo));
                }
            }
        }


        if !IN_CHECK {
            // Pinned pawns can't block checks
            for piece in pieces & pinned & !our_seventh {
                let targets = targets & get_line_rays(our_king, piece);
                if GEN_TYPE == GenType::Captures as u8
                || GEN_TYPE == GenType::All as u8 {
                    let to_squares = get_pawn_attacks(piece, color) & their_pieces & targets;

                    for to in to_squares {
                        move_list.push_move(Move::new(piece, to, MoveFlag::Capture));
                    }

                }
                if GEN_TYPE == GenType::Quiets as u8 
                || GEN_TYPE == GenType::All as u8 {
                    let to_squares = get_pawn_quiets(piece, color, blockers) & targets;
                    for to in to_squares {
                        let flag = if (piece as u8).abs_diff(to as u8) == 16 {
                            MoveFlag::DoublePawn
                        } else {
                            MoveFlag::Quiet
                        };
                        move_list.push_move(Move::new(piece, to, flag));
                    }
                }
            }

            for piece in pieces & pinned & our_seventh {
                let targets = targets & get_line_rays(our_king, piece);
                if GEN_TYPE == GenType::Captures as u8
                || GEN_TYPE == GenType::All as u8 {
                    let to_squares = get_pawn_attacks(piece, color) & their_pieces & targets;

                    for to in to_squares {
                        move_list.push_move(Move::new(piece, to, MoveFlag::KnightPromoCapture));
                        move_list.push_move(Move::new(piece, to, MoveFlag::BishopPromoCapture));
                        move_list.push_move(Move::new(piece, to, MoveFlag::RookPromoCapture));
                        move_list.push_move(Move::new(piece, to, MoveFlag::QueenPromoCapture));
                    }

                }
                if GEN_TYPE == GenType::Quiets as u8 
                || GEN_TYPE == GenType::All as u8 {
                    let to_squares = get_pawn_quiets(piece, color, blockers) & targets;
                    for to in to_squares {
                        move_list.push_move(Move::new(piece, to, MoveFlag::KnightPromo));
                        move_list.push_move(Move::new(piece, to, MoveFlag::BishopPromo));
                        move_list.push_move(Move::new(piece, to, MoveFlag::RookPromo));
                        move_list.push_move(Move::new(piece, to, MoveFlag::QueenPromo));
                    }
                }
            }
        }

        if (GEN_TYPE == GenType::Captures as u8
        || GEN_TYPE == GenType::All as u8)
        && let Some(en_passant) = self.en_passant() {

            let dest = Square::new(en_passant, Rank::Sixth.from_perspective(color));
            let victim = Square::new(en_passant, Rank::Fifth.from_perspective(color));

            if IN_CHECK { // Make sure EP is valid if we're in check
                let checker = self.checkers().first_square_unchecked();

                if checker != victim
                    && (get_between_rays(checker, our_king) & dest.bitboard()).is_empty()
                {
                    return;
                }
            }

            let their_queen = Piece::new(PieceType::Queen, !color);
            let their_bishop_likes = 
                self.piece_bitboard(Piece::new(PieceType::Bishop, !color)) |
                self.piece_bitboard(their_queen);
            let their_rook_likes =
                self.piece_bitboard(Piece::new(PieceType::Rook, !color)) |
                self.piece_bitboard(their_queen);
            
            for piece in get_pawn_attacks(dest, !color) & pieces {
                // Simulate the en passant, doing a ray check first to avoid magic lookups.

                let blockers = blockers ^ victim.bitboard() ^ piece.bitboard() | dest.bitboard();
                let on_ray = !(get_bishop_rays(our_king) & their_bishop_likes).is_empty();
                if on_ray && !(get_bishop_attacks(our_king, blockers) & their_bishop_likes).is_empty() {
                    continue;
                };
                let on_ray = !(get_rook_rays(our_king) & their_rook_likes).is_empty();
                if on_ray && !(get_rook_attacks(our_king, blockers) & their_rook_likes).is_empty() {
                    continue;
                };

                move_list.push_move(Move::new(piece, dest, MoveFlag::EnPassant));
            }
        }
    }

    #[inline(always)]
    fn add_knight_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let piece = Piece::new(PieceType::Knight, color);
        let pieces = self.piece_bitboard(piece);
        let pinned = self.pinned;
        let targets = self.target_squares::<IN_CHECK, GEN_TYPE>();
        let their_pieces = self.color_bitboard(!color);

        for piece in pieces & !pinned {
            let to_squares = get_knight_attacks(piece) & targets;
            for to in to_squares {
                let flag = if GEN_TYPE == GenType::Captures as u8 {
                    MoveFlag::Capture
                } else if GEN_TYPE == GenType::Quiets as u8 {
                    MoveFlag::Quiet
                } else if their_pieces.has(to) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };

                let mv = Move::new(piece, to, flag);
                move_list.push_move(mv);
            }
        }
    }
    #[inline]
    fn add_bishop_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let piece = Piece::new(PieceType::Bishop, color);
        let pieces = self.piece_bitboard(piece);
        let their_pieces = self.color_bitboard(!color);
        let our_king = self.king_square(color);
        let pinned = self.pinned;
        let blockers = self.occupancy();
        let targets = self.target_squares::<IN_CHECK, GEN_TYPE>();

        // Generate for non-pinned pieces
        for piece in pieces & !pinned {
            let to_squares = get_bishop_attacks(piece, blockers) & targets;
            for to in to_squares {
                let flag = if GEN_TYPE == GenType::Captures as u8 {
                    MoveFlag::Capture
                } else if GEN_TYPE == GenType::Quiets as u8 {
                    MoveFlag::Quiet
                } else if their_pieces.has(to) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                move_list.push_move(Move::new(piece, to, flag));
            }
        }

        if !IN_CHECK {
            // Pinned piece can't move if in check
            for piece in pieces & pinned {
                let targets = targets & get_line_rays(our_king, piece);
                let to_squares = get_bishop_attacks(piece, blockers) & targets;

                for to in to_squares {
                    let flag = if GEN_TYPE == GenType::Captures as u8 {
                        MoveFlag::Capture
                    } else if GEN_TYPE == GenType::Quiets as u8 {
                        MoveFlag::Quiet
                    } else if their_pieces.has(to) {
                        MoveFlag::Capture
                    } else {
                        MoveFlag::Quiet
                    };
                    move_list.push_move(Move::new(piece, to, flag));
                }
            }
        }
    }
    #[inline]
    fn add_rook_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let piece = Piece::new(PieceType::Rook, color);
        let pieces = self.piece_bitboard(piece);
        let our_king = self.king_square(color);
        let their_pieces = self.color_bitboard(!color);
        let pinned = self.pinned;
        let blockers = self.occupancy();
        let targets = self.target_squares::<IN_CHECK, GEN_TYPE>();

        // Generate for non-pinned pieces
        for piece in pieces & !pinned {
            let to_squares = get_rook_attacks(piece, blockers) & targets;
            for to in to_squares {
                let flag = if GEN_TYPE == GenType::Captures as u8 {
                    MoveFlag::Capture
                } else if GEN_TYPE == GenType::Quiets as u8 {
                    MoveFlag::Quiet
                } else if their_pieces.has(to) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                move_list.push_move(Move::new(piece, to, flag));
            }
        }

        if !IN_CHECK {
            // Pinned piece can't move if in check
            for piece in pieces & pinned {
                let targets = targets & get_line_rays(our_king, piece);
                let to_squares = get_rook_attacks(piece, blockers) & targets;

                for to in to_squares {
                    let flag = if GEN_TYPE == GenType::Captures as u8 {
                        MoveFlag::Capture
                    } else if GEN_TYPE == GenType::Quiets as u8 {
                        MoveFlag::Quiet
                    } else if their_pieces.has(to) {
                        MoveFlag::Capture
                    } else {
                        MoveFlag::Quiet
                    };
                    move_list.push_move(Move::new(piece, to, flag));
                }
            }
        }
    }
    #[inline]
    fn add_queen_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let piece = Piece::new(PieceType::Queen, color);
        let pieces = self.piece_bitboard(piece);
        let our_king = self.king_square(color);
        let pinned = self.pinned;
        let blockers = self.occupancy();
        let targets = self.target_squares::<IN_CHECK, GEN_TYPE>();
        let their_pieces = self.color_bitboard(!color);

        // Generate for non-pinned pieces
        for piece in pieces & !pinned {
            let to_squares = get_queen_attacks(piece, blockers) & targets;
            for to in to_squares {
                let flag = if GEN_TYPE == GenType::Captures as u8 {
                    MoveFlag::Capture
                } else if GEN_TYPE == GenType::Quiets as u8 {
                    MoveFlag::Quiet
                } else if their_pieces.has(to) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                move_list.push_move(Move::new(piece, to, flag));
            }
        }

        if !IN_CHECK {
            // Pinned piece can't move if in check
            for piece in pieces & pinned {
                let targets = targets & get_line_rays(our_king, piece);
                let to_squares = get_queen_attacks(piece, blockers) & targets;

                for to in to_squares {
                    let flag = if GEN_TYPE == GenType::Captures as u8 {
                        MoveFlag::Capture
                    } else if GEN_TYPE == GenType::Quiets as u8 {
                        MoveFlag::Quiet
                    } else if their_pieces.has(to) {
                        MoveFlag::Capture
                    } else {
                        MoveFlag::Quiet
                    };
                    move_list.push_move(Move::new(piece, to, flag));
                }
            }
        }
    }

    #[inline(always)]
    fn is_king_safe(&self, square: Square) -> bool {
        let color = self.side_to_move();
        let blockers = self.occupancy() 
            ^ self.piece_bitboard(Piece::new(PieceType::King, color))
            | square.bitboard();
            if !(get_pawn_attacks(square, color)
            & self.piece_bitboard(Piece::new(PieceType::Pawn, !color))).is_empty() {
                return false;
            }
            if !(get_knight_attacks(square) 
            & self.piece_bitboard(Piece::new(PieceType::Knight, !color))).is_empty() {
                return false;
            }
            if !(get_bishop_attacks(square, blockers) & (
                self.piece_bitboard(Piece::new(PieceType::Bishop, !color))
                | self.piece_bitboard(Piece::new(PieceType::Queen, !color))
            )).is_empty() {
                return false;
            }
            if !(get_rook_attacks(square, blockers) & (
                self.piece_bitboard(Piece::new(PieceType::Rook, !color))
                | self.piece_bitboard(Piece::new(PieceType::Queen, !color))
            )).is_empty() {
                return false;
            }
            if !(get_king_attacks(square)
            & self.piece_bitboard(Piece::new(PieceType::King, !color))).is_empty() {
                return false
            }

        true
    }
    #[inline(always)]
    fn can_castle(&self, rook: File, king_dest: File) -> bool {
        let color = self.side_to_move();
        let backrank = Rank::First.from_perspective(color);
        let rook = Square::new(rook, backrank);
        let king = self.king_square(color);
        let king_dest = Square::new(king_dest, backrank);
        let blockers = self.occupancy() ^ king.bitboard() ^ rook.bitboard();
        let king_to_rook = get_between_rays(king, rook);
        let king_to_dest = get_between_rays(king, king_dest);

        let must_be_safe = king_to_dest | king_dest.bitboard();
        let must_be_empty = must_be_safe | king_to_rook;

        (blockers & must_be_empty).is_empty()
        && (must_be_safe.into_iter().all(|square| self.is_king_safe(square)))
    }
    #[inline]
    fn add_king_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        let color = self.side_to_move();
        let our_king = self.king_square(color);
        let our_pieces = self.color_bitboard(color);
        let their_pieces = self.color_bitboard(!color);

        let targets = if GEN_TYPE == GenType::Captures as u8 {
            self.color_bitboard(!color)
        } else if GEN_TYPE == GenType::Quiets as u8 {
            !self.occupancy()
        } else {
            !our_pieces
        };
        for to in get_king_attacks(our_king) & targets {
            if self.is_king_safe(to) {
                let flag = if GEN_TYPE == GenType::Captures as u8 {
                    MoveFlag::Capture
                } else if GEN_TYPE == GenType::Quiets as u8 {
                    MoveFlag::Quiet
                } else if their_pieces.has(to) {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                };
                
                move_list.push_move(Move::new(our_king, to, flag));
            }
        }
        let rights = self.castling_rights();
        if !IN_CHECK & 
        (GEN_TYPE == GenType::Quiets as u8 || GEN_TYPE == GenType::All as u8) 
        & rights.any() {
            let rights = rights.color_rights(color);
            let back_rank = Rank::First.from_perspective(color);
            if rights.kingside {
                let rook = File::H;
                let king_dest = File::G;
                if self.can_castle(rook, king_dest) {
                    move_list.push_move(Move::new(
                        our_king,
                        Square::new(king_dest, back_rank), 
                        MoveFlag::KingCastle,
                    ));
                }
            }
            if rights.queenside {
                let rook = File::A;
                let king_dest = File::C;
                if self.can_castle(rook, king_dest) {
                    move_list.push_move(Move::new(
                        our_king,
                        Square::new(king_dest, back_rank), 
                        MoveFlag::QueenCastle,
                    ));
                }
            }
        }
    }

    fn add_all_moves<
        const IN_CHECK: bool, const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {
        self.add_pawn_moves::<IN_CHECK, GEN_TYPE>(move_list);
        self.add_knight_moves::<IN_CHECK, GEN_TYPE>(move_list);
        self.add_bishop_moves::<IN_CHECK, GEN_TYPE>(move_list);
        self.add_rook_moves::<IN_CHECK, GEN_TYPE>(move_list);
        self.add_queen_moves::<IN_CHECK, GEN_TYPE>(move_list);
        self.add_king_moves::<IN_CHECK, GEN_TYPE>(move_list);
    }

    /// Generate legal moves with a [GenType] into a Movelist
    #[inline(always)]
    pub fn generate_moves_with_type_into<
        const GEN_TYPE: u8
    >(&self, move_list: &mut MoveList) {      

        match self.checkers().len() {
            0 => self.add_all_moves::<false, GEN_TYPE>(move_list),
            1 => self.add_all_moves::<true, GEN_TYPE>(move_list),
            2 => self.add_king_moves::<true, GEN_TYPE>(move_list),
            _ => unreachable!("more than 2 checkers")
        }
    }

    /// Generate all legal moves
    #[inline(always)]
    pub fn generate_moves_into(&self, move_list: &mut MoveList) {
        self.generate_moves_with_type_into::<{ GenType::All as u8 }>(move_list);
    }

    /// Generate all legal capture moves
    #[inline(always)]
    pub fn generate_captures_into(&self, move_list: &mut MoveList) {
        self.generate_moves_with_type_into::<{ GenType::Captures as u8 }>(move_list);
    }

    /// Generate all legal quiet moves
    #[inline(always)]
    pub fn generate_quiets_into(&self, move_list: &mut MoveList) {
        self.generate_moves_with_type_into::<{ GenType::Quiets as u8 }>(move_list);
    }
}