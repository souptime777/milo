use crate::*;
use arrayvec::ArrayVec;

const MAX_MOVES_PER_POSITION: usize = 218;

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoredMove {
    mv: Move,
    score: i32,
}

impl ScoredMove {
    #[inline(always)]
    pub fn new(mv: Move, score: i32) -> Self {
        Self { mv, score }
    }
}

#[derive(Clone, Debug)]
pub struct MoveList(ArrayVec<ScoredMove, MAX_MOVES_PER_POSITION>);

impl MoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self ( ArrayVec::new() )
    }

    #[inline(always)]
    pub fn push(&mut self, entry: ScoredMove) {
        self.0.push(entry);
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    pub fn pop_best(&mut self) -> Option<Move> {
        let (max_idx, _) = self.0
            .iter()
            .enumerate()
            .max_by_key(|(_, scored_move)| scored_move.score)?;

        Some(self.0.swap_remove(max_idx).mv)
    }

    #[inline(always)]
    pub fn pop_move(&mut self) -> Option<Move> {
        Some(self.0.pop()?.mv)
    }

    pub fn score_with<F>(&mut self, mut scorer: F) 
    where 
        F: FnMut(&Move) -> i32 
    {
        for scored_move in self.0.iter_mut() {
            scored_move.score = scorer(&scored_move.mv)
        }
    }
}
