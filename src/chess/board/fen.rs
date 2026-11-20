use crate::{chess::board::castling_rights::CastlingRights, *};

use std::str::FromStr;

#[derive(Debug)]
pub enum FenParseError {
    InvalidPosition,
    InvalidSideToMove,
    InvalidCastlingRights,
    InvalidEpSquare,
    InvalidHalfmoves,
    InvalidFullmoves,
    MissingField,
}

impl FromStr for Board {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();

        let position_field = fields.next().ok_or(FenParseError::MissingField)?;
        let side_to_move_field = fields.next().ok_or(FenParseError::MissingField)?;
        let castling_rights_field = fields.next().ok_or(FenParseError::MissingField)?;
        let ep_square_field = fields.next().ok_or(FenParseError::MissingField)?;
        let halfmove_field = fields.next().ok_or(FenParseError::MissingField)?;
        let fullmove_field = fields.next().ok_or(FenParseError::MissingField)?;

        let mut mailbox: [Option<Piece>; 64] = [None; 64];
        let mut current_piece_idx: usize = 0;

        let mut piece_bitboards = [Bitboard::EMPTY; 12];
        let mut color_bitboards = [Bitboard::EMPTY; 2];

        for char in position_field.chars() {
            if let Ok(piece) = Piece::try_from(char) {
                mailbox[current_piece_idx] = Some(piece);
                let bit = 1u64 << current_piece_idx;

                piece_bitboards[piece as usize] |= Bitboard(bit);
                color_bitboards[piece.color() as usize] |= Bitboard(bit);

                current_piece_idx += 1;
            } else if let Some(digit) = char.to_digit(9) {
                current_piece_idx += digit as usize;
            }
        }

        let side_to_move = side_to_move_field
            .parse::<Color>()
            .map_err(|_| FenParseError::InvalidSideToMove)?;

        let castling_rights = CastlingRights::new(
            castling_rights_field.contains('K'),
            castling_rights_field.contains('Q'),
            castling_rights_field.contains('k'),
            castling_rights_field.contains('q'),
        );

        let ep_square = match ep_square_field {
            "-" => None,
            _ => Some(
                ep_square_field
                    .parse::<Square>()
                    .map_err(|_| FenParseError::InvalidEpSquare)?,
            ),
        };

        let ep_pawn_square = if let Some(sq) = ep_square {
            Some(match sq.rank() {
                Rank::Sixth => Square::new(sq.file(), Rank::Fifth),
                Rank::Third => Square::new(sq.file(), Rank::Fourth),
                _ => {
                    return Err(FenParseError::InvalidEpSquare);
                }
            })
        } else {
            None
        };

        let halfmove_clock = halfmove_field
            .parse::<u16>()
            .map_err(|_| FenParseError::InvalidHalfmoves)?;

        let fullmove_clock = fullmove_field
            .parse::<u16>()
            .map_err(|_| FenParseError::InvalidFullmoves)?;

        Ok(Self {
            piece_bitboards,
            color_bitboards,
            mailbox,
            side_to_move,
            castling_rights,
            ep_square,
            ep_pawn_square,
            halfmove_clock,
            fullmove_clock,
        })
    }
}

impl Board {
    /// Parse a fen string from a board.
    /// It wraps the implementation of the [FromStr] trait.
    #[inline(always)]
    pub fn from_fen(fen: &str) -> Result<Board, FenParseError> {
        fen.parse::<Self>()
    }
    /// Create a fen string from the board.
    ///
    /// # Examples
    /// ```
    /// use milo_core::Board;
    ///
    /// let initial_fen = "K7/Q1R2R2/2R2R2/8/1P4P1/1kP2P2/3PP3/7Q w - - 0 1";
    /// let board = Board::from_fen(initial_fen).unwrap();
    ///
    /// assert_eq!(board.to_fen(), initial_fen);
    /// ```
    #[inline(always)]
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in Rank::ALL {
            let mut consecutive_empty_squares: u32 = 0; // Counter for the numbers in fen string

            for file in File::ALL {
                let square = Square::new(file, rank);

                if let Some(piece) = self.piece_on(square) {
                    if consecutive_empty_squares != 0 {
                        fen.push(
                            char::from_digit(consecutive_empty_squares, 10).expect("Invalid digit"),
                        );
                        consecutive_empty_squares = 0;
                    }

                    fen.push(char::from(piece));
                } else {
                    consecutive_empty_squares += 1;
                }
            }

            if consecutive_empty_squares != 0 {
                fen.push(char::from_digit(consecutive_empty_squares, 10).expect("Invalid digit"));
            }

            if rank != Rank::First {
                fen.push('/');
            }
        }

        fen.push(' '); // Space separator
        fen.push(char::from(self.side_to_move)); // Side to move
        fen.push(' '); // Space separator

        // Castling rights
        if self.castling_rights.white_kingside()
            || self.castling_rights.white_queenside()
            || self.castling_rights.black_kingside()
            || self.castling_rights.black_queenside()
        {
            if self.castling_rights.white_kingside() {
                fen.push('K')
            }
            if self.castling_rights.white_queenside() {
                fen.push('Q')
            }
            if self.castling_rights.black_kingside() {
                fen.push('k')
            }
            if self.castling_rights.black_queenside() {
                fen.push('q')
            }
        } else {
            fen.push('-')
        }

        fen.push(' '); // Space separator

        let ep_square = match self.ep_square {
            Some(square) => &square.to_string(),
            None => "-",
        };

        fen.push_str(ep_square); // En passant square
        fen.push(' '); // Space separator
        fen.push_str(&self.halfmove_clock.to_string()); // Halfmoves
        fen.push(' '); // Space separator
        fen.push_str(&self.fullmove_clock.to_string()); // Fullmoves

        fen
    }
}
