use crate::*;
use arrayvec::ArrayVec;

const MAX_MOVES_PER_POSITION: usize = 218;

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredMove {
    mv: Move,
    score: i32,
}

#[derive(Clone, Debug)]
pub struct MoveList(ArrayVec<Move, MAX_MOVES_PER_POSITION>);

impl MoveList {
    pub fn new() -> Self {
        Self ( ArrayVec::new() )
    }

    pub fn push(&mut self, mv: Move) {
        self.0.push(mv);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}
