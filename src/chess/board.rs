use std::{
    str::FromStr,
    fmt,
};


use crate::*;

/// A small struct to hold a board's castling rights.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}
/// A chess board.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Board {
    // Bitboard for each PieceType and Color.
    piece_bitboards: [Bitboard; 12],
    color_bitboards: [Bitboard; 2],

    /* 
    An array that holds the same information as the bitboards.
    It is technically redundant information, but is faster to index in some cases
    */
    mailbox: [Option<Piece>; 64],

    // The color of the player who will play the next turn.
    side_to_move: Color,

    // Holds kingside + queenside castling rights for both players.
    castling_rights: CastlingRights,

    // The square behind a pawn if it were pushed to squares in the last move.
    ep_square: Option<Square>,

    // The amount moves since the last capture or pawn move.
    halfmove_clock: u16,

    // The amount of full moves that have been played by both players
    fullmove_clock: u16,
}

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

        let castling_rights = CastlingRights {
            white_kingside: castling_rights_field.contains('K'),
            white_queenside: castling_rights_field.contains('Q'),
            black_kingside: castling_rights_field.contains('k'),
            black_queenside: castling_rights_field.contains('q'),
        };

        let ep_square = match ep_square_field {
            "-" => None,
            _=> {
                Some(ep_square_field
                    .parse::<Square>()
                    .map_err(|_| FenParseError::InvalidEpSquare)?)
            }   
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
            halfmove_clock,
            fullmove_clock,
        })
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (sq, piece) in self.mailbox.iter().enumerate() { 

            // Newline every 8 squares
            if sq != 0 && sq % 8 == 0 {
                writeln!(f)?;
            }
            
            // Print piece
            let char = if let Some(piece) = piece {
                char::from(*piece)
            } else {
                '·'
            };
            
            write!(f, "{} ", char)?;
        }

        Ok(())
    }
}

impl Default for Board {
    fn default() -> Self {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse::<Self>().unwrap()
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
                        fen.push(char::from_digit(consecutive_empty_squares, 10)
                            .expect("Invalid digit"));
                        consecutive_empty_squares = 0;
                    }

                    fen.push(char::from(piece));
                } else {
                    consecutive_empty_squares += 1;
                }
                
            }

            if consecutive_empty_squares != 0 {
                fen.push(char::from_digit(consecutive_empty_squares, 10)
                    .expect("Invalid digit"));
            }

            if rank != Rank::First {

                fen.push('/');
            }
        }

        fen.push(' '); // Space separator
        fen.push(char::from(self.side_to_move)); // Side to move
        fen.push(' '); // Space separator

        // Castling rights
        if self.castling_rights.white_kingside
        || self.castling_rights.white_queenside
        || self.castling_rights.black_kingside
        || self.castling_rights.black_queenside {
            if self.castling_rights.white_kingside { fen.push('K') }
            if self.castling_rights.white_queenside { fen.push('Q') }
            if self.castling_rights.black_kingside { fen.push('k') }
            if self.castling_rights.black_queenside { fen.push('q') }
        } else {
            fen.push('-')
        }

        fen.push(' '); // Space separator

        let ep_square  = match self.ep_square{
            Some(square) => &square.to_string(),
            None => "-"
        };

        fen.push_str(ep_square); // En passant square
        fen.push(' '); // Space separator
        fen.push_str(&self.halfmove_clock.to_string()); // Halfmoves
        fen.push(' '); // Space separator
        fen.push_str(&self.fullmove_clock.to_string()); // Fullmoves

        fen
    }
    
    /// Get the [Piece] on a [Square].
    #[inline(always)]
    pub const fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.mailbox[sq as usize]
    }

    /// Get the [Bitboard] of a [PieceType].
    #[inline(always)]
    pub fn piece_type_bitboard(&self, piece_type: PieceType) -> Bitboard {
        self.piece_bitboards[piece_type as usize]
        | self.piece_bitboards[piece_type as usize + 6]
    }

    /// Get the [Bitboard] of a [Color].
    #[inline(always)]
    pub const fn color_bitboard(&self, color: Color) -> Bitboard {
        self.color_bitboards[color as usize]
    }

    /// Get the [Bitboard] of a [Piece].
    #[inline(always)]
    pub const fn piece_bitboard(&self, piece: Piece) -> Bitboard {
        self.piece_bitboards[piece as usize]
    }

    /// Get the [CastlingRights] of a [Board].
    #[inline(always)]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Get the optional en passant [Square] of a [Board].
    #[inline(always)]
    pub const fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    /// Get the halfmove clock of a [Board]
    #[inline(always)]
    pub const fn halfmove_clock(&self) -> u16 {
        self.halfmove_clock
    }

    /// Get the fullmove clock of a [Board]
    #[inline(always)]
    pub const fn fullmove_clock(&self) -> u16 {
        self.fullmove_clock
    }

    /// Put a piece on the board
    #[inline(always)]
    pub fn put_peice(&mut self, piece: Piece, sq: Square) {
        let sq_idx = sq as usize;
        let sq_bb = sq.bitboard();


        if let Some(captured) = self.mailbox[sq_idx] {
            self.piece_bitboards[captured as usize] ^= sq_bb;
        }

        self.piece_bitboards[piece as usize] |= sq_bb;
        self.mailbox[sq_idx] = Some(piece)
    }

    /// Remove a piece
    #[inline(always)]
    pub fn remove_piece(&mut self, sq: Square) {
        if let Some(piece) = self.piece_on(sq) {
            self.piece_bitboards[piece as usize] ^= sq.bitboard();
            self.mailbox[sq as usize] = None;
        }
    }

    /// Push a move to the board
    #[inline(always)]
    pub fn make_move(&mut self, mv: &Move) {
        let from = mv.from();
        let to = mv.to();
        let flag = mv.flag();
        match flag {
            MoveFlag::Quiet => {
                let piece = self.piece_on(from).unwrap();
                let move_mask = from.bitboard() | to.bitboard();

                // Update bitboards.
                self.piece_bitboards[piece as usize] ^= move_mask;

                // Update mailbox.
                self.mailbox.swap(from as usize, to as usize);
            }

            _ => todo!(),
        }

    }

}