pub mod chess;
pub mod uci;

pub use chess::{
    bitboard::*,
    board::*,
    color::*,
    file::*,
    piece::*,
    r#move::*,
    rank::*,
    square::*,
    tests,
};

pub mod config {
    pub const MAX_SEARCH_PLIES: u8 = 255;
    pub const HALFMOVE_RULE: u8 = 50;
}